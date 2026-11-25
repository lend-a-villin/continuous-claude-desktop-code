# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | **简体中文** | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

一款托盘应用：切换 Claude 账户后，Claude 桌面版中的 **Code** 会话依然都在。

Claude 桌面版按账户和组织分别保存 Code 会话列表。换一个账户登录，列表就显示为空，尽管对话其实还在电脑上。CCDC 会把会话复制到各个账户那里，这样无论下一个登录的是哪个账户，都能看到全部会话。

非官方工具，与 Anthropic 无关，也未获 Anthropic 认可。

**[下载最新版本](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · 无需安装其他软件

| 平台 | 版本 | 状态 |
| --- | --- | --- |
| Windows | 1.0 | 日常使用中（接续 0.3 版脚本） |
| macOS | 0.1 | 新版本；尚未在 Mac 上配合真实的 Claude 应用测试 |
| Linux | 0.1 | 新版本；已在 Debian 12、Ubuntu 24.04 和 Linux Mint 22 上测试安装，尚未配合真实的 Claude 应用测试 |

## 适用对象

- **共用电脑和轮班工作。** 在 24 小时三班倒的工作场所，每个班次都换一个人坐到同一个工位上（同一台电脑、同一个系统用户），各自用自己的 Claude 账户登录。有了 CCDC，下一班的人就能接着做上一班留下的 Code 工作。
- **一个人，多个账户。** 如果你在同一台电脑上切换个人账户和工作账户，或在不同组织之间切换，无论用哪个账户登录，Code 会话都在。

这台电脑上的所有 Code 会话，对在这里登录的每个账户都可见。如果使用者之间不应看到彼此的会话，请不要使用 CCDC。如果使用公司（组织）账户，请先确认公司的相关政策：CCDC 也会把工作账户的会话复制到个人账户的文件夹中，反之亦然。

## 要解决的问题

桌面版按账户和组织，把 Code 会话的**列表**分别存放在不同的文件夹中：

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Claude 数据文件夹 | Claude 日志 |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>`（或 `%APPDATA%\Claude`） | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

（通过自己的云服务商使用 Claude 的组织，用的是 `Claude-3p` 而不是 `Claude`；CCDC 也会查找那里。）

**对话**内容本身在 `~/.claude/projects/.../<cliSessionId>.jsonl` 中，所有账户共用。所以换一个账户登录后，对话明明还在磁盘上，Code 会话列表却显示为空。

## 工作原理

应用**只在启动时和登录时**读取会话文件夹（登录时会在约两秒内读取两次）。之后无法再让它单独重新读取列表。这两种事件都会记录在应用自己的日志中：

- 退出登录：`[account] Login-state transition (loggedOut: false → true ...)`
- 登录：`[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC 持续监视这份日志，并执行以下操作：

1. **退出登录** → 把每个会话的最新版本复制到所有账户/组织文件夹中，这样，下次无论哪个账户登录，它的文件夹里都已经有这些会话。只复制文件夹中缺失或不是最新的会话。
2. **登录** → 默认情况下副本保持不动。如果开启了“登录后删除副本”，CCDC 会等到应用读完列表（最后一条登录日志之后 6 秒），再删除除已登录账户文件夹以外所有位置的副本。

在应用中删除的会话，也会从其他文件夹中删除，因此不会在其他账户下重新出现。

复制前，CCDC 会检查磁盘可用空间。如果剩余空间不足以容纳副本再加 512 MB，就不复制任何内容，并显示警告。

### 为什么默认保留副本

保留副本对 SSD 最友好：

- **保留副本（默认）**：第一次退出登录时一次性复制全部会话。之后每次退出登录只写入自上次以来有变化的会话，通常是几个文件、几 MB，几乎立刻完成。
- **登录后删除副本**：每次登录都会删除副本，所以每次退出登录都要把所有会话重新写入一遍。以大约 300 个会话、10 个账户/组织文件夹为例，每次换班要写入约 600 MB，事后还要删除约 3,000 个文件。

代价是占用磁盘空间：副本会一直保留，大约占用（会话文件总大小 × 账户/组织文件夹数量）的空间。

### 为什么不用目录联接或硬链接

让所有账户文件夹都指向同一个真实文件夹，就不需要复制了，但 Claude 应用有意阻止了这两种做法：

- **目录联接（junction）和符号链接**：每次保存前，应用都会检查会话文件夹是不是真正的文件夹，如果是链接就拒绝保存（`Refusing non-directory at private dir path (symlink/file plant)`）。用真实应用测试时，通过联接可以读取列表，但所有会话和计划任务的保存都失败了，连全新会话的保存也不例外。
- **硬链接**：应用把拥有多个链接的文件视为不安全。它拒绝写入这类文件，读取时还会发出警告。而且它保存时是先写临时文件再重命名，每次保存都会破坏硬链接。

这些检查是为了防范从外部植入的文件或文件夹，因此 CCDC 不会绕过它们，而是复制与应用自己写入的文件同类的普通文件。

## 安装

从 [Releases](../../releases) 下载适合你系统的文件。这些文件没有代码签名（见[下文](#关于安全警告)）。

**Windows**：运行 `CCDC-<version>-windows-x64.msi`。不需要管理员权限。

- 安装到 `%LOCALAPPDATA%\Programs\CCDC`，在桌面和开始菜单中添加 `Continuous Claude Desktop Code` 快捷方式，并在托盘中启动 CCDC。
- 开机时自动启动；如不需要，可在托盘菜单中关闭“开机时自动启动”。
- 更新：运行较新的 `.msi` 即可。它会关闭 CCDC、替换文件并重新启动，你的设置会保留。
- 卸载：在 设置 > 应用 > 已安装的应用 中找到“Continuous Claude Desktop Code (CCDC)”。
- 从 0.3 版脚本升级：直接安装 `.msi` 即可。CCDC 首次启动时会停止旧版本，保留其设置和账户名称，然后将其移除。

**macOS**：打开 `CCDC-<version>-macos-universal.dmg`，把 **CCDC** 拖到“应用程序”文件夹，然后打开它。菜单栏中会出现一个 **C**。它默认开机自动启动，可在它的菜单中关闭。支持 Apple 芯片和 Intel。

**Linux**（Ubuntu 22.04+、Debian 12+、Linux Mint 21+，x86_64）：

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

然后从应用程序菜单启动 **Continuous Claude Desktop Code**。它默认开机自动启动，可在它的菜单中关闭。在没有 AppIndicator 扩展的 GNOME 上，托盘图标无法显示（Ubuntu 和 Linux Mint 已自带该扩展）。

卸载后，Claude 会话文件、CCDC 的设置和备份文件夹都会保留。

### 关于安全警告

代码签名需要绑定个人或公司的付费证书，而 CCDC 是一个小型免费项目，所以下载的文件没有签名。系统会在首次运行时发出警告；这只表示发布者未经验证，并不代表发现了任何有害内容。

CCDC 不含任何有害内容。它不联网、不收集数据，除了 Claude 应用的会话文件夹和它自己的文件夹之外，不会改动任何东西。所有代码都在本仓库中，发布文件由 GitHub Actions 从这些代码构建，构建日志公开可查。你可以用发布页中的 `SHA256SUMS.txt` 校验下载的文件，也可以[自己构建](#构建)。

- **Windows**：如果 SmartScreen 提示“Windows 已保护你的电脑”，请点击**更多信息**，再点击**仍要运行**。
- **macOS**：如果 macOS 提示无法验证该应用，请打开 **系统设置 > 隐私与安全性**，向下滚动，点击 CCDC 旁边的**仍要打开**。（也可以在终端中运行 `xattr -dr com.apple.quarantine /Applications/CCDC.app`。）
- **Linux**：`.deb` 软件包不会经过这类检查。

## 托盘应用

通知区域或菜单栏中会出现橙色的 **C** 图标（自动同步关闭时为灰色）。单击图标打开菜单：

| 菜单项 | 作用 |
| --- | --- |
| 状态行 | 显示 CCDC 是否正在监视登录和退出登录 |
| 自动同步（检测登录/退出登录） | 打开或关闭复制和清理 |
| 立即将会话移到账户 | 把所有会话集中到某个账户的文件夹；该账户未登录时会先确认 |
| 登录后删除副本 | 默认关闭（保留副本；见上文） |
| 复制/删除速度 | 可选“快（不限速）”“普通（60 MB/s）”或“慢（15 MB/s）”。删除速度在“普通”和“慢”下分别限制为每秒 500 个和 100 个文件 |
| 预计时间行 | 显示下次退出登录时复制大约需要多久；如果设置为删除副本，还会显示登录后清理所需的时间 |
| 开机时自动启动 | 登录电脑时启动 CCDC |
| Language | 选择显示语言。此项始终以英文显示；子菜单中的各语言名称均以该语言本身显示 |
| 查看日志 | 打开 CCDC 的 `watch.log` |
| 为账户命名（ccdc.json） | 打开设置文件，为账户设置易读的名称 |
| 退出 | 移除图标；如果正在复制，会等复制完成后再退出 |

复制开始时（附预计时间）、复制完成时、检测到登录时、清理完成时，都会显示通知。速度设得越慢，复制耗时越长，因此下一个人请等到出现“复制完成”通知，或等待大约预计的时间之后再登录。

预计时间根据文件夹的当前状态，以及在本机测得的复制和删除速度计算（保存在 CCDC 文件夹的 `state.json` 中）。

如果在 Windows 11 上看不到图标，请查看任务栏上的隐藏图标（`^`）。要让它一直显示，请在 设置 > 个性化 > 任务栏 > 其他系统托盘图标 中打开 CCDC。

CCDC 是一个约 1 MB 的原生程序。等待期间，它每半秒读取一次 Claude 日志中自上次检查以来新增的内容，自身只占用约 4 MB 内存。

## 设置

设置保存在 CCDC 文件夹的 `ccdc.json` 中：

| | CCDC 文件夹 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

在托盘菜单中所做的修改会保存到此文件，无需重启，从下一次复制或清理起生效。`watch.log` 和 `state.json` 也在同一文件夹中。

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| 键 | 取值 |
| --- | --- |
| `keepCopies` | `true`（默认）：保留副本。`false`：登录后删除副本 |
| `speed` | `"fast"`（默认）、`"normal"`、`"gentle"` |
| `language` | 留空表示跟随系统语言；填写 `"ja"` 等语言代码则使用该语言 |
| `startAtLogin` | 由托盘菜单中的“开机时自动启动”设置 |
| `aliases` | 在托盘菜单中为账户显示的名称 |
| `dataDir`、`appLog` | 仅在 CCDC 自己找不到 Claude 数据文件夹或日志时填写：二者的完整路径 |

## 语言

托盘菜单、通知和警告会使用系统显示语言，或使用在托盘菜单 **Language** 中选择的语言。支持的语言：英语、韩语、日语、中文（简体和繁体）、西班牙语、法语、德语、意大利语、葡萄牙语、俄语、乌克兰语、波兰语、荷兰语、土耳其语、越南语、印尼语、泰语、阿拉伯语、印地语。其他语言以英文显示。

所有界面文字都在一张表格 [`locales.csv`](locales.csv) 中，并内置在程序里：**每行一种语言**，每列一条文本。可以用 Excel 或 Google 表格打开；要添加语言，只需添加一行，再[构建](#构建) CCDC。

- `lang`：语言标记（`sv`、`pt-BR`、`zh-Hant` 等）。优先使用语言和地区都匹配的行，其次使用语言匹配的行。
- `name`：用该语言书写的语言名称（`Svenska`）；Language 菜单中显示的就是它。
- 单元格内的换行写作 `\n`，并请保留 `{0}`、`{1}`（那里会填入数字或名称）。
- 空单元格以英文显示。请保存为 UTF-8 CSV。

命令行输出和 `watch.log` 保持英文。

## 命令行

同一个程序也可以在终端中使用（Windows 上是 `%LOCALAPPDATA%\Programs\CCDC` 中的 `ccdc.exe`；macOS 上是 `/Applications/CCDC.app/Contents/MacOS/ccdc`）：

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

用 `--data-dir <path>` 和 `--app-log <path>` 可以手动指定 Claude 数据文件夹或日志。

## 重复文件的规则

- 同一会话出现在多个文件夹中时，以 `lastActivityAt` 最新的那份为准。副本沿用源文件的修改时间，因此相同的副本只需比较大小和时间就能识别，不必读取内容。
- 用较新的副本替换文件时，旧文件会移到 Claude 数据文件夹的 `ccdc-backups/<time>/` 中。
- 带有 `deleted_<id>` 标记的会话不会被复制，它们在其他文件夹中的副本会被删除。
- 首次登录的账户/组织还没有文件夹，因此退出登录时无法复制到那里。这种情况下，会话会在该账户下次登录或重启应用后显示。

## 注意事项

- CCDC 依赖 Claude 桌面版内部的文件结构和日志措辞，应用更新后行为可能改变。
- 会话文件中包含各账户的连接器（MCP）设置，因此在其他账户下打开会话时，部分连接器可能无法连接。
- 计划任务不共享，由各账户/组织文件夹分别保存。
- 本工具不处理 Claude Code CLI 中的账户切换。CLI 本来就能用 `--resume` 打开 `~/.claude/projects` 中的任何会话，与账户无关。

## 构建

CCDC 用 Rust 编写；先[安装 Rust](https://rustup.rs)，然后运行：

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

各平台的版本号在 `Cargo.toml` 的 `[package.metadata.ccdc]` 中设置。GitHub Actions（[`.github/workflows/build.yml`](.github/workflows/build.yml)）会在每次推送时构建全部三个平台，遇到 `v*` 标签时还会把它们附加到一个草稿发布中。

## 许可证

[MIT](LICENSE)
