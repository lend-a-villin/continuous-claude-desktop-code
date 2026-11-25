# Continuous Claude Desktop Code (CCDC)

**English** | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

A tray app that keeps your Claude desktop **Code** sessions when you switch Claude accounts.

The Claude desktop app keeps a separate Code session list for each account and organization. Sign in with another account and the list looks empty, even though the conversations are still on the computer. CCDC copies the sessions across, so whichever account signs in next sees all of them.

Unofficial; not affiliated with or endorsed by Anthropic.

**[Download the latest release](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · nothing else to install

| Platform | Version | Status |
| --- | --- | --- |
| Windows | 1.0 | In daily use (continues from the 0.3 scripts) |
| macOS | 0.1 | New; not yet tested with the real Claude app on a Mac |
| Linux | 0.1 | New; install-tested on Debian 12, Ubuntu 24.04 and Linux Mint 22, not yet with the real Claude app |

## Who it is for

- **Shared computers and shift work.** In a workplace that runs 24 hours in three shifts, a different person sits at the same desk (one computer, one system user) on each shift and signs in with their own Claude account. With CCDC, the next shift picks up the Code work the previous shift left off.
- **One person, several accounts.** If you switch between a personal and a work account, or between organizations, on the same computer, your Code sessions are there whichever account you sign in with.

Every Code session on the computer becomes visible to every account that signs in on it. Don't use CCDC where people must not see each other's sessions. If you use a company (organization) account, check your company's policy first: CCDC copies a work account's sessions into a personal account's folder too, and the other way round.

## The problem it solves

The desktop app keeps the **list** of Code sessions in a separate folder for each account and organization:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Claude data folder | Claude log |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (or `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Organizations that run Claude through their own cloud provider use `Claude-3p` instead of `Claude`; CCDC looks there too.)

The **conversations** themselves are in `~/.claude/projects/.../<cliSessionId>.jsonl`, which all accounts share. So after someone signs in with another account, the Code session list looks empty even though the conversations are still on disk.

## How it works

The app reads its session folder only **when it starts and when someone signs in** (twice, within about two seconds). There is no way to make it reload just the list later. The app's own log records both events:

- sign-out: `[account] Login-state transition (loggedOut: false → true ...)`
- sign-in: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC follows that log:

1. **Sign-out** → it copies the newest version of every session into every account/organization folder, so the next account to sign in already finds them in its own folder. Only sessions that are missing or out of date in a folder are copied.
2. **Sign-in** → by default the copies stay where they are. If "Delete copies after sign-in" is turned on, CCDC waits until the app has read its list (6 seconds after the last sign-in line) and then removes the copies everywhere except the signed-in account's folder.

A session deleted in the app is also removed from the other folders, so it does not come back under another account.

Before copying, CCDC checks free disk space. If there isn't room for the copies plus 512 MB, it copies nothing and shows a warning.

### Why copies are kept by default

Keeping the copies is the gentlest option for an SSD:

- **Keeping copies (default)**: the first sign-out copies every session once. After that, each sign-out writes only the sessions that changed since the last one, usually a few files and a few MB, and finishes almost at once.
- **Deleting copies after sign-in**: every sign-in deletes the copies, so every sign-out has to write all of them again. With about 300 sessions and 10 account/organization folders, that is roughly 600 MB written at every shift change, plus deleting about 3,000 files afterwards.

In exchange, the copies take disk space: they stay in place, taking about (total size of the session files × number of account/organization folders).

### Why not junctions or hard links

Pointing every account folder at one real folder would avoid copying, but the Claude app deliberately blocks both ways of doing it:

- **Junctions and symbolic links**: before every save, the app checks that the session folder is a real folder and refuses to save if it is a link (`Refusing non-directory at private dir path (symlink/file plant)`). In a test with the real app, the list could be read through a junction, but every session and scheduled-task save failed, including saves for sessions just created.
- **Hard links**: the app treats a file with more than one link as unsafe. It refuses to write to such files and warns when it reads them. It also saves by writing a temporary file and renaming it, which would break the link at every save.

These checks protect against files or folders planted from outside, so CCDC does not work around them. It copies ordinary files, the same kind the app writes itself.

## Install

Download the file for your system from [Releases](../../releases). The files are not code-signed (see [below](#about-the-security-warning)).

**Windows** — run `CCDC-<version>-windows-x64.msi`. No administrator rights are needed.

- Installs to `%LOCALAPPDATA%\Programs\CCDC`, adds a `Continuous Claude Desktop Code` shortcut to the desktop and the Start menu, and starts CCDC in the tray.
- Starts with the computer; turn that off with "Start with the computer" in the tray menu.
- Updating: run the newer `.msi`. It closes CCDC, replaces it and starts it again; your settings are kept.
- Uninstall in Settings > Apps > Installed apps > "Continuous Claude Desktop Code (CCDC)".
- Coming from the 0.3 scripts: just install the `.msi`. On its first start CCDC stops the old version, keeps its settings and account names, and removes it.

**macOS** — open `CCDC-<version>-macos-universal.dmg` and drag **CCDC** to Applications, then open it. A **C** appears in the menu bar. It starts with the computer unless you turn that off in its menu. Runs on Apple Silicon and Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Then start **Continuous Claude Desktop Code** from the applications menu. It starts with the computer unless you turn that off in its menu. On GNOME without the AppIndicator extension the tray icon cannot show (Ubuntu and Linux Mint include it).

Uninstalling leaves the Claude session files, CCDC's settings and the backup folder in place.

### About the security warning

Signing needs a paid certificate tied to a person or company, and CCDC is a small free project, so the downloads are not signed. Your system warns you the first time; that only means the publisher isn't verified, not that anything harmful was found.

CCDC contains nothing harmful. It does not connect to the internet, collect data or change anything outside the Claude app's session folders and its own folder. All of the code is in this repository, and the release files are built from it by GitHub Actions, whose logs are public. You can check a download against `SHA256SUMS.txt` in the release, or [build it yourself](#building).

- **Windows**: if SmartScreen says "Windows protected your PC", click **More info**, then **Run anyway**.
- **macOS**: if macOS says it can't verify the app, open **System Settings > Privacy & Security**, scroll down and click **Open Anyway** next to CCDC. (Or run `xattr -dr com.apple.quarantine /Applications/CCDC.app` in Terminal.)
- **Linux**: `.deb` packages are not checked this way.

## Tray app

An orange **C** appears in the notification area or menu bar (gray while auto sync is off). Click it to open the menu:

| Menu item | What it does |
| --- | --- |
| Status line | Shows whether CCDC is watching sign-ins and sign-outs |
| Auto sync (detect sign-in/sign-out) | Turns the copying and cleanup on or off |
| Move sessions to account now | Gathers every session into one account's folder; asks first if that account is not signed in |
| Delete copies after sign-in | Off by default (copies are kept; see above) |
| Copy/delete speed | Fast (no limit), Normal (60 MB/s) or Gentle (15 MB/s). Deleting is limited to 500 and 100 files per second at Normal and Gentle |
| Estimate line | Shows how long the next sign-out copy would take and, if copies are deleted, the cleanup after sign-in |
| Start with the computer | Starts CCDC when you sign in to the computer |
| Language | Chooses the display language. This item is always labeled in English; each language in the submenu is written in that language |
| View log | Opens CCDC's `watch.log` |
| Name accounts (ccdc.json) | Opens the settings file to give accounts readable names |
| Quit | Removes the icon; if a copy is running, waits for it to finish first |

Notifications appear when copying starts (with the estimated time), when it is done, when a sign-in is detected, and when cleanup is done. A slower speed makes copying take longer, so the next person should wait for the "Copy done" notification, or about the estimated time, before signing in.

The estimate is calculated from the current state of the folders and from the copy and delete speeds measured on this computer (stored in `state.json` in CCDC's folder).

If the icon doesn't show on Windows 11, look under the hidden icons (`^`) on the taskbar. To keep it visible, turn on CCDC in Settings > Personalization > Taskbar > Other system tray icons.

CCDC is a single native program of about 1 MB. While it waits it reads only what was added to the Claude log since the last check, every half second, and uses about 4 MB of memory of its own.

## Settings

Settings are kept in `ccdc.json` in CCDC's folder:

| | CCDC's folder |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Changes you make in the tray menu are saved to this file and apply from the next copy or cleanup, without a restart. `watch.log` and `state.json` are in the same folder.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Key | Values |
| --- | --- |
| `keepCopies` | `true` (default): copies are kept. `false`: copies are deleted after sign-in |
| `speed` | `"fast"` (default), `"normal"`, `"gentle"` |
| `language` | Empty to follow the system language, or a language code such as `"ja"` to use that language |
| `startAtLogin` | Set by "Start with the computer" in the tray menu |
| `aliases` | Names shown for accounts in the tray menu |
| `dataDir`, `appLog` | Only if CCDC can't find the Claude data folder or log by itself: their full paths |

## Languages

The tray menu, notifications and warnings follow the system display language, or the language chosen under **Language** in the tray menu. Supported: English, Korean, Japanese, Chinese (Simplified and Traditional), Spanish, French, German, Italian, Portuguese, Russian, Ukrainian, Polish, Dutch, Turkish, Vietnamese, Indonesian, Thai, Arabic and Hindi. Other languages are shown in English.

All UI text is in one sheet, [`locales.csv`](locales.csv), built into the program: **one row per language**, one column per message. Open it in Excel or Google Sheets; to add a language, add a row and [build](#building) CCDC.

- `lang`: a language tag (`sv`, `pt-BR`, `zh-Hant`, …). A row matching language and region is preferred, then one matching the language.
- `name`: the language's name in that language (`Svenska`); the Language menu shows it.
- Write line breaks inside a cell as `\n`, and keep `{0}`, `{1}` (numbers or names go there).
- Empty cells are shown in English. Save the file as UTF-8 CSV.

Command-line output and `watch.log` stay in English.

## Command line

The same program also works from a terminal (on Windows, `ccdc.exe` in `%LOCALAPPDATA%\Programs\CCDC`; on macOS, `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` and `--app-log <path>` point it at a Claude data folder or log by hand.

## Rules for duplicates

- When a session is in several folders, the copy with the latest `lastActivityAt` wins. Copies keep the source's modified time, so identical copies are recognized by size and time without reading them.
- When a newer copy replaces a file, the old file is moved to `ccdc-backups/<time>/` in the Claude data folder.
- Sessions with a `deleted_<id>` marker are not copied, and their copies in other folders are removed.
- An account/organization signing in for the very first time has no folder yet, so nothing could be copied there at sign-out. In that case its sessions show after its next sign-in or an app restart.

## Caveats

- CCDC depends on the Claude desktop app's internal file layout and log wording; an app update may change how it behaves.
- Session files contain per-account connector (MCP) settings, so some connectors may not connect when a session is opened under another account.
- Scheduled tasks are not shared; each account/organization folder keeps its own.
- Account switching in the Claude Code CLI is not covered. The CLI can already open any session in `~/.claude/projects` with `--resume`, whatever the account.

## Building

CCDC is written in Rust; [install Rust](https://rustup.rs), then:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

Each platform's version is set in `[package.metadata.ccdc]` in `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) builds all three on every push and, for a `v*` tag, attaches them to a draft release.

## License

[MIT](LICENSE)
