// CCDC (Continuous Claude Desktop Code): lets every account in the Claude
// desktop app see the same local Code sessions. Started without arguments it
// is the tray app; with a command it is a command-line tool.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod core;
mod i18n;
mod icon;
mod platform;
mod tray;

use std::path::PathBuf;
use std::process::ExitCode;

/// AppUserModelID on the Windows Start menu shortcut (needed for notifications).
#[allow(dead_code)]
pub const AUMID: &str = "CCDC.ContinuousClaudeDesktopCode";
/// macOS bundle identifier.
#[allow(dead_code)]
pub const BUNDLE_ID: &str = "io.github.lend-a-villin.ccdc";

#[derive(Default)]
struct Args {
    cmd: Option<String>,
    dry_run: bool,
    json: bool,
    data_dir: Option<String>,
    app_log: Option<String>,
    to: Option<String>,
    help: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "--dry-run" => a.dry_run = true,
            "--json" => a.json = true,
            "--data-dir" => a.data_dir = Some(value("--data-dir")?),
            "--app-log" => a.app_log = Some(value("--app-log")?),
            "--to" => a.to = Some(value("--to")?),
            "-h" | "--help" => a.help = true,
            "-V" | "--version" => a.cmd = Some("version".into()),
            s if s.starts_with("-psn_") => {} // macOS Finder adds this on old systems
            s if a.cmd.is_none() => a.cmd = Some(s.to_string()),
            s => return Err(format!("unexpected argument: {s}")),
        }
    }
    Ok(a)
}

const HELP: &str = "CCDC (Continuous Claude Desktop Code) - share Claude desktop Code sessions between accounts

  ccdc                          start the tray app
  ccdc status                   app state, settings, folders and session counts
  ccdc watch                    follow the app without a tray: copy on sign-out, clean up after sign-in
  ccdc estimate [--json]        how long the next sign-out copy / sign-in cleanup would take
  ccdc replicate [--dry-run]    copy the newest sessions into every account folder now
  ccdc move [--to <acc>[/<org>]] [--dry-run]
                                keep one set of sessions, in that account's folder
  ccdc version

Options:
  --data-dir <path>   the Claude app's data folder (the one holding claude-code-sessions)
  --app-log <path>    the Claude app's main.log

Settings are in ccdc.json in CCDC's folder (see `ccdc status`).";

fn data_dir(a: &Args) -> Result<PathBuf, String> {
    core::resolve_data_dir(a.data_dir.as_deref())
}

fn cmd_status(a: &Args) -> Result<(), String> {
    let d = data_dir(a)?;
    let log = core::resolve_app_log(&d, a.app_log.as_deref());
    let st = core::read_app_state(log.as_deref());
    let s = core::scan(&d)?;
    let settings = core::load_settings();
    println!("Data folder: {}", d.display());
    println!("App log: {}", log.as_ref().map(|p| p.display().to_string()).unwrap_or("not found".into()));
    println!("CCDC folder: {}", platform::state_dir().display());
    println!(
        "App state: {}",
        match &st {
            None => "unknown".to_string(),
            Some(core::LogEvent::SignedOut) => "signed out".to_string(),
            Some(core::LogEvent::SignedIn { account, org }) => format!("signed in {} / {}", core::short(account), core::short(org)),
        }
    );
    println!(
        "Settings: speed {}, {}",
        settings.speed.key(),
        if settings.keep_copies { "keep copies" } else { "remove copies after sign-in" }
    );
    for l in &s.leaves {
        let live = matches!(&st, Some(core::LogEvent::SignedIn { account, org }) if *account == l.account && *org == l.org);
        println!(
            "  {} / {}  sessions={}{}",
            core::short(&l.account),
            core::short(&l.org),
            l.sessions.len(),
            if live { "  <- in use" } else { "" }
        );
    }
    println!("Free disk: {}", core::mb(platform::free_bytes(&d.join(core::SESSIONS_DIR))));
    Ok(())
}

fn cmd_estimate(a: &Args) -> Result<(), String> {
    let e = core::estimate(&data_dir(a)?, &core::load_settings())?;
    if a.json {
        println!(
            "{}",
            serde_json::json!({
                "keepCopies": e.keep_copies, "speed": e.speed.key(), "copyFiles": e.copy_files, "copyMB": e.copy_mb,
                "purgeFiles": e.purge_files, "cleanupFiles": e.cleanup_files,
                "copySeconds": e.copy_seconds, "cleanupSeconds": e.cleanup_seconds,
            })
        );
    } else {
        println!("Speed {}, {}.", e.speed.key(), if e.keep_copies { "copies kept" } else { "copies removed after sign-in" });
        println!("Next sign-out: copy {} file(s), {}MB, about {}s.", e.copy_files, e.copy_mb, e.copy_seconds);
        if !e.keep_copies {
            println!("Next sign-in: remove {} copies, about {}s.", e.cleanup_files, e.cleanup_seconds);
        }
    }
    Ok(())
}

fn cmd_replicate(a: &Args) -> Result<ExitCode, String> {
    let r = core::replicate(&data_dir(a)?, a.dry_run, &core::load_settings(), &mut |_, _, _| {})?;
    if r.low_disk {
        println!("LOW_DISK: copying needs {}, only {} free. Nothing written.", core::mb(r.need), core::mb(r.free));
        return Ok(ExitCode::from(3));
    }
    if a.dry_run {
        println!(
            "Would write {} file(s), {}, remove {} deleted session copies (free {}). Nothing written.",
            r.copies,
            core::mb(r.need),
            r.purge,
            core::mb(r.free)
        );
    } else {
        println!("Wrote {} file(s), {}; removed {} deleted session copies.", r.written, core::mb(r.need), r.purged);
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_move(a: &Args) -> Result<(), String> {
    let d = data_dir(a)?;
    let log = core::resolve_app_log(&d, a.app_log.as_deref());
    let to = a.to.clone().unwrap_or_default().to_lowercase();
    let mut parts = to.splitn(2, '/');
    let mut account = parts.next().unwrap_or("").to_string();
    let mut org = parts.next().map(str::to_string);
    if account.is_empty() {
        match core::read_app_state(log.as_deref()) {
            Some(core::LogEvent::SignedIn { account: acc, org: o }) => {
                account = acc;
                org = Some(o);
            }
            _ => account = core::signed_in_account(&d, log.as_deref()).unwrap_or_default(),
        }
    }
    if account.is_empty() {
        return Err("No signed-in account found. Pass --to <account>.".into());
    }
    // allow short prefixes as shown by `status`
    let s = core::scan(&d)?;
    if let Some(l) = s.leaves.iter().find(|l| l.account.starts_with(&account)) {
        account = l.account.clone();
    }
    if let Some(o) = &org {
        if let Some(l) = s.leaves.iter().find(|l| l.account == account && l.org.starts_with(o.as_str())) {
            org = Some(l.org.clone());
        }
    }
    let r = core::consolidate(&d, &account, org.as_deref(), a.dry_run, false, &core::load_settings())?;
    if !r.found {
        return Err(format!(
            "Account {} has no session folder yet. Open the Code tab once while signed in to it.",
            core::short(&account)
        ));
    }
    let count = |k: core::OpKind| r.planned.iter().filter(|x| **x == k).count();
    println!("Target: {} / {}", core::short(&r.account), core::short(&r.org));
    if a.dry_run {
        println!(
            "Would move {}, remove {} copies, replace {} with newer. Nothing written.",
            count(core::OpKind::Move),
            count(core::OpKind::Remove),
            count(core::OpKind::Replace)
        );
    } else if r.planned.is_empty() {
        println!("Nothing to move.");
    } else {
        let mut msg = format!("Moved {} file(s), removed {} copies", r.moved, r.removed);
        if r.replaced > 0 {
            msg += &format!(
                ", replaced {} with newer (old ones in {})",
                r.replaced,
                r.backup_root.map(|p| p.display().to_string()).unwrap_or_default()
            );
        }
        println!("{msg}.");
    }
    Ok(())
}

/// Follows the app without a tray (command-line use).
fn cmd_watch(a: &Args) -> Result<(), String> {
    let d = data_dir(a)?;
    let log = core::resolve_app_log(&d, a.app_log.as_deref()).ok_or("Claude app log (main.log) not found. Pass --app-log.")?;
    core::ECHO_STDOUT.store(true, std::sync::atomic::Ordering::Relaxed);
    core::log(&format!("Watching {}", d.display()));
    core::log(&format!("Following app log {}", log.display()));
    let mut signed_in = match core::read_app_state(Some(&log)) {
        Some(core::LogEvent::SignedIn { account, org }) => Some((account, org)),
        _ => None,
    };
    let mut follower = core::LogFollower::new(log.clone());
    core::on_startup(&d, Some(&log), &mut |_| {});
    let settle = std::time::Duration::from_secs(6);
    let mut due: Option<(std::time::Instant, String, String)> = None;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        for ev in follower.poll() {
            match ev {
                core::LogEvent::SignedOut => {
                    signed_in = None;
                    due = None;
                    core::on_sign_out(&d, "Sign-out detected", &mut |_| {});
                }
                core::LogEvent::SignedIn { account, org } => {
                    if signed_in.as_ref() != Some(&(account.clone(), org.clone())) {
                        let keep = core::load_settings().keep_copies;
                        core::log(&format!(
                            "Sign-in detected: {}/{}, {}",
                            core::short(&account),
                            core::short(&org),
                            if keep { "copies kept".to_string() } else { format!("removing copies in {}s", settle.as_secs()) }
                        ));
                        signed_in = Some((account.clone(), org.clone()));
                    }
                    due = Some((std::time::Instant::now() + settle, account, org));
                }
            }
        }
        if due.as_ref().is_some_and(|(t, _, _)| std::time::Instant::now() >= *t) {
            let (_, account, org) = due.take().unwrap();
            core::on_sign_in(&d, &account, &org, "sign-in", &mut |_| {});
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            platform::attach_console();
            eprintln!("Error: {e}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    if args.cmd.is_none() && !args.help {
        tray::run();
        return ExitCode::SUCCESS;
    }
    platform::attach_console();
    let result = match args.cmd.as_deref() {
        _ if args.help => {
            println!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
        Some("status") => cmd_status(&args).map(|_| ExitCode::SUCCESS),
        Some("estimate") => cmd_estimate(&args).map(|_| ExitCode::SUCCESS),
        Some("replicate") => cmd_replicate(&args),
        Some("move") => cmd_move(&args).map(|_| ExitCode::SUCCESS),
        Some("watch") => cmd_watch(&args).map(|_| ExitCode::SUCCESS),
        Some("version") => {
            println!("ccdc {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Some(other) => Err(format!("unknown command: {other}\n\n{HELP}")),
        None => Ok(ExitCode::SUCCESS),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
