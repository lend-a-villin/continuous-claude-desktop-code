# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | **繁體中文** | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

一款系統匣應用程式，讓你切換 Claude 帳戶後，Claude 桌面版的 **Code** 工作階段依然都在。

Claude 桌面版會依帳戶和組織，分別保存一份 Code 工作階段清單。換另一個帳戶登入後，對話明明還在電腦上，清單卻看起來是空的。CCDC 會把工作階段複製到每個帳戶的資料夾，所以下次無論哪個帳戶登入，都能看到全部的工作階段。

非官方工具，與 Anthropic 無關，也未經 Anthropic 認可。

**[下載最新版本](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · 不需要另外安裝其他軟體

| 平台 | 版本 | 狀態 |
| --- | --- | --- |
| Windows | 1.0 | 新推出；已在 Windows 上搭配實際的 Claude 應用程式測試，運作符合說明（延續 0.3 版指令碼） |
| macOS | 0.1 | 新推出；尚未在 Mac 上搭配實際的 Claude 應用程式測試 |
| Linux | 0.1 | 新推出；已在 Debian 12、Ubuntu 24.04 和 Linux Mint 22 上完成安裝測試，尚未搭配實際的 Claude 應用程式測試 |

## 適用對象

- **共用電腦與輪班工作。** 在 24 小時三班制的工作場所，每個班次都換一個人坐到同一個座位上（同一台電腦、同一個系統使用者），各自用自己的 Claude 帳戶登入。有了 CCDC，下一班的人就能接著上一班留下的 Code 工作繼續做。
- **一個人使用多個帳戶。** 如果你在同一台電腦上切換個人帳戶和工作帳戶，或在不同組織之間切換，無論用哪個帳戶登入，你的 Code 工作階段都在。

也就是說，這台電腦上的所有 Code 工作階段，每個登入的帳戶都看得到。如果使用者之間不應看到彼此的工作階段，請不要使用 CCDC。如果使用公司（組織）帳戶，請先確認公司的相關政策：CCDC 也會把工作帳戶的工作階段複製到個人帳戶的資料夾，反之亦然。

## 要解決的問題

桌面版會依帳戶和組織，把 Code 工作階段的**清單**分別存放在不同的資料夾：

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Claude 資料夾 | Claude 記錄 |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>`（或 `%APPDATA%\Claude`） | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

（透過自家雲端服務供應商使用 Claude 的組織，資料夾名稱是 `Claude-3p` 而非 `Claude`；CCDC 也會在那裡尋找。）

**對話內容**本身在 `~/.claude/projects/.../<cliSessionId>.jsonl`，所有帳戶共用。因此有人換另一個帳戶登入後，對話明明還在磁碟上，Code 工作階段清單卻看起來是空的。

## 運作方式

應用程式**只在啟動和有人登入時**讀取它的工作階段資料夾（登入時約兩秒內會讀兩次）。之後無法再讓它單獨重新讀取清單。登出和登入都會記在應用程式自己的記錄中：

- 登出：`[account] Login-state transition (loggedOut: false → true ...)`
- 登入：`[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC 會持續監看這份記錄：

1. **登出** → 把每個工作階段的最新版本複製到所有帳戶/組織資料夾，這樣下一個登入的帳戶，在自己的資料夾裡就已經有這些工作階段。只會複製資料夾中缺少或不是最新的工作階段。
2. **登入** → 預設副本會保留在原處。如果開啟了「登入後刪除副本」，CCDC 會等到應用程式讀完清單（最後一筆登入記錄之後 6 秒），再把已登入帳戶資料夾以外的所有副本刪除。

在應用程式中刪除的工作階段，也會從其他資料夾移除，不會在換帳戶後又出現。

複製前，CCDC 會檢查磁碟可用空間。如果剩餘空間不足以容納副本再加 512 MB，就不會複製任何內容，並顯示警告。

### 為什麼預設保留副本

保留副本對 SSD 最溫和：

- **保留副本（預設）**：第一次登出時會把所有工作階段複製一次。之後每次登出只寫入自上次以來有變更的工作階段，通常是幾個檔案、幾 MB，幾乎瞬間完成。
- **登入後刪除副本**：每次登入都會刪除副本，所以每次登出都必須把所有工作階段重新寫入一遍。以約 300 個工作階段、10 個帳戶/組織資料夾來說，每次交班大約要寫入 600 MB，事後還要刪除約 3,000 個檔案。

代價是佔用磁碟空間：副本會一直保留，大約佔用（工作階段檔案總大小 × 帳戶/組織資料夾數量）的空間。

### 為什麼不用目錄連接或硬連結

讓所有帳戶資料夾都指向同一個實際資料夾，就不需要複製，但 Claude 應用程式刻意阻擋了這兩種做法：

- **目錄連接和符號連結**：每次儲存前，應用程式都會確認工作階段資料夾是真正的資料夾，如果是連結就拒絕儲存（`Refusing non-directory at private dir path (symlink/file plant)`）。以實際應用程式測試時，透過目錄連接可以讀取清單，但所有工作階段和排程工作的儲存全部失敗，連剛建立的工作階段也無法儲存。
- **硬連結**：應用程式把擁有多個連結的檔案視為不安全。它拒絕寫入這類檔案，讀取時也會發出警告。而且它儲存時是先寫入暫存檔再重新命名，每次儲存都會切斷連結。

這些檢查是為了防範從外部植入的檔案或資料夾，因此 CCDC 不會繞過它們，而是複製與應用程式自己寫入的檔案同類的一般檔案。

## 安裝

從 [Releases](../../releases) 下載適合你系統的檔案。這些檔案沒有程式碼簽章（請參閱[下文](#關於安全性警告)）。

**Windows**：執行 `CCDC-<version>-windows-x64.msi`。不需要系統管理員權限。

- 安裝到 `%LOCALAPPDATA%\Programs\CCDC`，在桌面和開始功能表建立 `Continuous Claude Desktop Code` 捷徑，並在系統匣中啟動 CCDC。
- 預設會在開機時啟動；若不需要，可在系統匣選單中關閉「開機時自動啟動」。
- 更新：執行較新的 `.msi` 即可。它會關閉 CCDC、取代檔案後再重新啟動，你的設定會保留。
- 解除安裝：在 設定 > 應用程式 > 已安裝的應用程式 > 「Continuous Claude Desktop Code (CCDC)」。
- 從 0.3 版指令碼升級：直接安裝 `.msi` 即可。CCDC 第一次啟動時會停止舊版本，保留其設定和帳戶名稱，然後將舊版本移除。

**macOS**：開啟 `CCDC-<version>-macos-universal.dmg`，把 **CCDC** 拖曳到「應用程式」資料夾，然後開啟它。選單列會出現 **C** 圖示。除非你在它的選單中關閉，否則會在開機時自動啟動。支援 Apple Silicon 和 Intel。

**Linux**（Ubuntu 22.04+、Debian 12+、Linux Mint 21+，x86_64）：

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

接著從應用程式選單啟動 **Continuous Claude Desktop Code**。除非你在它的選單中關閉，否則會在開機時自動啟動。在沒有 AppIndicator 擴充功能的 GNOME 上，系統匣圖示無法顯示（Ubuntu 和 Linux Mint 已內建此擴充功能）。

解除安裝後，Claude 工作階段檔案、CCDC 的設定和備份資料夾都會留在原處。

### 關於安全性警告

程式碼簽章需要購買與個人或公司綁定的付費憑證，而 CCDC 是小型的免費專案，所以下載檔案沒有簽章。第一次執行時系統會發出警告，但這只代表發行者未經驗證，並不表示發現了任何有害內容。

CCDC 不含任何有害內容。它不會連上網際網路、不會收集資料，除了 Claude 應用程式的工作階段資料夾和自己的資料夾之外，也不會變更任何東西。所有程式碼都在本存放庫中，發行檔案由 GitHub Actions 從這些程式碼建置，建置記錄公開可查。你可以用發行版本中的 `SHA256SUMS.txt` 核對下載的檔案，或[自行建置](#建置)。

- **Windows**：如果 SmartScreen 顯示「Windows 已保護您的電腦」，請按一下 **其他資訊**，再按 **仍要執行**。
- **macOS**：如果 macOS 表示無法驗證此應用程式，請開啟 **系統設定 > 隱私權與安全性**，向下捲動，按一下 CCDC 旁邊的 **強制打開**。（或在「終端機」中執行 `xattr -dr com.apple.quarantine /Applications/CCDC.app`。）
- **Linux**：`.deb` 套件不會經過這類檢查。

## 系統匣應用程式

通知區域或選單列會出現橘色的 **C** 圖示（自動同步關閉時為灰色）。按一下圖示即可開啟選單：

| 選單項目 | 作用 |
| --- | --- |
| 狀態列 | 顯示 CCDC 是否正在監看登入與登出 |
| 自動同步（偵測登入/登出） | 開啟或關閉複製與清理 |
| 立即將工作階段移到帳戶 | 把所有工作階段集中到某個帳戶的資料夾；如果該帳戶未登入，會先詢問 |
| 登入後刪除副本 | 預設關閉（保留副本，見上文） |
| 複製/刪除速度 | 可選「快（不限速）」「一般（60 MB/s）」或「慢（15 MB/s）」。刪除速度在「一般」和「慢」下分別限制為每秒 500 個和 100 個檔案 |
| 預估時間列 | 顯示下次登出時複製大約需要多久；若設定為刪除副本，也會顯示登入後清理所需的時間 |
| 開機時自動啟動 | 登入電腦時自動啟動 CCDC |
| Language | 選擇顯示語言。此項目一律以英文標示，子選單中的各語言名稱都以該語言本身顯示 |
| 檢視記錄 | 開啟 CCDC 的 `watch.log` |
| 為帳戶命名（ccdc.json） | 開啟設定檔，為帳戶取一個容易辨識的名稱 |
| 結束 | 移除圖示；如果正在複製，會等複製完成後再結束 |

開始複製時（附預估時間）、複製完成時、偵測到登入時、清理完成時，都會顯示通知。速度設定得越慢，複製花的時間越長，因此下一個人請等到出現「複製完成」通知，或等候大約預估的時間後再登入。

預估時間是依資料夾目前的狀態，以及在這台電腦上測得的複製與刪除速度計算（儲存在 CCDC 資料夾的 `state.json` 中）。

如果在 Windows 11 上看不到圖示，請查看工作列上的隱藏圖示（`^`）。若要一直顯示，請在 設定 > 個人化 > 工作列 > 其他系統匣圖示 中開啟 CCDC。

CCDC 是一個約 1 MB 的原生程式。等候期間，它每半秒只讀取 Claude 記錄中自上次檢查以來新增的部分，本身只使用約 4 MB 記憶體。

## 設定

設定儲存在 CCDC 資料夾的 `ccdc.json` 中：

| | CCDC 資料夾 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

在系統匣選單中所做的變更會儲存到此檔案，不必重新啟動，從下次複製或清理起生效。`watch.log` 和 `state.json` 也在同一個資料夾。

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| 鍵 | 值 |
| --- | --- |
| `keepCopies` | `true`（預設）：保留副本。`false`：登入後刪除副本 |
| `speed` | `"fast"`（預設）、`"normal"`、`"gentle"` |
| `language` | 留空表示跟隨系統顯示語言；填入 `"ja"` 等語言代碼則以該語言顯示 |
| `startAtLogin` | 由系統匣選單的「開機時自動啟動」設定 |
| `aliases` | 在系統匣選單中顯示的帳戶名稱 |
| `dataDir`、`appLog` | 只在 CCDC 自己找不到 Claude 資料夾或記錄時才需要：填入其完整路徑 |

## 語言

系統匣選單、通知和警告會使用系統的顯示語言，或使用在系統匣選單 **Language** 中所選的語言。支援的語言：英文、韓文、日文、中文（簡體和繁體）、西班牙文、法文、德文、義大利文、葡萄牙文、俄文、烏克蘭文、波蘭文、荷蘭文、土耳其文、越南文、印尼文、泰文、阿拉伯文和印地文。其他語言以英文顯示。

所有介面文字都在一張表格 [`locales.csv`](locales.csv) 中，並內建於程式裡：**每列一種語言**，每欄一則訊息。可以用 Excel 或 Google 試算表開啟；要新增語言，只要新增一列，再[建置](#建置) CCDC。

- `lang`：語言標記（`sv`、`pt-BR`、`zh-Hant` 等）。優先使用語言和地區都相符的列，其次使用語言相符的列。
- `name`：以該語言書寫的語言名稱（`Svenska`）；Language 選單中顯示的就是它。
- 儲存格內的換行請寫成 `\n`，並保留 `{0}`、`{1}`（那裡會填入數字或名稱）。
- 空白儲存格以英文顯示。請儲存為 UTF-8 CSV。

命令列輸出和 `watch.log` 維持英文。

## 命令列

同一個程式也可以在終端機中使用（Windows 上是 `%LOCALAPPDATA%\Programs\CCDC` 中的 `ccdc.exe`；macOS 上是 `/Applications/CCDC.app/Contents/MacOS/ccdc`）：

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` 和 `--app-log <path>` 可手動指定 Claude 資料夾或記錄的位置。

## 重複檔案的規則

- 同一個工作階段出現在多個資料夾時，以 `lastActivityAt` 最新的那份為準。副本會保留來源檔案的修改時間，因此相同的副本不必讀取內容，只看大小和時間就能辨識。
- 用較新的副本取代檔案時，舊檔案會移到 Claude 資料夾中的 `ccdc-backups/<time>/`。
- 帶有 `deleted_<id>` 標記的工作階段不會被複製，其他資料夾中的副本也會被移除。
- 第一次登入的帳戶/組織還沒有資料夾，因此登出時無法複製到那裡。這種情況下，工作階段會在該帳戶下次登入或重新啟動應用程式後顯示。

## 注意事項

- CCDC 依賴 Claude 桌面版內部的檔案結構和記錄措辭，應用程式更新後行為可能改變。
- 工作階段檔案中包含各帳戶的連接器（MCP）設定，因此在其他帳戶下開啟工作階段時，部分連接器可能無法連線。
- 排程工作不會共用，由各帳戶/組織資料夾分別保存。
- 本工具不處理 Claude Code CLI 中的帳戶切換。CLI 本來就能用 `--resume` 開啟 `~/.claude/projects` 中的任何工作階段，與帳戶無關。

## 建置

CCDC 以 Rust 撰寫；請先[安裝 Rust](https://rustup.rs)，然後執行：

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

各平台的版本號設定在 `Cargo.toml` 的 `[package.metadata.ccdc]` 中。GitHub Actions（[`.github/workflows/build.yml`](.github/workflows/build.yml)）會在每次推送時建置三個平台的版本；若是 `v*` 標籤，還會把它們附加到草稿版本中。

## 授權

[MIT](LICENSE)
