# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | **繁體中文** | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

一款 Windows 系統匣應用程式，讓 Claude 桌面版的本機 **Code** 工作階段，不論是哪個 Claude 帳戶在這台電腦上登入，都能繼續使用。

非官方工具，與 Anthropic 無關，也未經 Anthropic 認可。

## 適用對象

CCDC 專為 **24 小時三班制的工作場所**設計。每個班次都換一個人坐到同一個座位上（同一台電腦、同一個 Windows 使用者），各自用自己的 Claude 帳戶登入 Claude 桌面版。有了 CCDC，下一班的人登入後，就能接著上一班留下的 Code 工作繼續做。

也就是說，這台電腦上的所有 Code 工作階段，每個登入的人都看得到。如果使用者之間不應看到彼此的工作階段，請不要使用 CCDC。如果是公司（組織）帳戶，也請確認公司的相關政策。

## 要解決的問題

桌面版會依帳戶和組織，把 Code 工作階段的**清單**分別存放在不同的資料夾：

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

**對話內容**本身在 `~/.claude/projects\...\<cliSessionId>.jsonl`，所有帳戶共用。因此有人換另一個帳戶登入後，對話明明還在磁碟上，Code 工作階段清單卻看起來是空的。

## 運作方式

應用程式**只在啟動和登入時**讀取它的工作階段資料夾（登入時約兩秒內會讀兩次）。之後無法再讓它單獨重新讀取清單。登出和登入都會記在應用程式自己的記錄 `%LOCALAPPDATA%\Claude\Logs\main.log` 中：

- 登出：`[account] Login-state transition (loggedOut: false → true ...)`
- 登入：`[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC 會持續監看這份記錄，並執行下列動作：

1. **登出** → 把每個工作階段的最新版本複製到所有帳戶/組織資料夾，這樣下次無論哪個帳戶登入，它的資料夾裡都已經有這些工作階段。只會複製資料夾中缺少或不是最新的工作階段。
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

- **目錄連接和符號連結**：每次儲存前，應用程式都會確認工作階段資料夾是真正的資料夾，如果是連結就拒絕儲存（`Refusing non-directory at private dir path (symlink/file plant)`）。以實際應用程式測試時，透過連接可以讀取清單，但所有工作階段和排程工作的儲存全部失敗，連全新工作階段的儲存也不例外。
- **硬連結**：應用程式把擁有多個連結的檔案視為不安全。它拒絕寫入這類檔案，讀取時也會發出警告。而且它儲存時是先寫入暫存檔再重新命名，每次儲存都會切斷連結。

這些檢查是為了防範從外部植入的檔案或資料夾，因此 CCDC 不會繞過它們，而是複製與應用程式自己寫入的檔案同類的一般檔案。

## 系統需求

- Windows 10 或 11，以及 Claude 桌面版（MSIX 安裝版）
- [Node.js](https://nodejs.org) 18 或更新版本

## 安裝

從 [Releases](../../releases) 下載安裝檔，解壓縮後執行 `setup.cmd`。不需要系統管理員權限。

- 安裝位置：`%LOCALAPPDATA%\Programs\CCDC`。
- 會在桌面和開始功能表建立 `Continuous Claude Desktop Code` 捷徑。
- 在 設定 > 應用程式 > 已安裝的應用程式 中顯示為「Continuous Claude Desktop Code (CCDC)」，可在那裡解除安裝。
- 安裝完成後立即在系統匣中啟動。
- 若要在登入 Windows 時自動啟動，請以 `setup.cmd /startup` 安裝；若要取消自動啟動，請以 `setup.cmd /nostartup` 安裝。重新安裝時如果兩個選項都不加，會沿用先前的選擇。
- 重新安裝時，如果 CCDC 正在執行，會先將其停止，再取代檔案；你的設定（`ccdc.json`）會保留。
- 解除安裝後，Claude 工作階段檔案和 CCDC 的備份資料夾會留在原處。

如果想不安裝、直接從本存放庫執行，請執行 `ccdc-tray.vbs`，或使用下方的命令列。

## 系統匣應用程式

通知區域會出現橘色的 **C** 圖示（自動同步關閉時為灰色）。按一下圖示即可開啟選單：

| 選單項目 | 作用 |
| --- | --- |
| 狀態列 | 顯示 CCDC 是否正在監看登入與登出 |
| 自動同步（偵測登入/登出） | 開啟或關閉複製與清理 |
| 立即將工作階段移到帳戶 | 把所有工作階段集中到某個帳戶的資料夾；如果該帳戶未登入，會先詢問 |
| 登入後刪除副本 | 預設關閉（保留副本，見上文） |
| 複製/刪除速度 | 可選「快（不限速）」「一般（60 MB/s）」或「慢（15 MB/s）」。刪除速度在「一般」和「慢」下分別限制為每秒 500 個和 100 個檔案 |
| 預估時間列 | 顯示下次登出時複製大約需要多久；若設定為刪除副本，也會顯示登入後清理所需的時間 |
| Language | 選擇顯示語言。此項目一律以英文標示，子選單中的各語言名稱都以該語言本身顯示 |
| 檢視記錄 | 開啟 `%LOCALAPPDATA%\CCDC\watch.log` |
| 為帳戶命名（ccdc.json） | 開啟設定檔，為帳戶取一個容易辨識的名稱 |
| 結束 | 移除圖示；如果正在複製，會等複製完成後再結束 |

開始複製時（附預估時間）、複製完成時、偵測到登入時、清理完成時，都會顯示通知。速度設定得越慢，複製花的時間越長，因此下一個人請等到出現「複製完成」通知，或等候大約預估的時間後再登入。

預估時間是依資料夾目前的狀態，以及在這台電腦上測得的複製與刪除速度計算（儲存在 `%LOCALAPPDATA%\CCDC\state.json`）。

如果在 Windows 11 上看不到圖示，請查看工作列上的隱藏圖示（`^`）。若要一直顯示，請在 設定 > 個人化 > 工作列 > 其他系統匣圖示 中開啟 Windows PowerShell。

### 記憶體用量

應用程式記錄由系統匣直接監看，只讀取自上次檢查以來新增的部分。Node.js 只在複製或清理期間執行，結束後就會退出，所以兩次交班之間只有系統匣在執行。閒置時，系統匣大約每分鐘會把用不到的記憶體還給 Windows（使用量約 15 MB）。

## 設定

設定儲存在程式資料夾的 `ccdc.json` 中。在系統匣選單中變更後會儲存到此檔案，不必重新啟動，從下次複製或清理起生效。

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| 鍵 | 值 |
| --- | --- |
| `keepCopies` | `true`（預設）：保留副本。`false`：登入後刪除副本 |
| `speed` | `"fast"`（預設）、`"normal"`、`"gentle"` |
| `language` | 留空表示跟隨 Windows 顯示語言；填入 `"ja"` 等語言代碼則以該語言顯示 |
| `aliases` | 在系統匣選單中顯示的帳戶名稱 |

## 語言

系統匣選單、通知、警告以及安裝/解除安裝對話方塊會使用 Windows 顯示語言，或使用在系統匣選單 **Language** 中所選的語言。支援的語言：英文、韓文、日文、中文（簡體和繁體）、西班牙文、法文、德文、義大利文、葡萄牙文、俄文、烏克蘭文、波蘭文、荷蘭文、土耳其文、越南文、印尼文、泰文、阿拉伯文和印地文。其他語言以英文顯示。

所有介面文字都在一張表格 [`locales.csv`](locales.csv) 中：**每列一種語言**，每欄一則訊息。可以用 Excel 或 Google 試算表開啟；要新增語言，只要新增一列。

- `lang`：Windows 文化特性名稱（`sv`、`pt-BR`、`zh-Hant` 等）。優先使用語言和地區都相符的列，其次使用語言相符的列。
- `name`：以該語言書寫的語言名稱（`Svenska`）；Language 選單中顯示的就是它。
- 儲存格內的換行請寫成 `\n`，並保留 `{0}`、`{1}`（那裡會填入數字或名稱）。
- 空白儲存格以英文顯示。請儲存為 UTF-8 CSV。

命令列輸出和 `watch.log` 由系統匣讀取，因此維持英文。

## 命令列

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

系統匣會在需要時執行下列單次工作：`node ccdc.mjs signout`、`node ccdc.mjs signin --account <id> --org <id>`，以及 `node ccdc.mjs startup`（在系統匣啟動或重新開啟自動同步時，同步至應用程式目前的狀態）。

## 重複檔案的規則

- 同一個工作階段出現在多個資料夾時，以 `lastActivityAt` 最新的那份為準。副本會保留來源檔案的修改時間，因此相同的副本不必讀取內容，只看大小和時間就能辨識。
- 用較新的副本取代檔案時，舊檔案會移到應用程式資料所在資料夾的 `ccdc-backups\<時間>\`。
- 帶有 `deleted_<id>` 標記的工作階段不會被複製，其他資料夾中的副本也會被移除。
- 第一次登入的帳戶/組織還沒有資料夾，因此登出時無法複製到那裡。這種情況下，工作階段會在該帳戶下次登入或重新啟動應用程式後顯示。

## 注意事項

- CCDC 依賴 Claude 桌面版內部的檔案結構和記錄措辭，應用程式更新後行為可能改變。
- 工作階段檔案中包含各帳戶的連接器（MCP）設定，因此在其他帳戶下開啟工作階段時，部分連接器可能無法連線。
- 排程工作不會共用，由各帳戶/組織資料夾分別保存。
- 本工具不處理 Claude Code CLI 中的帳戶切換。CLI 本來就能用 `--resume` 開啟 `~/.claude/projects` 中的任何工作階段，與帳戶無關。

## 授權

[MIT](LICENSE)
