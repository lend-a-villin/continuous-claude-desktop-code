# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | **Tiếng Việt** | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Ứng dụng khay hệ thống giúp bạn giữ lại các phiên **Code** của ứng dụng Claude cho máy tính khi chuyển đổi tài khoản Claude.

Ứng dụng Claude cho máy tính lưu một danh sách phiên Code riêng cho từng tài khoản và tổ chức. Khi đăng nhập bằng tài khoản khác, danh sách trông như trống, dù các cuộc trò chuyện vẫn còn trên máy. CCDC sao chép các phiên sang, nên dù tài khoản nào đăng nhập tiếp theo cũng thấy được tất cả.

Công cụ không chính thức; không liên kết với Anthropic và không được Anthropic chứng thực.

**[Tải bản phát hành mới nhất](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · không cần cài thêm gì

| Nền tảng | Phiên bản | Trạng thái |
| --- | --- | --- |
| Windows | 1.0 | Mới; đã thử với ứng dụng Claude thật trên Windows và hoạt động như mô tả (kế tục các script 0.3) |
| macOS | 0.1 | Mới; chưa thử với ứng dụng Claude thật trên máy Mac |
| Linux | 0.1 | Mới; đã thử cài đặt trên Debian 12, Ubuntu 24.04 và Linux Mint 22, nhưng chưa thử với ứng dụng Claude thật |

## Dành cho ai

- **Máy tính dùng chung và làm việc theo ca.** Ở nơi làm việc hoạt động 24 giờ theo ba ca, mỗi ca có một người khác ngồi vào cùng một chỗ (một máy tính, một người dùng hệ thống) và đăng nhập bằng tài khoản Claude của riêng mình. Với CCDC, người ca sau có thể làm tiếp công việc Code mà ca trước để lại.
- **Một người, nhiều tài khoản.** Nếu bạn chuyển qua lại giữa tài khoản cá nhân và tài khoản công việc, hoặc giữa các tổ chức, trên cùng một máy tính, thì dù đăng nhập bằng tài khoản nào, các phiên Code của bạn vẫn ở đó.

Mọi phiên Code trên máy đều hiển thị với mọi tài khoản đăng nhập trên máy đó. Đừng dùng CCDC ở nơi mọi người không được xem phiên của nhau. Nếu dùng tài khoản công ty (tổ chức), hãy kiểm tra quy định của công ty trước: CCDC cũng sao chép các phiên của tài khoản công việc vào thư mục của tài khoản cá nhân, và ngược lại.

## Vấn đề CCDC giải quyết

Ứng dụng Claude cho máy tính lưu **danh sách** phiên Code trong một thư mục riêng cho từng tài khoản và tổ chức:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Thư mục dữ liệu Claude | Nhật ký Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (hoặc `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Các tổ chức chạy Claude qua nhà cung cấp đám mây của riêng mình dùng `Claude-3p` thay cho `Claude`; CCDC cũng tìm ở đó.)

Bản thân các **cuộc trò chuyện** nằm trong `~/.claude/projects/.../<cliSessionId>.jsonl`, dùng chung cho mọi tài khoản. Vì vậy, sau khi ai đó đăng nhập bằng tài khoản khác, danh sách phiên Code trông như trống dù các cuộc trò chuyện vẫn còn trên đĩa.

## Cách hoạt động

Ứng dụng chỉ đọc thư mục phiên **khi khởi động và khi có người đăng nhập** (hai lần, trong vòng khoảng hai giây). Không có cách nào để buộc nó chỉ đọc lại danh sách vào lúc khác. Nhật ký của chính ứng dụng ghi lại cả hai sự kiện:

- đăng xuất: `[account] Login-state transition (loggedOut: false → true ...)`
- đăng nhập: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC theo dõi nhật ký đó:

1. **Đăng xuất** → CCDC sao chép bản mới nhất của mọi phiên vào thư mục của mọi tài khoản/tổ chức, nên tài khoản đăng nhập tiếp theo đã thấy sẵn các phiên trong thư mục của chính mình. Chỉ những phiên còn thiếu hoặc đã cũ trong một thư mục mới được sao chép.
2. **Đăng nhập** → theo mặc định, các bản sao được giữ nguyên. Nếu bật "Xóa bản sao sau khi đăng nhập", CCDC đợi ứng dụng đọc xong danh sách (6 giây sau dòng đăng nhập cuối cùng) rồi xóa các bản sao ở mọi nơi, trừ thư mục của tài khoản đang đăng nhập.

Phiên bị xóa trong ứng dụng cũng bị xóa khỏi các thư mục khác, nên phiên đó không xuất hiện lại ở tài khoản khác.

Trước khi sao chép, CCDC kiểm tra dung lượng đĩa trống. Nếu không đủ chỗ cho các bản sao cộng thêm 512 MB, CCDC sẽ không sao chép gì và hiện cảnh báo.

### Vì sao mặc định giữ bản sao

Giữ bản sao là lựa chọn ít gây hao mòn SSD nhất:

- **Giữ bản sao (mặc định)**: lần đăng xuất đầu tiên sao chép tất cả các phiên một lần. Sau đó, mỗi lần đăng xuất chỉ ghi các phiên đã thay đổi kể từ lần trước, thường là vài tệp và vài MB, và xong gần như ngay lập tức.
- **Xóa bản sao sau khi đăng nhập**: mỗi lần đăng nhập xóa các bản sao, nên mỗi lần đăng xuất lại phải ghi toàn bộ các phiên một lần nữa. Với khoảng 300 phiên và 10 thư mục tài khoản/tổ chức, đó là khoảng 600 MB được ghi ở mỗi lần giao ca, cộng với việc xóa khoảng 3.000 tệp sau đó.

Đổi lại, các bản sao chiếm dung lượng đĩa: chúng vẫn được giữ lại và chiếm khoảng (tổng dung lượng các tệp phiên × số thư mục tài khoản/tổ chức).

### Vì sao không dùng điểm nối thư mục hay liên kết cứng

Nếu mọi thư mục tài khoản cùng trỏ tới một thư mục thật thì không cần sao chép, nhưng ứng dụng Claude cố ý chặn cả hai cách này:

- **Điểm nối thư mục (junction) và liên kết tượng trưng (symbolic link)**: trước mỗi lần lưu, ứng dụng kiểm tra thư mục phiên có phải thư mục thật không và từ chối lưu nếu đó là liên kết (`Refusing non-directory at private dir path (symlink/file plant)`). Khi thử với ứng dụng thật, danh sách vẫn đọc được qua điểm nối, nhưng mọi lần lưu phiên và tác vụ đã lên lịch đều thất bại, kể cả với phiên vừa tạo.
- **Liên kết cứng (hard link)**: ứng dụng coi tệp có nhiều hơn một liên kết là không an toàn. Nó từ chối ghi vào những tệp đó và cảnh báo khi đọc. Ngoài ra, nó lưu bằng cách ghi một tệp tạm rồi đổi tên, điều này sẽ làm đứt liên kết ở mỗi lần lưu.

Các kiểm tra này nhằm chặn tệp hoặc thư mục bị cài từ bên ngoài, nên CCDC không lách qua chúng. CCDC sao chép các tệp bình thường, giống loại tệp mà chính ứng dụng ghi ra.

## Cài đặt

Tải tệp dành cho hệ điều hành của bạn từ [Releases](../../releases). Các tệp này không được ký mã (xem [bên dưới](#về-cảnh-báo-bảo-mật)).

**Windows** — chạy `CCDC-<version>-windows-x64.msi`. Không cần quyền quản trị.

- Cài vào `%LOCALAPPDATA%\Programs\CCDC`, tạo lối tắt `Continuous Claude Desktop Code` trên màn hình nền và trong menu Start, rồi khởi động CCDC trong khay hệ thống.
- Tự khởi động cùng máy tính; để tắt, dùng mục "Khởi động cùng máy tính" trong menu khay.
- Cập nhật: chạy tệp `.msi` mới hơn. Trình cài đặt sẽ đóng CCDC, thay bản mới rồi khởi động lại; thiết lập của bạn được giữ nguyên.
- Gỡ cài đặt trong Cài đặt > Ứng dụng > Ứng dụng đã cài đặt > "Continuous Claude Desktop Code (CCDC)".
- Nếu đang dùng các script 0.3: chỉ cần cài tệp `.msi`. Ở lần khởi động đầu tiên, CCDC sẽ dừng phiên bản cũ, giữ lại thiết lập và tên tài khoản của nó, rồi gỡ bỏ nó.

**macOS** — mở `CCDC-<version>-macos-universal.dmg`, kéo **CCDC** vào thư mục Applications rồi mở ứng dụng. Chữ **C** sẽ xuất hiện trên thanh menu. CCDC tự khởi động cùng máy tính, trừ khi bạn tắt tùy chọn này trong menu của nó. Chạy được trên cả Apple Silicon và Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Sau đó khởi động **Continuous Claude Desktop Code** từ menu ứng dụng. CCDC tự khởi động cùng máy tính, trừ khi bạn tắt tùy chọn này trong menu của nó. Trên GNOME không có tiện ích mở rộng AppIndicator, biểu tượng khay không thể hiển thị (Ubuntu và Linux Mint có sẵn tiện ích này).

Gỡ cài đặt vẫn giữ nguyên các tệp phiên của Claude, thiết lập của CCDC và thư mục sao lưu.

### Về cảnh báo bảo mật

Muốn ký mã thì cần một chứng chỉ trả phí gắn với một cá nhân hoặc công ty, còn CCDC là một dự án nhỏ, miễn phí, nên các tệp tải về không được ký. Hệ điều hành sẽ cảnh báo bạn ở lần chạy đầu tiên; cảnh báo đó chỉ có nghĩa là nhà phát hành chưa được xác minh, chứ không phải đã phát hiện điều gì có hại.

CCDC không chứa gì có hại. CCDC không kết nối Internet, không thu thập dữ liệu và không thay đổi gì ngoài các thư mục phiên của ứng dụng Claude và thư mục của chính nó. Toàn bộ mã nguồn nằm trong kho này, và các tệp phát hành được GitHub Actions build từ mã nguồn đó, với nhật ký công khai. Bạn có thể đối chiếu tệp tải về với `SHA256SUMS.txt` trong bản phát hành, hoặc [tự build](#build).

- **Windows**: nếu SmartScreen báo "Windows đã bảo vệ PC của bạn", hãy nhấp **Thông tin thêm**, rồi **Vẫn chạy**.
- **macOS**: nếu macOS báo không thể xác minh ứng dụng, hãy mở **Cài đặt hệ thống > Quyền riêng tư & Bảo mật**, cuộn xuống và nhấp **Vẫn mở** bên cạnh CCDC. (Hoặc chạy `xattr -dr com.apple.quarantine /Applications/CCDC.app` trong Terminal.)
- **Linux**: gói `.deb` không bị kiểm tra theo cách này.

## Ứng dụng khay hệ thống

Một chữ **C** màu cam xuất hiện trong vùng thông báo hoặc trên thanh menu (màu xám khi tắt tự động đồng bộ). Nhấp vào để mở menu:

| Mục menu | Chức năng |
| --- | --- |
| Dòng trạng thái | Cho biết CCDC có đang theo dõi đăng nhập và đăng xuất hay không |
| Tự động đồng bộ (phát hiện đăng nhập/đăng xuất) | Bật hoặc tắt việc sao chép và dọn dẹp |
| Chuyển phiên sang tài khoản ngay | Gom mọi phiên vào thư mục của một tài khoản; hỏi xác nhận nếu tài khoản đó chưa đăng nhập |
| Xóa bản sao sau khi đăng nhập | Mặc định tắt (bản sao được giữ; xem ở trên) |
| Tốc độ sao chép/xóa | Nhanh (không giới hạn), Bình thường (60 MB/s) hoặc Nhẹ nhàng (15 MB/s). Ở mức Bình thường và Nhẹ nhàng, việc xóa được giới hạn lần lượt ở 500 và 100 tệp mỗi giây |
| Dòng ước tính | Cho biết lần sao chép khi đăng xuất sắp tới sẽ mất bao lâu; nếu bật xóa bản sao, cho biết cả thời gian dọn dẹp sau khi đăng nhập |
| Khởi động cùng máy tính | Khởi động CCDC khi bạn đăng nhập vào máy tính |
| Language | Chọn ngôn ngữ hiển thị. Mục này luôn ghi bằng tiếng Anh; mỗi ngôn ngữ trong menu con được viết bằng chính ngôn ngữ đó |
| Xem nhật ký | Mở `watch.log` của CCDC |
| Đặt tên cho tài khoản (ccdc.json) | Mở tệp thiết lập để đặt tên dễ nhận biết cho các tài khoản |
| Thoát | Gỡ biểu tượng; nếu đang sao chép thì đợi sao chép xong |

Thông báo hiện ra khi bắt đầu sao chép (kèm thời gian ước tính), khi sao chép xong, khi phát hiện đăng nhập và khi dọn dẹp xong. Khi đặt tốc độ chậm, việc sao chép mất nhiều thời gian hơn, nên người tiếp theo cần đợi thông báo "Đã sao chép xong", hoặc khoảng thời gian ước tính, rồi mới đăng nhập.

Thời gian ước tính được tính từ trạng thái hiện tại của các thư mục và từ tốc độ sao chép, xóa đo được trên máy này (lưu trong `state.json` ở thư mục của CCDC).

Nếu không thấy biểu tượng trên Windows 11, hãy xem các biểu tượng ẩn (`^`) trên thanh tác vụ. Để biểu tượng luôn hiển thị, hãy bật CCDC trong Cài đặt > Cá nhân hóa > Thanh tác vụ > Các biểu tượng khay hệ thống khác.

CCDC là một chương trình gốc duy nhất, dung lượng khoảng 1 MB. Trong lúc chờ, cứ mỗi nửa giây CCDC chỉ đọc phần được thêm vào nhật ký Claude kể từ lần kiểm tra trước, và bản thân nó chỉ dùng khoảng 4 MB bộ nhớ.

## Thiết lập

Thiết lập được lưu trong `ccdc.json` ở thư mục của CCDC:

| | Thư mục của CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Những gì bạn thay đổi trong menu khay hệ thống sẽ được lưu vào tệp này và áp dụng từ lần sao chép hoặc dọn dẹp kế tiếp, không cần khởi động lại. `watch.log` và `state.json` cũng nằm trong thư mục này.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Khóa | Giá trị |
| --- | --- |
| `keepCopies` | `true` (mặc định): giữ bản sao. `false`: xóa bản sao sau khi đăng nhập |
| `speed` | `"fast"` (mặc định), `"normal"`, `"gentle"` |
| `language` | Để trống để theo ngôn ngữ của hệ thống, hoặc nhập mã ngôn ngữ như `"ja"` để dùng ngôn ngữ đó |
| `startAtLogin` | Được đặt qua mục "Khởi động cùng máy tính" trong menu khay hệ thống |
| `aliases` | Tên hiển thị cho các tài khoản trong menu khay hệ thống |
| `dataDir`, `appLog` | Chỉ dùng khi CCDC không tự tìm được thư mục dữ liệu hoặc nhật ký của Claude: đường dẫn đầy đủ tới chúng |

## Ngôn ngữ

Menu khay hệ thống, thông báo và cảnh báo theo ngôn ngữ hiển thị của hệ thống, hoặc ngôn ngữ chọn ở mục **Language** trong menu khay. Hỗ trợ: tiếng Anh, tiếng Hàn, tiếng Nhật, tiếng Trung (giản thể và phồn thể), tiếng Tây Ban Nha, tiếng Pháp, tiếng Đức, tiếng Ý, tiếng Bồ Đào Nha, tiếng Nga, tiếng Ukraina, tiếng Ba Lan, tiếng Hà Lan, tiếng Thổ Nhĩ Kỳ, tiếng Việt, tiếng Indonesia, tiếng Thái, tiếng Ả Rập và tiếng Hindi. Các ngôn ngữ khác hiển thị bằng tiếng Anh.

Toàn bộ văn bản giao diện nằm trong một bảng duy nhất, [`locales.csv`](locales.csv), được tích hợp sẵn vào chương trình: **mỗi hàng là một ngôn ngữ**, mỗi cột là một thông điệp. Mở bằng Excel hoặc Google Trang tính; để thêm ngôn ngữ, hãy thêm một hàng rồi [build](#build) lại CCDC.

- `lang`: thẻ ngôn ngữ (`sv`, `pt-BR`, `zh-Hant`, …). Ưu tiên hàng khớp cả ngôn ngữ và khu vực, rồi đến hàng khớp ngôn ngữ.
- `name`: tên ngôn ngữ viết bằng chính ngôn ngữ đó (`Svenska`); menu Language hiển thị tên này.
- Ký tự xuống dòng trong ô được viết là `\n`, và giữ nguyên `{0}`, `{1}` (số hoặc tên sẽ được điền vào đó).
- Ô trống sẽ hiển thị bằng tiếng Anh. Lưu tệp dưới dạng CSV UTF-8.

Đầu ra dòng lệnh và `watch.log` giữ bằng tiếng Anh.

## Dòng lệnh

Cùng một chương trình đó cũng dùng được từ terminal (trên Windows là `ccdc.exe` trong `%LOCALAPPDATA%\Programs\CCDC`; trên macOS là `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` và `--app-log <path>` dùng để chỉ định thủ công thư mục dữ liệu hoặc nhật ký của Claude.

## Quy tắc khi có bản trùng

- Khi một phiên có trong nhiều thư mục, bản có `lastActivityAt` mới nhất được chọn. Các bản sao giữ nguyên thời gian sửa đổi của bản gốc, nên bản giống hệt được nhận ra qua kích thước và thời gian mà không cần đọc nội dung.
- Khi một bản mới hơn thay thế tệp, tệp cũ được chuyển vào `ccdc-backups/<time>/` trong thư mục dữ liệu Claude.
- Các phiên có dấu `deleted_<id>` không được sao chép, và các bản sao của chúng ở thư mục khác bị xóa.
- Tài khoản/tổ chức đăng nhập lần đầu chưa có thư mục, nên lúc đăng xuất không thể sao chép vào đó. Khi đó, các phiên sẽ hiển thị sau lần đăng nhập tiếp theo của tài khoản đó hoặc khi khởi động lại ứng dụng.

## Lưu ý

- CCDC phụ thuộc vào cấu trúc tệp bên trong và câu chữ nhật ký của ứng dụng Claude cho máy tính; bản cập nhật ứng dụng có thể làm thay đổi cách hoạt động.
- Tệp phiên chứa cài đặt trình kết nối (MCP) theo từng tài khoản, nên một số trình kết nối có thể không kết nối được khi mở phiên từ tài khoản khác.
- Tác vụ đã lên lịch không được chia sẻ; mỗi thư mục tài khoản/tổ chức giữ tác vụ riêng.
- Không hỗ trợ đổi tài khoản trong Claude Code CLI. CLI vốn đã mở được mọi phiên trong `~/.claude/projects` bằng `--resume`, bất kể tài khoản nào.

## Build

CCDC được viết bằng Rust; hãy [cài Rust](https://rustup.rs), rồi chạy:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

Phiên bản của từng nền tảng được đặt trong `[package.metadata.ccdc]` ở `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) build cả ba bản ở mỗi lần push, và với tag `v*` thì đính kèm chúng vào một bản phát hành nháp.

## Giấy phép

[MIT](LICENSE)
