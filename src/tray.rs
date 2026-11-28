// The tray (menu bar) app.
//   - a worker thread follows the Claude app log and does the copying and cleanup;
//   - the main thread owns the icon, the menu and dialogs (macOS and Linux need that).
// Menu: auto sync on/off; for when a sign-out was missed, move sessions now (shown
// while copies are deleted after sign-in) or sync all accounts now (shown while copies
// are kept); delete copies after sign-in,
// copy/delete speed with a time estimate, start with the computer, Language,
// log, account names, quit. UI text comes from locales.csv.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tray_icon::menu::{CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::core::{self, Account, Estimate, LogEvent, Report};
use crate::i18n::Strings;
use crate::{icon, platform};

const MOVE_POS: usize = 3; // header, separator, auto sync, then move now / sync all
const SETTLE: Duration = Duration::from_secs(6); // the app reads its session folder twice within ~2 s of sign-in

enum Cmd {
    AutoSync(bool),
    Move(Account),
    SyncAll,
    Refresh,
    Quit,
}

enum UiEvent {
    Menu(MenuEvent),
    Tray,
    Info { accounts: Result<Vec<Account>, String>, estimate: Option<Estimate> },
    Notify(String),
    LowDisk { need: u64, free: u64 },
    MoveDone(String, bool),
    WorkerDone,
}

// ---------- notifications ----------

fn notify(text: &str) {
    let mut n = notify_rust::Notification::new();
    n.summary("CCDC").body(text).appname("CCDC");
    #[cfg(windows)]
    {
        // toasts need the AppUserModelID that the installer put on the Start menu shortcut
        if let Some(p) = std::env::var_os("APPDATA").map(PathBuf::from) {
            if p.join(r"Microsoft\Windows\Start Menu\Programs\Continuous Claude Desktop Code.lnk").exists() {
                n.app_id(crate::AUMID);
            }
        }
    }
    if let Err(e) = n.show() {
        core::log(&format!("notification failed: {e}"));
    }
}

fn report_text(s: &Strings, r: &Report) -> String {
    match r {
        Report::CopyStarted { files, bytes, seconds } => {
            format!("{} {}", s.t("ev_copy_start", &[files, &core::mb(*bytes)]), s.t("eta_seconds", &[seconds]))
        }
        Report::CopyDone { files, seconds } => s.t("ev_copy_done", &[files, &format!("{seconds:.1}")]),
        Report::NothingToCopy => s.t("ev_signout_nothing", &[]),
        Report::LowDisk { .. } => String::new(),
        Report::Cleaned { removed, account } => s.t("ev_cleanup", &[removed, account]),
        Report::LateMove { moved } => s.t("ev_late_move", &[moved]),
        Report::Failed(e) => s.t("ev_failed", &[e]),
    }
}

// ---------- worker: follows the app log, runs the jobs ----------

struct Worker {
    proxy: EventLoopProxy<UiEvent>,
    busy: Arc<AtomicBool>,
    data_dir: Option<PathBuf>,
    app_log: Option<PathBuf>,
    follower: Option<core::LogFollower>,
    auto_sync: bool,
    signed_in: Option<(String, String)>,
    signin_due: Option<(Instant, String, String)>,
    last_error: String,
}

impl Worker {
    fn send(&self, e: UiEvent) {
        let _ = self.proxy.send_event(e);
    }

    fn report(&self, r: Report) {
        match r {
            Report::LowDisk { need, free } => self.send(UiEvent::LowDisk { need, free }),
            r => {
                let s = Strings::load(&core::load_settings().language);
                self.send(UiEvent::Notify(report_text(&s, &r)));
            }
        }
    }

    /// Finds the app's folders; retried until they exist (e.g. the app is installed later).
    fn locate(&mut self) -> bool {
        if self.data_dir.is_some() {
            return true;
        }
        match core::resolve_data_dir(None) {
            Ok(d) => {
                self.app_log = core::resolve_app_log(&d, None);
                core::log(&format!("Watching {}", d.display()));
                match &self.app_log {
                    Some(l) => core::log(&format!("Following app log {}", l.display())),
                    None => core::log("Claude app log (main.log) not found; set \"appLog\" in ccdc.json"),
                }
                self.follower = self.app_log.clone().map(core::LogFollower::new);
                if let Some(LogEvent::SignedIn { account, org }) = core::read_app_state(self.app_log.as_deref()) {
                    self.signed_in = Some((account, org));
                }
                self.data_dir = Some(d);
                true
            }
            Err(e) => {
                if e != self.last_error {
                    core::log(&e);
                    self.last_error = e;
                }
                false
            }
        }
    }

    fn run_job(&mut self, job: impl FnOnce(&mut Worker, &PathBuf)) {
        let Some(d) = self.data_dir.clone() else { return };
        self.busy.store(true, Ordering::SeqCst);
        job(self, &d);
        self.busy.store(false, Ordering::SeqCst);
        self.send(UiEvent::WorkerDone);
    }

    fn startup(&mut self) {
        let log = self.app_log.clone();
        self.run_job(|w, d| core::on_startup(d, log.as_deref(), &mut |r| w.report(r)));
    }

    fn poll_log(&mut self) {
        let Some(f) = self.follower.as_mut() else { return };
        for ev in f.poll() {
            match ev {
                LogEvent::SignedOut => {
                    self.signed_in = None;
                    self.signin_due = None;
                    self.run_job(|w, d| core::on_sign_out(d, "Sign-out detected", &mut |r| w.report(r)));
                }
                LogEvent::SignedIn { account, org } => {
                    let changed = self.signed_in.as_ref() != Some(&(account.clone(), org.clone()));
                    if changed {
                        self.signed_in = Some((account.clone(), org.clone()));
                        let keep = core::load_settings().keep_copies;
                        let what = if keep { "copies kept".to_string() } else { format!("removing copies in {}s", SETTLE.as_secs()) };
                        core::log(&format!("Sign-in detected: {}/{}, {what}", core::short(&account), core::short(&org)));
                        let s = Strings::load(&core::load_settings().language);
                        let text = if keep {
                            s.t("ev_signin_keep", &[&core::short(&account)])
                        } else {
                            s.t("ev_signin", &[&core::short(&account), &SETTLE.as_secs()])
                        };
                        self.send(UiEvent::Notify(text));
                    }
                    self.signin_due = Some((Instant::now() + SETTLE, account, org));
                }
            }
        }
    }

    fn refresh(&self) {
        let settings = core::load_settings();
        let (accounts, estimate) = match &self.data_dir {
            Some(d) => (core::accounts(d, self.app_log.as_deref(), &settings), core::estimate(d, &settings).ok()),
            None => (Err(self.last_error.clone()), None),
        };
        self.send(UiEvent::Info { accounts, estimate });
    }

    fn manual_move(&mut self, acc: Account) {
        let Some(d) = self.data_dir.clone() else {
            self.send(UiEvent::MoveDone(self.last_error.clone(), false));
            return;
        };
        let settings = core::load_settings();
        let s = Strings::load(&settings.language);
        let name = account_name(&acc);
        self.busy.store(true, Ordering::SeqCst);
        let res = core::consolidate(&d, &acc.account, None, false, false, &settings);
        self.busy.store(false, Ordering::SeqCst);
        let (text, ok) = match res {
            Ok(r) if !r.found => (s.t("move_no_folder", &[&name]), false),
            Ok(r) if r.planned.is_empty() => (s.t("move_nothing", &[&name]), true),
            Ok(r) => {
                core::log(&format!("manual: moved {} file(s), removed {} copies, replaced {}", r.moved, r.removed, r.replaced));
                (s.t("move_done", &[&name, &r.moved, &r.removed, &r.replaced]), true)
            }
            Err(e) => {
                core::log(&format!("manual: move failed: {e}"));
                (s.t("move_failed", &[&e]), false)
            }
        };
        self.send(UiEvent::MoveDone(text, ok));
        self.send(UiEvent::WorkerDone);
    }

    /// Copies every session into every account folder now: the sign-out copy, run by
    /// hand when the app's sign-out was not detected.
    fn manual_sync(&mut self) {
        if self.data_dir.is_none() {
            self.send(UiEvent::Notify(Strings::load(&core::load_settings().language).t("ev_failed", &[&self.last_error])));
            return;
        }
        self.run_job(|w, d| {
            let s = Strings::load(&core::load_settings().language);
            core::on_sign_out(d, "Manual sync", &mut |r| match r {
                Report::CopyStarted { files, bytes, seconds } => w.send(UiEvent::Notify(format!(
                    "{} {}",
                    s.t("sync_start", &[&files, &core::mb(bytes)]),
                    s.t("eta_seconds", &[&seconds])
                ))),
                Report::CopyDone { files, seconds } => {
                    w.send(UiEvent::Notify(s.t("sync_done", &[&files, &format!("{seconds:.1}")])))
                }
                Report::NothingToCopy => w.send(UiEvent::Notify(s.t("sync_nothing", &[]))),
                r => w.report(r),
            });
        });
    }

    fn run(mut self, rx: Receiver<Cmd>) {
        if self.locate() {
            self.startup();
        }
        let mut last_locate = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Cmd::Quit) | Err(RecvTimeoutError::Disconnected) => return,
                Ok(Cmd::AutoSync(on)) => {
                    self.auto_sync = on;
                    self.signin_due = None;
                    if on && self.locate() {
                        // catch up with whatever happened while it was off
                        if let Some(f) = self.follower.as_mut() {
                            f.skip_to_end();
                        }
                        self.startup();
                    }
                }
                Ok(Cmd::Move(acc)) => self.manual_move(acc),
                Ok(Cmd::SyncAll) => self.manual_sync(),
                Ok(Cmd::Refresh) => self.refresh(),
                Err(RecvTimeoutError::Timeout) => {}
            }
            if self.data_dir.is_none() && last_locate.elapsed() > Duration::from_secs(10) {
                last_locate = Instant::now();
                if self.locate() && self.auto_sync {
                    self.startup();
                }
            }
            if !self.auto_sync {
                continue;
            }
            self.poll_log();
            if let Some((due, _, _)) = &self.signin_due {
                if Instant::now() >= *due {
                    let (_, account, org) = self.signin_due.take().unwrap();
                    self.run_job(|w, d| core::on_sign_in(d, &account, &org, "sign-in", &mut |r| w.report(r)));
                }
            }
        }
    }
}

fn account_name(a: &Account) -> String {
    if a.alias.is_empty() {
        core::short(&a.account).to_string()
    } else {
        format!("{} ({})", a.alias, core::short(&a.account))
    }
}

// ---------- the menu ----------

struct Ui {
    strings: Strings,
    tray: Option<TrayIcon>,
    icon_on: Icon,
    icon_off: Icon,
    auto_sync: bool,
    menu: Menu,
    header: MenuItem,
    watch: CheckMenuItem,
    move_to: Submenu,
    move_items: Vec<(CheckMenuItem, Account)>,
    sync_all: MenuItem,
    /// true: "move now" is in the menu (copies are deleted); false: "sync all accounts"
    move_shown: bool,
    delete: CheckMenuItem,
    speed: Submenu,
    speeds: Vec<(CheckMenuItem, core::Speed)>,
    estimate: MenuItem,
    autostart: CheckMenuItem,
    lang: Submenu,
    langs: Vec<(CheckMenuItem, String)>,
    log: MenuItem,
    alias: MenuItem,
    quit: MenuItem,
}

fn make_icon(color: [u8; 3]) -> Icon {
    let size = 32;
    Icon::from_rgba(icon::draw(size, color), size, size).expect("icon")
}

impl Ui {
    fn new() -> Ui {
        let settings = core::load_settings();
        let strings = Strings::load(&settings.language);
        let menu = Menu::new();
        let header = MenuItem::new("", false, None);
        let watch = CheckMenuItem::new("", true, true, None);
        let move_to = Submenu::new("", true);
        let sync_all = MenuItem::new("", true, None);
        let move_shown = !settings.keep_copies;
        let move_or_sync: &dyn IsMenuItem = if move_shown { &move_to } else { &sync_all };
        let delete = CheckMenuItem::new("", true, !settings.keep_copies, None);
        let speed = Submenu::new("", true);
        let speeds: Vec<(CheckMenuItem, core::Speed)> =
            core::Speed::ALL.iter().map(|&s| (CheckMenuItem::new("", true, settings.speed == s, None), s)).collect();
        for (item, _) in &speeds {
            let _ = speed.append(item);
        }
        let estimate = MenuItem::new("", false, None);
        let autostart = CheckMenuItem::new("", true, platform::autostart_enabled(), None);
        // "Language" is always written in English, and each language in its own name,
        // so someone who cannot read the current language can still find their own.
        let lang = Submenu::new("Language", true);
        let log = MenuItem::new("", true, None);
        let alias = MenuItem::new("", true, None);
        let quit = MenuItem::new("", true, None);
        let sep = PredefinedMenuItem::separator();
        let items: [&dyn IsMenuItem; 16] = [
            &header, &sep, &watch, move_or_sync, &sep, &delete, &speed, &estimate, &autostart, &sep, &lang, &sep, &log, &alias, &sep, &quit,
        ];
        for it in items {
            let _ = menu.append(it);
        }
        let mut ui = Ui {
            strings,
            tray: None,
            icon_on: make_icon(icon::ORANGE),
            icon_off: make_icon(icon::GREY),
            auto_sync: true,
            menu,
            header,
            watch,
            move_to,
            move_items: Vec::new(),
            sync_all,
            move_shown,
            delete,
            speed,
            speeds,
            estimate,
            autostart,
            lang,
            langs: Vec::new(),
            log,
            alias,
            quit,
        };
        ui.build_language_menu();
        ui.set_texts();
        ui
    }

    fn t(&self, key: &str, args: &[&dyn std::fmt::Display]) -> String {
        self.strings.t(key, args)
    }

    fn create_tray(&mut self) {
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(self.menu.clone()))
            .with_menu_on_left_click(true)
            .with_icon(self.icon_on.clone())
            .with_tooltip(self.t("tooltip_on", &[]))
            .build();
        match tray {
            Ok(t) => self.tray = Some(t),
            Err(e) => core::log(&format!("tray icon failed: {e}")),
        }
        self.update_state();
    }

    fn build_language_menu(&mut self) {
        for (item, _) in self.langs.drain(..) {
            let _ = self.lang.remove(&item);
        }
        while self.lang.remove_at(0).is_some() {}
        let current = core::load_settings().language;
        let auto = CheckMenuItem::new("Auto (system language)", true, current.is_empty(), None);
        let _ = self.lang.append(&auto);
        let _ = self.lang.append(&PredefinedMenuItem::separator());
        self.langs.push((auto, String::new()));
        for (code, name) in &self.strings.languages {
            let item = CheckMenuItem::new(name, true, !current.is_empty() && *code == self.strings.lang, None);
            let _ = self.lang.append(&item);
            self.langs.push((item, code.clone()));
        }
    }

    /// (Re)labels everything in the current language; "Language" itself stays English.
    fn set_texts(&mut self) {
        self.watch.set_text(self.t("menu_auto", &[]));
        self.move_to.set_text(self.t("menu_move", &[]));
        self.sync_all.set_text(self.t("menu_sync_all", &[]));
        self.delete.set_text(self.t("menu_delete_copies", &[]));
        self.speed.set_text(self.t("menu_speed", &[]));
        for (item, s) in &self.speeds {
            item.set_text(self.strings.t(&format!("speed_{}", s.key()), &[]));
        }
        self.autostart.set_text(self.t("menu_autostart", &[]));
        self.log.set_text(self.t("menu_log", &[]));
        self.alias.set_text(self.t("menu_alias", &[]));
        self.quit.set_text(self.t("menu_quit", &[]));
        self.estimate.set_text("…");
        self.update_state();
    }

    fn update_state(&self) {
        let on = self.auto_sync;
        if let Some(t) = &self.tray {
            let _ = t.set_icon(Some(if on { self.icon_on.clone() } else { self.icon_off.clone() }));
            let mut tip = self.t(if on { "tooltip_on" } else { "tooltip_off" }, &[]);
            if tip.chars().count() > 63 {
                tip = tip.chars().take(62).collect::<String>() + "…"; // Windows limit
            }
            let _ = t.set_tooltip(Some(tip));
        }
        self.watch.set_checked(on);
        self.header.set_text(self.t(if on { "header_on" } else { "header_off" }, &[]));
    }

    fn apply_info(&mut self, accounts: Result<Vec<Account>, String>, estimate: Option<Estimate>) {
        let settings = core::load_settings();
        self.delete.set_checked(!settings.keep_copies);
        self.show_move(!settings.keep_copies);
        for (item, s) in &self.speeds {
            item.set_checked(*s == settings.speed);
        }
        self.autostart.set_checked(platform::autostart_enabled());
        match estimate {
            Some(e) => {
                let text = if e.keep_copies {
                    self.t("est_keep", &[&e.copy_seconds])
                } else {
                    self.t("est_delete", &[&e.copy_seconds, &e.cleanup_seconds])
                };
                self.estimate.set_text(text);
            }
            None => self.estimate.set_text("—"),
        }
        for (item, _) in self.move_items.drain(..) {
            let _ = self.move_to.remove(&item);
        }
        while self.move_to.remove_at(0).is_some() {}
        match accounts {
            Ok(list) => {
                for a in list {
                    let mut label = self.t("account_label", &[&account_name(&a), &a.sessions]);
                    if a.signed_in {
                        label += &format!("  {}", self.t("signed_in_tag", &[]));
                    }
                    let item = CheckMenuItem::new(label, true, false, None);
                    let _ = self.move_to.append(&item);
                    self.move_items.push((item, a));
                }
            }
            Err(_) => {
                let _ = self.move_to.append(&MenuItem::new(self.t("accounts_error", &[]), false, None));
            }
        }
    }

    /// Puts "move now" (copies deleted after sign-in) or "sync all accounts"
    /// (copies kept) in the menu, right below auto sync.
    fn show_move(&mut self, show: bool) {
        if show == self.move_shown {
            return;
        }
        let (old, new): (&dyn IsMenuItem, &dyn IsMenuItem) =
            if show { (&self.sync_all, &self.move_to) } else { (&self.move_to, &self.sync_all) };
        let _ = self.menu.remove(old);
        let _ = self.menu.insert(new, MOVE_POS);
        self.move_shown = show;
    }

    fn save(&self, key: &str, value: Value) -> bool {
        match core::save_setting(key, value) {
            Ok(()) => true,
            Err(_) => {
                notify(&self.t("settings_unreadable", &[]));
                false
            }
        }
    }
}

fn dialog(title: &str, text: &str, yes_no: bool) -> bool {
    let d = rfd::MessageDialog::new().set_title(title).set_description(text).set_level(rfd::MessageLevel::Warning);
    let d = if yes_no { d.set_buttons(rfd::MessageButtons::YesNo) } else { d.set_buttons(rfd::MessageButtons::Ok) };
    matches!(d.show(), rfd::MessageDialogResult::Yes | rfd::MessageDialogResult::Ok)
}

// ---------- start ----------

pub fn run() {
    let settings = core::load_settings();
    let Some(_lock) = platform::single_instance() else {
        let s = Strings::load(&settings.language);
        dialog("CCDC", &s.t("already_running", &[]), false);
        return;
    };
    #[cfg(windows)]
    platform::migrate_legacy_install();
    // macOS / Linux: start with the computer unless turned off (Windows: the installer sets it)
    #[cfg(not(windows))]
    if settings.start_at_login.is_none() {
        let _ = platform::set_autostart(true);
        let _ = core::save_setting("startAtLogin", json!(true));
    }
    if settings.start_at_login == Some(false) && platform::autostart_enabled() {
        let _ = platform::set_autostart(false); // an update may have turned it back on
    }
    #[cfg(target_os = "macos")]
    let _ = notify_rust::set_application(crate::BUNDLE_ID);

    let mut event_loop = EventLoopBuilder::<UiEvent>::with_user_event().build();
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        event_loop.set_activation_policy(ActivationPolicy::Accessory); // menu bar only, no Dock icon
    }
    let _ = &mut event_loop;
    let proxy = event_loop.create_proxy();
    let p = proxy.clone();
    MenuEvent::set_event_handler(Some(Arc::new(move |e: MenuEvent| {
        let _ = p.send_event(UiEvent::Menu(e));
    })));
    let p = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        // the pointer is over the icon: refresh accounts and the estimate before the menu opens
        if matches!(e, TrayIconEvent::Enter { .. } | TrayIconEvent::Click { .. }) {
            let _ = p.send_event(UiEvent::Tray);
        }
    }));

    let (tx, rx): (Sender<Cmd>, Receiver<Cmd>) = mpsc::channel();
    let busy = Arc::new(AtomicBool::new(false));
    let worker = Worker {
        proxy: proxy.clone(),
        busy: busy.clone(),
        data_dir: None,
        app_log: None,
        follower: None,
        auto_sync: true,
        signed_in: None,
        signin_due: None,
        last_error: String::new(),
    };
    let handle = std::thread::spawn(move || worker.run(rx));
    let mut handle = Some(handle);

    let mut ui = Ui::new();
    let mut last_refresh = Instant::now() - Duration::from_secs(3600);

    event_loop.run(move |event, _, control_flow| {
        // Linux trays send no hover events: refresh now and then instead
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_secs(300));
        match event {
            Event::NewEvents(StartCause::Init) => {
                ui.create_tray();
                notify(&ui.t("started", &[]));
                let _ = tx.send(Cmd::Refresh);
                last_refresh = Instant::now();
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                let _ = tx.send(Cmd::Refresh);
                last_refresh = Instant::now();
            }
            Event::UserEvent(UiEvent::Tray) => {
                if last_refresh.elapsed() > Duration::from_secs(2) {
                    let _ = tx.send(Cmd::Refresh);
                    last_refresh = Instant::now();
                }
            }
            Event::UserEvent(UiEvent::Info { accounts, estimate }) => ui.apply_info(accounts, estimate),
            Event::UserEvent(UiEvent::WorkerDone) => {
                let _ = tx.send(Cmd::Refresh);
            }
            Event::UserEvent(UiEvent::Notify(text)) => notify(&text),
            Event::UserEvent(UiEvent::MoveDone(text, _ok)) => notify(&text),
            Event::UserEvent(UiEvent::LowDisk { need, free }) => {
                dialog(&ui.t("lowdisk_title", &[]), &ui.t("lowdisk_body", &[&core::mb(need), &core::mb(free)]), false);
            }
            Event::UserEvent(UiEvent::Menu(e)) => {
                let id = e.id;
                if id == *ui.watch.id() {
                    ui.auto_sync = !ui.auto_sync;
                    let _ = tx.send(Cmd::AutoSync(ui.auto_sync));
                    ui.update_state();
                } else if id == *ui.delete.id() {
                    let keep = core::load_settings().keep_copies;
                    if ui.save("keepCopies", json!(!keep)) {
                        ui.show_move(keep); // copies now deleted -> "move now"; kept -> "sync all"
                    }
                    let _ = tx.send(Cmd::Refresh);
                } else if let Some((_, s)) = ui.speeds.iter().find(|(i, _)| *i.id() == id) {
                    let s = *s;
                    ui.save("speed", json!(s.key()));
                    let _ = tx.send(Cmd::Refresh);
                } else if id == *ui.autostart.id() {
                    let on = !platform::autostart_enabled();
                    if let Err(err) = platform::set_autostart(on) {
                        core::log(&format!("start with the computer: {err}"));
                    }
                    ui.save("startAtLogin", json!(on));
                    ui.autostart.set_checked(platform::autostart_enabled());
                } else if let Some((_, code)) = ui.langs.iter().find(|(i, _)| *i.id() == id) {
                    let code = code.clone();
                    if ui.save("language", json!(code)) {
                        ui.strings = Strings::load(&code);
                        ui.build_language_menu();
                        ui.set_texts();
                        let _ = tx.send(Cmd::Refresh);
                    }
                } else if let Some((item, acc)) = ui.move_items.iter().find(|(i, _)| *i.id() == id) {
                    item.set_checked(false);
                    let acc = acc.clone();
                    if acc.signed_in || dialog("CCDC", &ui.t("move_confirm", &[]), true) {
                        let _ = tx.send(Cmd::Move(acc));
                    }
                } else if id == *ui.sync_all.id() {
                    let _ = tx.send(Cmd::SyncAll);
                } else if id == *ui.log.id() {
                    platform::open_text_file(&platform::state_dir().join("watch.log"));
                } else if id == *ui.alias.id() {
                    // fill in the accounts on first use, so there is something to name
                    let s = core::load_settings();
                    if s.aliases.is_empty() {
                        if let Ok(d) = core::resolve_data_dir(None) {
                            if let Ok(list) = core::accounts(&d, None, &s) {
                                let m: serde_json::Map<String, Value> =
                                    list.iter().map(|a| (core::short(&a.account).to_string(), json!(""))).collect();
                                ui.save("aliases", Value::Object(m));
                            }
                        }
                    }
                    platform::open_text_file(&core::settings_file());
                } else if id == *ui.quit.id() {
                    // let a running copy finish (up to 30 s)
                    let t0 = Instant::now();
                    while busy.load(Ordering::SeqCst) && t0.elapsed() < Duration::from_secs(30) {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    let _ = tx.send(Cmd::Quit);
                    if let Some(h) = handle.take() {
                        let _ = h.join();
                    }
                    ui.tray = None;
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

// Windows only: macOS menus must be made on the main thread, which tests do not run on.
#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn move_now_and_sync_all_take_turns() {
        let mut ui = Ui::new();
        let ids = |ui: &Ui| ui.menu.items().iter().map(|i| i.id().clone()).collect::<Vec<_>>();
        for show in [true, false, true, false, false] {
            ui.show_move(show);
            let ids = ids(&ui);
            let (shown, hidden) = if show { (ui.move_to.id(), ui.sync_all.id()) } else { (ui.sync_all.id(), ui.move_to.id()) };
            assert_eq!(&ids[MOVE_POS], shown);
            assert!(!ids.contains(hidden));
            assert_eq!(ids.len(), 16);
        }
    }
}
