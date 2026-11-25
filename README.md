# Continuous Claude Desktop Code (CCDC)

**English** | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

A Windows tray app that keeps your Claude desktop **Code** sessions when you switch Claude accounts.

The Claude desktop app keeps a separate Code session list for each account and organization. Sign in with another account and the list looks empty, even though the conversations are still on the PC. CCDC copies the sessions across, so whichever account signs in next sees all of them.

Unofficial; not affiliated with or endorsed by Anthropic.

**[Download the latest release](../../releases/latest)** · Windows 10/11 · Node.js 18+ · no admin rights needed

## Who it is for

- **Shared PCs and shift work.** In a workplace that runs 24 hours in three shifts, a different person sits at the same desk (one PC, one Windows user) on each shift and signs in with their own Claude account. With CCDC, the next shift picks up the Code work the previous shift left off.
- **One person, several accounts.** If you switch between a personal and a work account, or between organizations, on the same PC, your Code sessions are there whichever account you sign in with.

Every Code session on the PC becomes visible to every account that signs in on it. Don't use CCDC where people must not see each other's sessions. If you use a company (organization) account, check your company's policy first: CCDC copies a work account's sessions into a personal account's folder too, and the other way round.

## The problem it solves

The desktop app keeps the **list** of Code sessions in a separate folder for each account and organization:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

The **conversations** themselves are in `~/.claude/projects\...\<cliSessionId>.jsonl`, which all accounts share. So after someone signs in with another account, the Code session list looks empty even though the conversations are still on disk.

## How it works

The app reads its session folder only **when it starts and when someone signs in** (twice, within about two seconds). There is no way to make it reload just the list later. The app's own log, `%LOCALAPPDATA%\Claude\Logs\main.log`, records both events:

- sign-out: `[account] Login-state transition (loggedOut: false → true ...)`
- sign-in: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC follows that log:

1. **Sign-out** → it copies the newest version of every session into every account/organization folder, so the next account to sign in already finds them in its own folder. Only sessions that are missing or out of date in a folder are copied.
2. **Sign-in** → by default the copies stay where they are. If "Delete copies after sign-in" is turned on, CCDC waits until the app has read its list (6 seconds after the last sign-in line) and then removes the copies everywhere except the signed-in account's folder.

A session deleted in the app is also removed from the other folders, so it does not come back under another account.

Before copying, CCDC checks free disk space. If there isn't room for the copies plus 512 MB, it copies nothing and shows a warning.

### Why copies are kept by default

Keeping the copies is the gentlest option for an SSD:

- **Copies kept (default)**: the first sign-out copies everything once. After that, each sign-out writes only the sessions that changed since the last one, usually a few files and a few MB, and finishes almost at once.
- **Copies deleted after sign-in**: every sign-in deletes the copies, so every sign-out has to write all of them again. With about 300 sessions and 10 account/organization folders, that is roughly 600 MB written at every shift change, plus deleting about 3,000 files afterwards.

The cost of keeping them is disk space: the copies stay on disk, taking about (total size of the session files × number of account/organization folders).

### Why not junctions or hard links

Pointing every account folder at one real folder would avoid copying, but the Claude app deliberately blocks both ways of doing it:

- **Junctions and symbolic links**: before every save, the app checks that the session folder is a real folder and refuses to save if it is a link (`Refusing non-directory at private dir path (symlink/file plant)`). In a test with the real app, the list could be read through a junction, but every session and scheduled-task save failed, including saves for brand-new sessions.
- **Hard links**: the app treats a file with more than one link as unsafe. It refuses to write to such files and warns when it reads them. It also saves by writing a temporary file and renaming it, which would break the link at every save.

These checks protect against files or folders planted from outside, so CCDC does not work around them. It copies ordinary files, the same kind the app writes itself.

## Requirements

- Windows 10 or 11, with the Claude desktop app (MSIX install)
- [Node.js](https://nodejs.org) 18 or later

## Install

Download the installer from [Releases](../../releases), unzip it, and run `setup.cmd`. No administrator rights are needed.

- Installs to `%LOCALAPPDATA%\Programs\CCDC`.
- Adds a `Continuous Claude Desktop Code` shortcut to the desktop and the Start menu.
- Appears in Settings > Apps > Installed apps as "Continuous Claude Desktop Code (CCDC)"; uninstall it there.
- Starts in the tray right after installing.
- `setup.cmd /startup` also starts it when you sign in to Windows; `setup.cmd /nostartup` stops that. Without either option, a reinstall keeps the earlier choice.
- A reinstall stops CCDC if it is running and replaces its files; your settings (`ccdc.json`) are kept.
- Uninstalling leaves the Claude session files and CCDC's backup folder in place.

To run it straight from this repository instead, run `ccdc-tray.vbs`, or use the command line below.

## Tray app

An orange **C** appears in the notification area (gray while auto sync is off). Click it to open the menu:

| Menu item | What it does |
| --- | --- |
| Status line | Whether CCDC is watching sign-ins and sign-outs |
| Auto sync (detect sign-in/sign-out) | Turns the copying and cleanup on or off |
| Move sessions to account now | Gathers every session into one account's folder; asks first if that account is not signed in |
| Delete copies after sign-in | Off by default (copies are kept; see above) |
| Copy/delete speed | Fast (no limit), Normal (60 MB/s) or Gentle (15 MB/s). Deleting is limited to 500 and 100 files per second at Normal and Gentle |
| Estimate line | How long the next sign-out copy (and, if deleting, the sign-in cleanup) would take |
| Language | Display language. Always labeled in English; each language in the submenu is written in that language |
| View log | Opens `%LOCALAPPDATA%\CCDC\watch.log` |
| Name accounts (ccdc.json) | Opens the settings file to give accounts readable names |
| Quit | Removes the icon (waits for a running copy to finish) |

Notifications appear when copying starts (with the estimated time), when it is done, when a sign-in is detected, and when cleanup is done. A slower speed makes copying take longer, so the next person should wait for the "Copy done" notification, or about the estimated time, before signing in.

The estimate is calculated from the current state of the folders and from the copy and delete speeds measured on this PC (stored in `%LOCALAPPDATA%\CCDC\state.json`).

If the icon doesn't show on Windows 11, look under the hidden icons (`^`) on the taskbar. To keep it visible, turn on Windows PowerShell in Settings > Personalization > Taskbar > Other system tray icons.

### Memory use

The tray watches the app log itself, reading only what was added since the last check. Node.js runs only while copying or cleaning up and exits afterwards, so between shift changes only the tray is running. It hands unused memory back to Windows about once a minute while idle (around 15 MB in use).

## Settings

Settings are kept in `ccdc.json` in the program folder. The tray menu changes them for you; changes apply to the next copy or cleanup without a restart.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Key | Values |
| --- | --- |
| `keepCopies` | `true` (default): keep the copies. `false`: delete them after sign-in |
| `speed` | `"fast"` (default), `"normal"`, `"gentle"` |
| `language` | Empty to follow Windows, or a language code such as `"ja"` |
| `aliases` | Names shown for accounts in the tray menu |

## Languages

The tray menu, notifications, warnings and the install/uninstall dialogs follow the Windows display language, or the language chosen under **Language** in the tray menu. Supported: English, Korean, Japanese, Chinese (Simplified and Traditional), Spanish, French, German, Italian, Portuguese, Russian, Ukrainian, Polish, Dutch, Turkish, Vietnamese, Indonesian, Thai, Arabic and Hindi. Other languages are shown in English.

All UI text is in one sheet, [`locales.csv`](locales.csv): **one row per language**, one column per message. Open it in Excel or Google Sheets; to add a language, add a row.

- `lang`: a Windows culture name (`sv`, `pt-BR`, `zh-Hant`, …). A row matching language and region is preferred, then one matching the language.
- `name`: the language's name in that language (`Svenska`); the Language menu shows it.
- Write line breaks inside a cell as `\n`, and keep `{0}`, `{1}` (numbers or names go there).
- Empty cells fall back to English. Save as UTF-8 CSV.

Command-line output and `watch.log` stay in English because the tray reads them.

## Command line

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

The tray runs these one-shot jobs: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>`, and `node ccdc.mjs startup` (catch up with the app's current state when the tray starts or auto sync is turned back on).

## Rules for duplicates

- When a session is in several folders, the copy with the latest `lastActivityAt` wins. Copies keep the source's modified time, so identical copies are recognized by size and time without reading them.
- When a newer copy replaces a file, the old file is moved to `ccdc-backups\<time>\` in the app's data folder.
- Sessions with a `deleted_<id>` marker are not copied, and their copies in other folders are removed.
- An account/organization signing in for the very first time has no folder yet, so nothing could be copied there at sign-out. Its sessions show after its next sign-in or an app restart.

## Caveats

- CCDC depends on the Claude desktop app's internal file layout and log wording; an app update may change how it behaves.
- Session files contain per-account connector (MCP) settings, so some connectors may not connect when a session is opened under another account.
- Scheduled tasks are not shared; each account/organization folder keeps its own.
- Account switching in the Claude Code CLI is not covered. The CLI can already open any session in `~/.claude/projects` with `--resume`, whatever the account.

## License

[MIT](LICENSE)
