// Everything that differs between Windows, macOS and Linux.

use std::path::{Path, PathBuf};

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from)
}

#[cfg(not(windows))]
fn home() -> PathBuf {
    env_path("HOME").unwrap_or_else(|| PathBuf::from("/"))
}

// ---------- folders ----------

/// CCDC's own folder: settings (ccdc.json), watch.log, state.json.
pub fn state_dir() -> PathBuf {
    #[cfg(windows)]
    let d = env_path("LOCALAPPDATA").unwrap_or_else(std::env::temp_dir).join("CCDC");
    #[cfg(target_os = "macos")]
    let d = home().join("Library/Application Support/CCDC");
    #[cfg(all(unix, not(target_os = "macos")))]
    let d = env_path("XDG_CONFIG_HOME").unwrap_or_else(|| home().join(".config")).join("ccdc");
    let _ = std::fs::create_dir_all(&d);
    d
}

/// Where the Claude desktop app may keep its data (the folder holding claude-code-sessions).
pub fn data_dir_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    #[cfg(windows)]
    {
        // MSIX install: %LOCALAPPDATA%\Claude-Data\<package>; classic install: %APPDATA%\Claude
        if let Some(local) = env_path("LOCALAPPDATA") {
            if let Ok(rd) = std::fs::read_dir(local.join("Claude-Data")) {
                let mut dirs: Vec<PathBuf> = rd.flatten().filter(|e| e.path().is_dir()).map(|e| e.path()).collect();
                dirs.sort();
                v.extend(dirs);
            }
        }
        if let Some(roaming) = env_path("APPDATA") {
            v.push(roaming.join("Claude"));
            v.push(roaming.join("Claude-3p"));
        }
    }
    // macOS / Linux: Electron's default <app data>/Claude. Organizations that run Claude
    // through their own cloud provider ("3p" mode) get a "-3p" suffix (seen in the macOS app).
    #[cfg(target_os = "macos")]
    for name in ["Claude", "Claude-3p"] {
        v.push(home().join("Library/Application Support").join(name));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let config = env_path("XDG_CONFIG_HOME").unwrap_or_else(|| home().join(".config"));
        for name in ["Claude", "Claude-3p", "claude-desktop"] {
            v.push(config.join(name));
        }
    }
    v
}

/// Where the app's main.log may be; the newest existing one is used.
pub fn app_log_candidates(data_dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    #[cfg(windows)]
    if let Some(local) = env_path("LOCALAPPDATA") {
        v.push(local.join("Claude").join("Logs").join("main.log"));
    }
    #[cfg(target_os = "macos")]
    {
        // ~/Library/Logs/<data folder name>, e.g. Logs/Claude or Logs/Claude-3p
        let name = data_dir.file_name().map(|n| n.to_os_string()).unwrap_or_else(|| "Claude".into());
        v.push(home().join("Library/Logs").join(name).join("main.log"));
    }
    // Linux (and older builds): <data folder>/logs
    v.push(data_dir.join("logs").join("main.log"));
    v.push(data_dir.join("Logs").join("main.log"));
    v
}

// ---------- disk ----------

pub fn free_bytes(dir: &Path) -> u64 {
    if let Ok(mb) = std::env::var("CCDC_TEST_FREE_MB") {
        if let Ok(n) = mb.parse::<u64>() {
            return n * 1024 * 1024;
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut free = 0u64;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, std::ptr::null_mut(), std::ptr::null_mut())
        };
        if ok != 0 {
            free
        } else {
            u64::MAX
        }
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::ffi::OsStrExt;
        let c = std::ffi::CString::new(dir.as_os_str().as_bytes()).unwrap_or_default();
        let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::statvfs(c.as_ptr(), &mut st) } == 0 {
            st.f_bavail as u64 * st.f_frsize as u64
        } else {
            u64::MAX
        }
    }
}

// ---------- time (no date library needed) ----------

pub fn local_time() -> String {
    #[cfg(windows)]
    {
        let mut t = unsafe { std::mem::zeroed::<windows_sys::Win32::Foundation::SYSTEMTIME>() };
        unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t) };
        format!("{:02}:{:02}:{:02}", t.wHour, t.wMinute, t.wSecond)
    }
    #[cfg(not(windows))]
    {
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&now, &mut tm) };
        format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
    }
}

/// 2026-11-26T02-58-39-123Z style name for backup folders.
pub fn utc_stamp(secs: u64, millis: u32) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    // civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}T{:02}-{:02}-{:02}-{millis:03}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

// ---------- language ----------

/// The user's display language, e.g. "ko-KR", "zh-Hant-TW", "pt-BR".
pub fn ui_language() -> String {
    #[cfg(windows)]
    {
        // GetUserDefaultUILanguage is the display language; the locale APIs give the regional format
        use windows_sys::Win32::Globalization::{GetUserDefaultUILanguage, LCIDToLocaleName};
        let lang = unsafe { GetUserDefaultUILanguage() } as u32;
        let mut buf = [0u16; 85];
        let n = unsafe { LCIDToLocaleName(lang, buf.as_mut_ptr(), buf.len() as i32, 0) };
        if n > 1 {
            return String::from_utf16_lossy(&buf[..(n - 1) as usize]);
        }
        "en".into()
    }
    #[cfg(not(windows))]
    {
        sys_locale::get_locale().unwrap_or_else(|| "en".into()).replace('_', "-")
    }
}

// ---------- one tray icon per user ----------

pub struct InstanceLock {
    #[cfg(windows)]
    _handle: isize,
    #[cfg(not(windows))]
    _fd: i32,
}

pub fn single_instance() -> Option<InstanceLock> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows_sys::Win32::System::Threading::CreateMutexW;
        let name: Vec<u16> = "Local\\CCDC.Tray".encode_utf16().chain(Some(0)).collect();
        let h = unsafe { CreateMutexW(std::ptr::null(), 1, name.as_ptr()) };
        if h.is_null() || unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return None;
        }
        Some(InstanceLock { _handle: h as isize })
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(state_dir().join("tray.lock").as_os_str().as_bytes()).ok()?;
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600) };
        if fd < 0 || unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return None;
        }
        Some(InstanceLock { _fd: fd })
    }
}

// ---------- start at sign-in ----------

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(target_os = "macos")]
fn launch_agent() -> PathBuf {
    home().join("Library/LaunchAgents/io.github.lend-a-villin.ccdc.plist")
}

#[cfg(all(unix, not(target_os = "macos")))]
fn autostart_file() -> PathBuf {
    env_path("XDG_CONFIG_HOME").unwrap_or_else(|| home().join(".config")).join("autostart/ccdc.desktop")
}

pub fn autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey(RUN_KEY)
            .and_then(|k| k.get_value::<String, _>("CCDC"))
            .is_ok()
    }
    #[cfg(target_os = "macos")]
    {
        launch_agent().exists()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        autostart_file().exists()
    }
}

pub fn set_autostart(on: bool) -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    #[cfg(windows)]
    {
        let (key, _) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).create_subkey(RUN_KEY)?;
        if on {
            key.set_value("CCDC", &format!("\"{}\"", exe.display()))
        } else {
            match key.delete_value("CCDC") {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let path = launch_agent();
        if !on {
            return match std::fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        // launch the .app (not the bare binary) so macOS treats it as the app
        let app = exe.ancestors().find(|p| p.extension().map(|e| e == "app").unwrap_or(false)).map(Path::to_path_buf);
        let args = match app {
            Some(app) => format!("<string>/usr/bin/open</string><string>-a</string><string>{}</string>", xml(&app.display().to_string())),
            None => format!("<string>{}</string>", xml(&exe.display().to_string())),
        };
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(
            &path,
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
                 <plist version=\"1.0\"><dict>\n\
                 <key>Label</key><string>io.github.lend-a-villin.ccdc</string>\n\
                 <key>ProgramArguments</key><array>{args}</array>\n\
                 <key>RunAtLoad</key><true/>\n\
                 </dict></plist>\n"
            ),
        )
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let path = autostart_file();
        if !on {
            return match std::fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
                _ => Ok(()),
            };
        }
        std::fs::create_dir_all(path.parent().unwrap())?;
        // TryExec: once the package is removed, the desktop skips this entry
        std::fs::write(
            &path,
            format!(
                "[Desktop Entry]\nType=Application\nName=CCDC\nComment=Continuous Claude Desktop Code\n\
                 Exec={0}\nTryExec={0}\nIcon=ccdc\nX-GNOME-Autostart-enabled=true\nNoDisplay=true\n",
                exe.display()
            ),
        )
    }
}

#[cfg(target_os = "macos")]
fn xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// ---------- opening files ----------

pub fn open_text_file(path: &Path) {
    if !path.exists() {
        let _ = std::fs::write(path, "");
    }
    #[cfg(windows)]
    let r = std::process::Command::new("notepad.exe").arg(path).spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg("-t").arg(path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(path).spawn();
    if let Err(e) = r {
        crate::core::log(&format!("could not open {}: {e}", path.display()));
    }
}

// ---------- Windows: console for command-line use ----------

/// The exe has no console of its own (it is a tray app); when it is run from a
/// terminal with a command, print to that terminal.
pub fn attach_console() {
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::System::Console::AttachConsole(windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS);
    }
}

// ---------- Windows: replace the 0.3 script install ----------

/// CCDC 0.3 was PowerShell scripts installed by setup.cmd into
/// %LOCALAPPDATA%\Programs\CCDC, with its own "Installed apps" entry and
/// shortcuts that start wscript.exe. Stops it, keeps its settings, removes it.
#[cfg(windows)]
pub fn migrate_legacy_install() {
    use winreg::enums::HKEY_CURRENT_USER;
    const KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\CCDC";
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let Ok(k) = hkcu.open_subkey(KEY) else { return };
    let uninstall: String = k.get_value("UninstallString").unwrap_or_default();
    if !uninstall.contains("uninstall.ps1") {
        return; // not the script install
    }
    let dir: String = k.get_value("InstallLocation").unwrap_or_default();
    drop(k);
    let dir = PathBuf::from(dir);
    crate::core::log("Replacing the CCDC 0.3 script install");

    // settings: ccdc.json used to sit next to the scripts
    let old = dir.join("ccdc.json");
    let new = state_dir().join("ccdc.json");
    if old.is_file() && !new.exists() {
        let _ = std::fs::copy(&old, &new);
    }
    // stop the old tray, and remove its shortcuts (only those that start the old scripts)
    let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-CimInstance Win32_Process -Filter "Name='powershell.exe' OR Name='node.exe'" |
  Where-Object { $_.CommandLine -match 'Programs\\CCDC\\(tray\.ps1|ccdc\.mjs)' } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
$sh = New-Object -ComObject WScript.Shell
foreach ($f in [Environment]::GetFolderPath('Desktop'), [Environment]::GetFolderPath('Programs'), [Environment]::GetFolderPath('Startup')) {
  $l = Join-Path $f 'Continuous Claude Desktop Code.lnk'
  if ((Test-Path $l) -and ($sh.CreateShortcut($l).TargetPath -match 'wscript\.exe$')) { Remove-Item $l -Force }
}
"#;
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", ps])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .status();
    for f in [
        "ccdc.mjs", "tray.ps1", "ccdc-tray.vbs", "i18n.ps1", "locales.csv", "uninstall.ps1", "common.ps1", "CCDC.ico",
        "README.md", "README.ko.md", "LICENSE", "ccdc.json",
    ] {
        let _ = std::fs::remove_file(dir.join(f));
    }
    let _ = hkcu.delete_subkey_all(KEY);
}

#[cfg(test)]
mod tests {
    #[test]
    fn stamps() {
        assert_eq!(super::utc_stamp(0, 0), "1970-01-01T00-00-00-000Z");
        assert_eq!(super::utc_stamp(1_795_651_200, 5), "2026-11-26T00-00-00-005Z");
    }
}
