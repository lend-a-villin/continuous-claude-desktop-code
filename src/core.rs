// What CCDC does to the Claude desktop app's session folders.
//
// The desktop app stores Code session metadata per account/org:
//   <data>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
// The transcripts live in ~/.claude/projects and are shared by every account.
//
// The app reads that folder only when it starts and when someone signs in, so:
//   - on sign-out, the newest copy of every session is copied into every
//     account/org folder (only files that changed), so whichever account signs
//     in next finds them all;
//   - once the app has signed in and read its folder, the copies in the other
//     folders are kept by default ("keepCopies", fewer SSD writes) or removed
//     again when "keepCopies" is false.
// Sign-out and sign-in are read from the app's own log (main.log).

use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::platform;

pub const SESSIONS_DIR: &str = "claude-code-sessions";
const DISK_RESERVE: u64 = 512 * 1024 * 1024; // keep this much free after copying
const HEAD_BYTES: usize = 4096; // lastActivityAt sits near the start of a session file
const MB: f64 = 1024.0 * 1024.0;

// assumed disk rates until CCDC has measured this computer
const DEFAULT_COPY_MBPS: f64 = 150.0;
const DEFAULT_DELETES_PER_SEC: f64 = 1500.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Speed {
    Fast,
    Normal,
    Gentle,
}

impl Speed {
    pub const ALL: [Speed; 3] = [Speed::Fast, Speed::Normal, Speed::Gentle];
    pub fn key(self) -> &'static str {
        match self {
            Speed::Fast => "fast",
            Speed::Normal => "normal",
            Speed::Gentle => "gentle",
        }
    }
    fn from_key(s: &str) -> Option<Speed> {
        Speed::ALL.into_iter().find(|v| v.key() == s)
    }
    /// copy MB/s and deletes/s (infinite = no limit)
    fn limits(self) -> (f64, f64) {
        match self {
            Speed::Fast => (f64::INFINITY, f64::INFINITY),
            Speed::Normal => (60.0, 500.0),
            Speed::Gentle => (15.0, 100.0),
        }
    }
}

// ---------- log (watch.log, always English) ----------

pub static ECHO_STDOUT: AtomicBool = AtomicBool::new(false);

pub fn log(msg: &str) {
    let line = format!("[{}] {}", platform::local_time(), msg);
    if ECHO_STDOUT.load(Ordering::Relaxed) {
        println!("{line}");
    }
    let path = platform::state_dir().join("watch.log");
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{line}");
    }
}

pub fn short(s: &str) -> &str {
    &s[..s.len().min(8)]
}

pub fn mb(n: u64) -> String {
    format!("{}MB", (n as f64 / MB).round() as u64)
}

// ---------- settings (ccdc.json) and measured rates (state.json) ----------

pub fn settings_file() -> PathBuf {
    platform::state_dir().join("ccdc.json")
}

fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub keep_copies: bool,
    pub speed: Speed,
    pub language: String,
    pub aliases: Vec<(String, String)>,
    pub start_at_login: Option<bool>,
    pub data_dir: Option<String>,
    pub app_log: Option<String>,
}

pub fn load_settings() -> Settings {
    let j = read_json(&settings_file()).unwrap_or(Value::Null);
    let s = |k: &str| j.get(k).and_then(Value::as_str).map(str::to_string);
    Settings {
        // copies are kept unless turned off: deleting them means rewriting every session at each sign-out
        keep_copies: j.get("keepCopies").and_then(Value::as_bool) != Some(false),
        speed: s("speed").and_then(|v| Speed::from_key(&v)).unwrap_or(Speed::Fast),
        language: s("language").unwrap_or_default(),
        aliases: j
            .get("aliases")
            .and_then(Value::as_object)
            .map(|m| m.iter().map(|(k, v)| (k.to_lowercase(), v.as_str().unwrap_or("").to_string())).collect())
            .unwrap_or_default(),
        start_at_login: j.get("startAtLogin").and_then(Value::as_bool),
        data_dir: s("dataDir").filter(|v| !v.is_empty()),
        app_log: s("appLog").filter(|v| !v.is_empty()),
    }
}

/// Sets one key in ccdc.json, keeping everything else (account names included).
/// Refuses to touch a file it cannot read, so a typo never wipes the account names.
pub fn save_setting(key: &str, value: Value) -> Result<(), String> {
    let path = settings_file();
    let mut obj = if path.exists() {
        match read_json(&path) {
            Some(Value::Object(m)) => m,
            _ => return Err("unreadable".into()),
        }
    } else {
        let mut m = Map::new();
        m.insert("aliases".into(), json!({}));
        m
    };
    obj.insert(key.into(), value);
    let text = serde_json::to_string_pretty(&Value::Object(obj)).unwrap();
    fs::write(&path, text).map_err(|e| e.to_string())
}

pub fn alias_for(aliases: &[(String, String)], account: &str) -> String {
    aliases
        .iter()
        .find(|(k, _)| !k.is_empty() && account.starts_with(k.as_str()))
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

fn state_file() -> PathBuf {
    platform::state_dir().join("state.json")
}

fn load_rates() -> (f64, f64) {
    let j = read_json(&state_file()).unwrap_or(Value::Null);
    let r = |k: &str, d: f64| j["rates"][k].as_f64().filter(|v| *v > 0.0).unwrap_or(d);
    (r("copyMBps", DEFAULT_COPY_MBPS), r("deletesPerSec", DEFAULT_DELETES_PER_SEC))
}

// remembers how fast this computer copied/deleted at "fast" speed, for estimates
fn save_rate(key: &str, value: f64) {
    if !(value > 0.0) || !value.is_finite() {
        return;
    }
    let mut s = match read_json(&state_file()) {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    };
    let rates = s.entry("rates").or_insert_with(|| json!({}));
    if !rates.is_object() {
        *rates = json!({});
    }
    let old = rates[key].as_f64().unwrap_or(0.0);
    let v = if old > 0.0 { old * 0.5 + value * 0.5 } else { value };
    rates[key] = json!((v * 10.0).round() / 10.0);
    let _ = fs::write(state_file(), serde_json::to_string_pretty(&Value::Object(s)).unwrap());
}

// ---------- locating the app's folders ----------

pub fn resolve_data_dir(opt: Option<&str>) -> Result<PathBuf, String> {
    let env = std::env::var("CCDC_DATA_DIR").ok().filter(|v| !v.is_empty());
    let set = load_settings().data_dir;
    if let Some(d) = opt.map(str::to_string).or(env).or(set) {
        let p = PathBuf::from(&d);
        if !p.join(SESSIONS_DIR).is_dir() {
            return Err(format!("No {SESSIONS_DIR} folder in {d}"));
        }
        return Ok(p);
    }
    let dirs: Vec<PathBuf> = platform::data_dir_candidates()
        .into_iter()
        .filter(|d| d.join(SESSIONS_DIR).is_dir())
        .collect();
    match dirs.len() {
        0 => Err("Claude desktop data folder not found. Set \"dataDir\" in ccdc.json or pass --data-dir.".into()),
        1 => Ok(dirs.into_iter().next().unwrap()),
        _ => Err(format!(
            "Several Claude data folders found; set \"dataDir\" in ccdc.json or pass --data-dir:\n  {}",
            dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join("\n  ")
        )),
    }
}

/// The app's main.log: the newest of the usual places, unless set.
pub fn resolve_app_log(data_dir: &Path, opt: Option<&str>) -> Option<PathBuf> {
    let env = std::env::var("CCDC_APP_LOG").ok().filter(|v| !v.is_empty());
    if let Some(p) = opt.map(str::to_string).or(env).or(load_settings().app_log) {
        return Some(PathBuf::from(p));
    }
    platform::app_log_candidates(data_dir)
        .into_iter()
        .filter(|p| p.is_file())
        .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH))
}

// ---------- the app log ----------

#[derive(Clone, Debug, PartialEq)]
pub enum LogEvent {
    SignedOut,
    SignedIn { account: String, org: String },
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36 && s.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
}

/// Recognises the two lines CCDC cares about (arrows and dashes matched loosely):
///   Login-state transition (loggedOut: false → true
///   [LocalSessionManager] Initialization succeeded — accountId=<uuid>, orgId=<uuid>
pub fn parse_log_line(line: &str) -> Option<LogEvent> {
    if let Some(i) = line.find("Login-state transition (loggedOut: false ") {
        let rest = &line[i + "Login-state transition (loggedOut: false ".len()..];
        let mut parts = rest.splitn(2, ' ');
        let _arrow = parts.next()?;
        if parts.next()?.starts_with("true") {
            return Some(LogEvent::SignedOut);
        }
        return None;
    }
    let i = line.find("[LocalSessionManager] Initialization succeeded ")?;
    let rest = &line[i..];
    let a = rest.find("accountId=")? + "accountId=".len();
    let o = rest.find("orgId=")? + "orgId=".len();
    let account = rest.get(a..a + 36)?;
    let org = rest.get(o..o + 36)?;
    if !is_uuid(account) || !is_uuid(org) || rest.as_bytes().get(a + 36) != Some(&b',') {
        return None;
    }
    Some(LogEvent::SignedIn { account: account.to_lowercase(), org: org.to_lowercase() })
}

/// Sign-in state as of the end of the app log. Reads backwards in chunks.
pub fn read_app_state(app_log: Option<&Path>) -> Option<LogEvent> {
    let mut f = File::open(app_log?).ok()?;
    let size = f.metadata().ok()?.len();
    const CHUNK: u64 = 64 * 1024;
    let mut end = size;
    let mut tail: Vec<u8> = Vec::new();
    while end > 0 {
        let start = end.saturating_sub(CHUNK);
        let mut buf = vec![0u8; (end - start) as usize];
        f.seek(SeekFrom::Start(start)).ok()?;
        f.read_exact(&mut buf).ok()?;
        buf.extend_from_slice(&tail);
        let text = String::from_utf8_lossy(&buf).into_owned();
        let mut lines: Vec<&str> = text.split('\n').collect();
        // the first line may be cut; finish it with the next chunk
        tail = if start > 0 { lines.remove(0).as_bytes().to_vec() } else { Vec::new() };
        for line in lines.iter().rev() {
            if let Some(ev) = parse_log_line(line) {
                return Some(ev);
            }
        }
        end = start;
    }
    None
}

pub fn signed_in_account(data_dir: &Path, app_log: Option<&Path>) -> Option<String> {
    match read_app_state(app_log) {
        Some(LogEvent::SignedIn { account, .. }) => Some(account),
        Some(LogEvent::SignedOut) => None,
        None => read_json(&data_dir.join("config.json"))?
            .get("lastKnownAccountUuid")?
            .as_str()
            .map(str::to_lowercase),
    }
}

// ---------- scanning ----------

fn session_id(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    let id = lower.strip_prefix("local_")?.strip_suffix(".json")?;
    is_uuid(id).then(|| id.to_string())
}

fn deleted_id(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    let id = lower.strip_prefix("deleted_")?;
    is_uuid(id).then(|| id.to_string())
}

#[derive(Clone, Copy, Debug)]
pub struct FileInfo {
    size: u64,
    mtime: u128, // ms
    activity: u64,
}

fn mtime_ms(m: &fs::Metadata) -> u128 {
    m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis()).unwrap_or(0)
}

/// size, mtime and lastActivityAt of a session file; only the first few KB are read
fn file_info(path: &Path) -> Option<FileInfo> {
    let mut f = File::open(path).ok()?;
    let m = f.metadata().ok()?;
    if m.len() == 0 {
        return None;
    }
    let mut buf = vec![0u8; HEAD_BYTES.min(m.len() as usize)];
    let n = f.read(&mut buf).ok()?;
    let head = String::from_utf8_lossy(&buf[..n]);
    let activity = head
        .find("\"lastActivityAt\"")
        .and_then(|i| {
            let rest = head[i + 16..].trim_start().strip_prefix(':')?.trim_start();
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        })
        .unwrap_or(0);
    Some(FileInfo { size: m.len(), mtime: mtime_ms(&m), activity })
}

// copies keep the source's modified time, so same size + same time = same file
fn same_file(a: &FileInfo, b: &FileInfo) -> bool {
    a.size == b.size && a.mtime == b.mtime
}
fn newer(a: &FileInfo, b: &FileInfo) -> bool {
    a.activity > b.activity || (a.activity == b.activity && a.mtime > b.mtime)
}

#[derive(Clone, Debug)]
pub struct Leaf {
    pub account: String,
    pub org: String,
    pub dir: PathBuf,
    pub sessions: Vec<String>,
    deleted: Vec<String>,
    last_write: u128,
    mtime: u128,
}

pub struct Scan {
    root: PathBuf,
    pub leaves: Vec<Leaf>,
}

fn subdirs(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut v: Vec<(String, PathBuf)> = fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

pub fn scan(data_dir: &Path) -> Result<Scan, String> {
    let root = data_dir.join(SESSIONS_DIR);
    if !root.is_dir() {
        return Err(format!("No {SESSIONS_DIR} folder in {}", data_dir.display()));
    }
    let mut leaves = Vec::new();
    for (acc, acc_dir) in subdirs(&root) {
        for (org, dir) in subdirs(&acc_dir) {
            let mut names: Vec<String> = fs::read_dir(&dir)
                .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
                .unwrap_or_default();
            names.sort();
            // the org in use is the one whose session files the app wrote last
            let mut last_write = 0;
            for n in &names {
                if session_id(n).is_some() || n == "waiting-input" {
                    if let Ok(m) = fs::metadata(dir.join(n)) {
                        last_write = last_write.max(mtime_ms(&m));
                    }
                }
            }
            leaves.push(Leaf {
                account: acc.to_lowercase(),
                org: org.to_lowercase(),
                sessions: names.iter().filter(|n| session_id(n).is_some()).cloned().collect(),
                deleted: names.iter().filter(|n| deleted_id(n).is_some()).cloned().collect(),
                mtime: fs::metadata(&dir).map(|m| mtime_ms(&m)).unwrap_or(0),
                last_write,
                dir,
            });
        }
    }
    Ok(Scan { root, leaves })
}

/// The account's org folder: the one given, else the one the app wrote most recently.
fn pick_target(leaves: &[Leaf], account: &str, org: Option<&str>) -> Option<usize> {
    let mine = leaves.iter().enumerate().filter(|(_, l)| l.account == account);
    if let Some(org) = org {
        return mine.into_iter().find(|(_, l)| l.org == org).map(|(i, _)| i);
    }
    mine.max_by(|(_, a), (_, b)| (a.last_write, a.mtime).cmp(&(b.last_write, b.mtime))).map(|(i, _)| i)
}

// ---------- throttling ----------

struct Throttle {
    start: Instant,
    done: f64,
    limit: f64,
}

impl Throttle {
    fn new(limit_per_sec: f64) -> Self {
        Throttle { start: Instant::now(), done: 0.0, limit: limit_per_sec }
    }
    fn step(&mut self, amount: f64) {
        self.done += amount;
        if !self.limit.is_finite() {
            return;
        }
        let ahead = self.done / self.limit - self.start.elapsed().as_secs_f64();
        if ahead > 0.005 {
            std::thread::sleep(Duration::from_secs_f64(ahead));
        }
    }
}

// ---------- replicate (on sign-out) ----------

struct Copy {
    src: PathBuf,
    dest: PathBuf,
    bytes: u64,
}

struct ReplicatePlan {
    copies: Vec<Copy>,
    purge: Vec<PathBuf>,
    sessions: usize,
}

// What a sign-out would do now: copy the newest version of each session into
// every folder that lacks it or has an older one, and remove copies of
// sessions that were deleted somewhere (so they do not come back).
fn plan_replicate(s: &Scan) -> ReplicatePlan {
    let deleted: HashSet<String> = s.leaves.iter().flat_map(|l| l.deleted.iter().filter_map(|d| deleted_id(d))).collect();
    let mut best: HashMap<String, (PathBuf, String, FileInfo)> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut infos: HashMap<PathBuf, FileInfo> = HashMap::new();
    let mut purge = Vec::new();
    for l in &s.leaves {
        for name in &l.sessions {
            let id = session_id(name).unwrap();
            let file = l.dir.join(name);
            if deleted.contains(&id) {
                purge.push(file);
                continue;
            }
            let Some(info) = file_info(&file) else { continue };
            infos.insert(file.clone(), info);
            match best.get(&id) {
                Some((_, _, cur)) if !newer(&info, cur) => {}
                prev => {
                    if prev.is_none() {
                        order.push(id.clone());
                    }
                    best.insert(id, (file, name.clone(), info));
                }
            }
        }
    }
    let mut copies = Vec::new();
    // most recently used folders first, in case sign-in comes before we finish
    let mut leaves: Vec<&Leaf> = s.leaves.iter().collect();
    leaves.sort_by(|a, b| b.last_write.cmp(&a.last_write));
    for leaf in leaves {
        let have: HashSet<String> = leaf.sessions.iter().map(|f| f.to_lowercase()).collect();
        for id in &order {
            let (src, name, info) = &best[id];
            let dest = leaf.dir.join(name);
            if &dest == src {
                continue;
            }
            if have.contains(&name.to_lowercase()) {
                if let Some(cur) = infos.get(&dest) {
                    if same_file(cur, info) || !newer(info, cur) {
                        continue;
                    }
                }
            }
            copies.push(Copy { src: src.clone(), dest, bytes: info.size });
        }
    }
    ReplicatePlan { copies, purge, sessions: best.len() }
}

#[derive(Debug, Default)]
pub struct ReplicateResult {
    pub copies: usize,
    pub purge: usize,
    pub need: u64,
    pub free: u64,
    pub written: usize,
    pub purged: usize,
    pub low_disk: bool,
    pub seconds: f64,
}

/// Copies one file so that the copy keeps the source's modified time, through a
/// temporary file so the app never sees a half-written session.
fn copy_keep_time(src: &Path, dest: &Path) -> std::io::Result<()> {
    let mut tmp = dest.as_os_str().to_owned();
    tmp.push(".ccdc.tmp");
    let tmp = PathBuf::from(tmp);
    let modified = fs::metadata(src)?.modified()?;
    fs::copy(src, &tmp)?;
    let r = File::options().write(true).open(&tmp).and_then(|f| f.set_modified(modified)).and_then(|_| fs::rename(&tmp, dest));
    if r.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    r
}

/// `on_start(copies, bytes, estimated seconds)` is called once work begins.
pub fn replicate(
    data_dir: &Path,
    dry_run: bool,
    settings: &Settings,
    on_start: &mut dyn FnMut(usize, u64, f64),
) -> Result<ReplicateResult, String> {
    let s = scan(data_dir)?;
    let plan = plan_replicate(&s);
    let need: u64 = plan.copies.iter().map(|c| c.bytes).sum();
    let free = platform::free_bytes(&s.root);
    let mut r = ReplicateResult { copies: plan.copies.len(), purge: plan.purge.len(), need, free, ..Default::default() };
    if (plan.copies.is_empty() && plan.purge.is_empty()) || dry_run {
        return Ok(r);
    }
    if need > free.saturating_sub(DISK_RESERVE) {
        r.low_disk = true;
        return Ok(r);
    }
    let (copy_lim, del_lim) = settings.speed.limits();
    on_start(r.copies, need, estimate_seconds(need, plan.purge.len(), settings.speed));

    let t0 = Instant::now();
    let mut throttle = Throttle::new(copy_lim * MB);
    for c in &plan.copies {
        match copy_keep_time(&c.src, &c.dest) {
            Ok(()) => r.written += 1,
            Err(e) => log(&format!("copy failed {}: {e}", c.dest.file_name().unwrap_or_default().to_string_lossy())),
        }
        throttle.step(c.bytes as f64);
    }
    let copy_secs = t0.elapsed().as_secs_f64();
    let mut throttle = Throttle::new(del_lim);
    for f in &plan.purge {
        if fs::remove_file(f).is_ok() {
            r.purged += 1;
        }
        throttle.step(1.0);
    }
    r.seconds = t0.elapsed().as_secs_f64();
    if settings.speed == Speed::Fast && need > 20 * 1024 * 1024 && copy_secs > 0.2 {
        save_rate("copyMBps", need as f64 / MB / copy_secs);
    }
    Ok(r)
}

// ---------- consolidate (after sign-in, and the manual "move") ----------

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OpKind {
    Move,
    Replace,
    Remove,
}

pub struct Op {
    pub kind: OpKind,
    src: PathBuf,
    dest: PathBuf,
    name: String,
}

// Leaves one set of session files, in the target folder. For a session found
// elsewhere too: a same or older copy is removed; a newer copy replaces the
// target's file, which is backed up.
fn plan_consolidate(s: &Scan, ti: usize) -> Vec<Op> {
    let target = &s.leaves[ti];
    let mut ops = Vec::new();
    let mut in_target: HashSet<String> = target.sessions.iter().map(|f| f.to_lowercase()).collect();
    let mut deleted_in_target: HashSet<String> = target.deleted.iter().map(|f| f.to_lowercase()).collect();
    for (i, leaf) in s.leaves.iter().enumerate() {
        if i == ti {
            continue;
        }
        for name in &leaf.deleted {
            let src = leaf.dir.join(name);
            let dest = target.dir.join(name);
            if deleted_in_target.insert(name.to_lowercase()) {
                ops.push(Op { kind: OpKind::Move, src, dest, name: name.clone() });
            } else {
                ops.push(Op { kind: OpKind::Remove, src, dest, name: name.clone() });
            }
        }
        for name in &leaf.sessions {
            let src = leaf.dir.join(name);
            let dest = target.dir.join(name);
            if in_target.insert(name.to_lowercase()) {
                ops.push(Op { kind: OpKind::Move, src, dest, name: name.clone() });
                continue;
            }
            let Some(a) = file_info(&src) else { continue }; // unreadable (being written?): next time
            let kind = match file_info(&dest) {
                Some(b) if !same_file(&a, &b) && newer(&a, &b) => OpKind::Replace,
                _ => OpKind::Remove,
            };
            ops.push(Op { kind, src, dest, name: name.clone() });
        }
    }
    ops
}

#[derive(Debug, Default)]
pub struct ConsolidateResult {
    pub found: bool,
    pub account: String,
    pub org: String,
    pub planned: Vec<OpKind>,
    pub moved: usize,
    pub markers: usize,
    pub removed: usize,
    pub replaced: usize,
    pub backup_root: Option<PathBuf>,
}

fn timestamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    platform::utc_stamp(d.as_secs(), d.subsec_millis())
}

pub fn consolidate(
    data_dir: &Path,
    account: &str,
    org: Option<&str>,
    dry_run: bool,
    create: bool,
    settings: &Settings,
) -> Result<ConsolidateResult, String> {
    let mut s = scan(data_dir)?;
    let mut ti = pick_target(&s.leaves, account, org);
    if ti.is_none() && create {
        if let Some(org) = org {
            fs::create_dir_all(s.root.join(account).join(org)).map_err(|e| e.to_string())?;
            s = scan(data_dir)?;
            ti = pick_target(&s.leaves, account, Some(org));
        }
    }
    let Some(ti) = ti else { return Ok(ConsolidateResult::default()) };
    let ops = plan_consolidate(&s, ti);
    let mut r = ConsolidateResult {
        found: true,
        account: s.leaves[ti].account.clone(),
        org: s.leaves[ti].org.clone(),
        planned: ops.iter().map(|o| o.kind).collect(),
        ..Default::default()
    };
    if dry_run || ops.is_empty() {
        return Ok(r);
    }
    let backup_root = data_dir.join("ccdc-backups").join(timestamp());
    let mut throttle = Throttle::new(settings.speed.limits().1);
    let t0 = Instant::now();
    for op in &ops {
        let res: std::io::Result<()> = (|| {
            match op.kind {
                OpKind::Remove => {
                    fs::remove_file(&op.src)?;
                    r.removed += 1;
                }
                OpKind::Replace | OpKind::Move => {
                    if op.kind == OpKind::Replace {
                        let rel = op.dest.strip_prefix(&s.root).unwrap_or(&op.dest);
                        let bak = backup_root.join(rel);
                        fs::create_dir_all(bak.parent().unwrap())?;
                        fs::rename(&op.dest, &bak)?;
                        r.backup_root = Some(backup_root.clone());
                        r.replaced += 1;
                    } else if deleted_id(&op.name).is_some() {
                        r.markers += 1;
                    } else {
                        r.moved += 1;
                    }
                    fs::rename(&op.src, &op.dest)?; // same volume: a rename, not a copy
                }
            }
            Ok(())
        })();
        if let Err(e) = res {
            log(&format!("skipped {}: {e}", op.name));
        }
        throttle.step(1.0);
    }
    let secs = t0.elapsed().as_secs_f64();
    if settings.speed == Speed::Fast && r.removed > 200 && secs > 0.1 {
        save_rate("deletesPerSec", r.removed as f64 / secs);
    }
    Ok(r)
}

// ---------- estimates ----------

fn estimate_seconds(copy_bytes: u64, deletes: usize, speed: Speed) -> f64 {
    let (copy_rate, del_rate) = load_rates();
    let (copy_lim, del_lim) = speed.limits();
    copy_bytes as f64 / (copy_rate.min(copy_lim) * MB) + deletes as f64 / del_rate.min(del_lim)
}

#[derive(Debug, Clone)]
pub struct Estimate {
    pub keep_copies: bool,
    pub speed: Speed,
    pub copy_files: usize,
    pub copy_mb: u64,
    pub purge_files: usize,
    pub cleanup_files: usize,
    pub copy_seconds: u64,
    pub cleanup_seconds: u64,
}

/// How long the next sign-out (copy) and sign-in (cleanup) would take right now.
pub fn estimate(data_dir: &Path, settings: &Settings) -> Result<Estimate, String> {
    let s = scan(data_dir)?;
    let plan = plan_replicate(&s);
    let copy_bytes: u64 = plan.copies.iter().map(|c| c.bytes).sum();
    // after copying, every folder holds every session; cleanup removes all but one folder's
    let cleanup = if settings.keep_copies { 0 } else { plan.sessions * s.leaves.len().saturating_sub(1) };
    let round = |x: f64| if x <= 0.0 { 0 } else { x.ceil().max(1.0) as u64 };
    Ok(Estimate {
        keep_copies: settings.keep_copies,
        speed: settings.speed,
        copy_files: plan.copies.len(),
        copy_mb: (copy_bytes as f64 / MB).round() as u64,
        purge_files: plan.purge.len(),
        cleanup_files: cleanup,
        copy_seconds: round(estimate_seconds(copy_bytes, plan.purge.len(), settings.speed)),
        cleanup_seconds: round(estimate_seconds(0, cleanup, settings.speed)),
    })
}

#[derive(Debug, Clone)]
pub struct Account {
    pub account: String,
    pub alias: String,
    pub signed_in: bool,
    pub sessions: usize,
}

pub fn accounts(data_dir: &Path, app_log: Option<&Path>, settings: &Settings) -> Result<Vec<Account>, String> {
    let s = scan(data_dir)?;
    let active = signed_in_account(data_dir, app_log);
    let mut out: Vec<Account> = Vec::new();
    for l in &s.leaves {
        if let Some(a) = out.iter_mut().find(|a| a.account == l.account) {
            a.sessions += l.sessions.len();
        } else {
            out.push(Account {
                account: l.account.clone(),
                alias: alias_for(&settings.aliases, &l.account),
                signed_in: active.as_deref() == Some(l.account.as_str()),
                sessions: l.sessions.len(),
            });
        }
    }
    Ok(out)
}

// ---------- the two jobs: what happens on sign-out and after sign-in ----------

/// What the tray shows for a job (the UI turns these into its language).
pub enum Report {
    CopyStarted { files: usize, bytes: u64, seconds: u64 },
    CopyDone { files: usize, seconds: f64 },
    NothingToCopy,
    LowDisk { need: u64, free: u64 },
    Cleaned { removed: usize, account: String },
    LateMove { moved: usize },
    Failed(String),
}

pub fn on_sign_out(data_dir: &Path, reason: &str, report: &mut dyn FnMut(Report)) {
    let settings = load_settings();
    let mut started = |files: usize, bytes: u64, secs: f64| {
        let secs = secs.ceil().max(1.0) as u64;
        log(&format!("{reason}: copying sessions into every account folder ({files} file(s), {}, about {secs}s)...", mb(bytes)));
        report(Report::CopyStarted { files, bytes, seconds: secs });
    };
    let res = replicate(data_dir, false, &settings, &mut started);
    match res {
        Ok(r) if r.low_disk => {
            log(&format!(
                "LOW_DISK: copying sessions to every account needs {}, only {} free. Skipped; sessions stay in the last account.",
                mb(r.need),
                mb(r.free)
            ));
            report(Report::LowDisk { need: r.need, free: r.free });
        }
        Ok(r) if r.written > 0 || r.purged > 0 => {
            log(&format!("Copy done: sessions are in every account folder ({} file(s), {}, {:.1}s)", r.written, mb(r.need), r.seconds));
            report(Report::CopyDone { files: r.written, seconds: r.seconds });
        }
        Ok(_) => {
            log(&format!("{reason}: nothing to copy (every folder is up to date)"));
            report(Report::NothingToCopy);
        }
        Err(e) => {
            log(&format!("replicate failed: {e}"));
            report(Report::Failed(e));
        }
    }
}

pub fn on_sign_in(data_dir: &Path, account: &str, org: &str, reason: &str, report: &mut dyn FnMut(Report)) {
    let settings = load_settings();
    if settings.keep_copies {
        return; // copies stay; the next sign-out only copies what changed
    }
    match consolidate(data_dir, account, Some(org), false, true, &settings) {
        Ok(r) => {
            if r.moved + r.markers + r.removed + r.replaced > 0 {
                let mut msg = format!("{reason}: kept sessions in {}/{}; removed {} copies elsewhere", short(account), short(org), r.removed);
                if r.moved > 0 {
                    msg += &format!(", moved in {}", r.moved);
                }
                if r.replaced > 0 {
                    msg += &format!(", {} replaced with newer", r.replaced);
                }
                log(&msg);
                report(Report::Cleaned { removed: r.removed, account: short(account).to_string() });
                if r.moved > 0 {
                    log(&format!(
                        "Note: {} session(s) moved in after the app read its list; they show after the next sign-in or app restart.",
                        r.moved
                    ));
                    report(Report::LateMove { moved: r.moved });
                }
            }
        }
        Err(e) => {
            log(&format!("cleanup failed: {e}"));
            report(Report::Failed(e));
        }
    }
}

/// Catch up with the app's current state (at start, or when auto sync is turned back on).
pub fn on_startup(data_dir: &Path, app_log: Option<&Path>, report: &mut dyn FnMut(Report)) {
    match read_app_state(app_log) {
        None => log("App state unknown: waiting for the next sign-in or sign-out"),
        Some(LogEvent::SignedOut) => {
            log("App is signed out");
            on_sign_out(data_dir, "startup", report);
        }
        Some(LogEvent::SignedIn { account, org }) => {
            log(&format!("App is signed in: {}/{}", short(&account), short(&org)));
            on_sign_in(data_dir, &account, &org, "startup", report);
        }
    }
}

// ---------- following the app log ----------

/// Reads what was appended to the app log since last time.
pub struct LogFollower {
    path: PathBuf,
    pos: u64,
    carry: String,
    buf: Vec<u8>,
}

impl LogFollower {
    /// Starts at the current end of the log.
    pub fn new(path: PathBuf) -> Self {
        let pos = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        LogFollower { path, pos, carry: String::new(), buf: vec![0; 64 * 1024] }
    }

    pub fn skip_to_end(&mut self) {
        self.pos = fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        self.carry.clear();
    }

    pub fn poll(&mut self) -> Vec<LogEvent> {
        let mut events = Vec::new();
        let Ok(len) = fs::metadata(&self.path).map(|m| m.len()) else { return events };
        if len < self.pos {
            self.pos = 0; // rotated: main.log started over
            self.carry.clear();
        }
        if len == self.pos {
            return events;
        }
        let Ok(mut f) = File::open(&self.path) else { return events };
        while self.pos < len {
            if f.seek(SeekFrom::Start(self.pos)).is_err() {
                break;
            }
            let want = self.buf.len().min((len - self.pos) as usize);
            let n = match f.read(&mut self.buf[..want]) {
                Ok(n) if n > 0 => n,
                _ => break,
            };
            self.pos += n as u64;
            let chunk = String::from_utf8_lossy(&self.buf[..n]);
            // most new lines are unrelated; skip the split unless a line we care about could be there
            if self.carry.is_empty() && !chunk.contains("Login-state transition") && !chunk.contains("[LocalSessionManager]") {
                self.carry = match chunk.rfind('\n') {
                    Some(i) => chunk[i + 1..].to_string(),
                    None => chunk.into_owned(),
                };
                continue;
            }
            let text = std::mem::take(&mut self.carry) + &chunk;
            let mut lines: Vec<&str> = text.split('\n').collect();
            let last = lines.pop().unwrap_or("");
            self.carry = if last.len() > 16 * 1024 { String::new() } else { last.to_string() };
            for line in lines {
                if let Some(ev) = parse_log_line(line) {
                    events.push(ev);
                }
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_log_lines() {
        assert_eq!(
            parse_log_line("2026-11-26 [info] [account] Login-state transition (loggedOut: false → true, reason x)"),
            Some(LogEvent::SignedOut)
        );
        assert_eq!(parse_log_line("Login-state transition (loggedOut: true → false"), None);
        let a = "7b49d3e4-0000-4000-8000-000000000001";
        let o = "d8f1fb49-0000-4000-8000-000000000002";
        let line = format!("[info] [LocalSessionManager] Initialization succeeded — accountId={a}, orgId={o}");
        assert_eq!(parse_log_line(&line), Some(LogEvent::SignedIn { account: a.into(), org: o.into() }));
        assert_eq!(parse_log_line("[LocalSessionManager] Initialization succeeded — accountId=bad, orgId=bad"), None);
    }

    #[test]
    fn session_names() {
        assert!(session_id("local_7b49d3e4-0000-4000-8000-000000000001.json").is_some());
        assert!(session_id("local_x.json").is_none());
        assert!(deleted_id("deleted_7b49d3e4-0000-4000-8000-000000000001").is_some());
    }
}
