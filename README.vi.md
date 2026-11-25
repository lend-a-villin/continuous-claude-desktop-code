# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | **Tiếng Việt** | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Ứng dụng khay hệ thống Windows giúp các phiên **Code** cục bộ của ứng dụng Claude cho máy tính luôn dùng được với bất kỳ tài khoản Claude nào đăng nhập trên máy.

Công cụ không chính thức; không liên kết với Anthropic và không được Anthropic chứng thực.

## Dành cho ai

CCDC được thiết kế cho **nơi làm việc hoạt động 24 giờ theo ba ca**. Mỗi ca, một người khác ngồi vào cùng một chỗ (một máy tính, một người dùng Windows) và đăng nhập vào ứng dụng Claude cho máy tính bằng tài khoản Claude của riêng mình. Với CCDC, người ca sau chỉ cần đăng nhập là có thể làm tiếp công việc Code mà ca trước để lại.

Điều này có nghĩa là bất kỳ ai đăng nhập vào máy cũng đều thấy được mọi phiên Code trên máy đó. Đừng dùng CCDC ở nơi mọi người không được xem phiên của nhau. Nếu dùng tài khoản công ty (tổ chức), hãy kiểm tra cả quy định của công ty.

## Vấn đề CCDC giải quyết

Ứng dụng Claude cho máy tính lưu **danh sách** phiên Code trong một thư mục riêng cho từng tài khoản và tổ chức:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

Bản thân các **cuộc trò chuyện** nằm trong `~/.claude/projects\...\<cliSessionId>.jsonl`, dùng chung cho mọi tài khoản. Vì vậy, sau khi ai đó đăng nhập bằng tài khoản khác, danh sách phiên Code trông như trống dù các cuộc trò chuyện vẫn còn trên đĩa.

## Cách hoạt động

Ứng dụng chỉ đọc thư mục phiên **khi khởi động và khi có người đăng nhập** (hai lần, trong vòng khoảng hai giây). Không có cách nào để buộc nó chỉ đọc lại danh sách vào lúc khác. Nhật ký của chính ứng dụng, `%LOCALAPPDATA%\Claude\Logs\main.log`, ghi lại cả việc đăng xuất lẫn đăng nhập:

- đăng xuất: `[account] Login-state transition (loggedOut: false → true ...)`
- đăng nhập: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC theo dõi nhật ký đó và hoạt động như sau:

1. **Đăng xuất** → CCDC sao chép bản mới nhất của mọi phiên vào thư mục của mọi tài khoản/tổ chức, nên dù tài khoản nào đăng nhập tiếp theo, các phiên cũng đã có sẵn trong thư mục của tài khoản đó. Chỉ những phiên còn thiếu hoặc đã cũ trong một thư mục mới được sao chép.
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

- **Điểm nối thư mục (junction) và liên kết tượng trưng (symbolic link)**: trước mỗi lần lưu, ứng dụng kiểm tra thư mục phiên có phải thư mục thật không và từ chối lưu nếu đó là liên kết (`Refusing non-directory at private dir path (symlink/file plant)`). Khi thử với ứng dụng thật, danh sách vẫn đọc được qua điểm nối, nhưng mọi lần lưu phiên và tác vụ đã lên lịch đều thất bại, kể cả với phiên hoàn toàn mới.
- **Liên kết cứng (hard link)**: ứng dụng coi tệp có nhiều hơn một liên kết là không an toàn. Nó từ chối ghi vào những tệp đó và cảnh báo khi đọc. Ngoài ra, nó lưu bằng cách ghi một tệp tạm rồi đổi tên, điều này sẽ làm đứt liên kết ở mỗi lần lưu.

Các kiểm tra này nhằm chặn tệp hoặc thư mục bị cài từ bên ngoài, nên CCDC không lách qua chúng. CCDC sao chép các tệp bình thường, giống loại tệp mà chính ứng dụng ghi ra.

## Yêu cầu

- Windows 10 hoặc 11, có ứng dụng Claude cho máy tính (bản cài MSIX)
- [Node.js](https://nodejs.org) 18 trở lên

## Cài đặt

Tải trình cài đặt từ [Releases](../../releases), giải nén rồi chạy `setup.cmd`. Không cần quyền quản trị.

- Cài vào `%LOCALAPPDATA%\Programs\CCDC`.
- Tạo lối tắt `Continuous Claude Desktop Code` trên màn hình nền và trong menu Start.
- Xuất hiện trong Cài đặt > Ứng dụng > Ứng dụng đã cài đặt với tên "Continuous Claude Desktop Code (CCDC)"; bạn có thể gỡ cài đặt tại đó.
- Khởi động trong khay hệ thống ngay sau khi cài.
- Để CCDC tự chạy khi bạn đăng nhập Windows, hãy cài bằng `setup.cmd /startup`; để tắt tự chạy, dùng `setup.cmd /nostartup`. Cài lại mà không có tùy chọn nào thì thiết lập trước đó được giữ nguyên.
- Cài lại sẽ dừng CCDC nếu đang chạy và thay thế các tệp của nó; thiết lập của bạn (`ccdc.json`) được giữ lại.
- Gỡ cài đặt vẫn giữ nguyên các tệp phiên của Claude và thư mục sao lưu của CCDC.

Để chạy trực tiếp từ kho này mà không cài, hãy chạy `ccdc-tray.vbs` hoặc dùng dòng lệnh bên dưới.

## Ứng dụng khay hệ thống

Một biểu tượng chữ **C** màu cam xuất hiện trong vùng thông báo (màu xám khi tắt tự động đồng bộ). Nhấp vào để mở menu:

| Mục menu | Chức năng |
| --- | --- |
| Dòng trạng thái | Cho biết CCDC có đang theo dõi đăng nhập và đăng xuất hay không |
| Tự động đồng bộ (phát hiện đăng nhập/đăng xuất) | Bật hoặc tắt việc sao chép và dọn dẹp |
| Chuyển phiên sang tài khoản ngay | Gom mọi phiên vào thư mục của một tài khoản; hỏi xác nhận nếu tài khoản đó chưa đăng nhập |
| Xóa bản sao sau khi đăng nhập | Mặc định tắt (bản sao được giữ; xem ở trên) |
| Tốc độ sao chép/xóa | Nhanh (không giới hạn), Bình thường (60 MB/s) hoặc Nhẹ nhàng (15 MB/s). Ở mức Bình thường và Nhẹ nhàng, việc xóa được giới hạn lần lượt ở 500 và 100 tệp mỗi giây |
| Dòng ước tính | Cho biết lần sao chép khi đăng xuất sắp tới sẽ mất bao lâu; nếu bật xóa bản sao, cho biết cả thời gian dọn dẹp sau khi đăng nhập |
| Language | Chọn ngôn ngữ hiển thị. Mục này luôn ghi bằng tiếng Anh; mỗi ngôn ngữ trong menu con được viết bằng chính ngôn ngữ đó |
| Xem nhật ký | Mở `%LOCALAPPDATA%\CCDC\watch.log` |
| Đặt tên cho tài khoản (ccdc.json) | Mở tệp thiết lập để đặt tên dễ nhận biết cho các tài khoản |
| Thoát | Gỡ biểu tượng; nếu đang sao chép thì đợi sao chép xong |

Thông báo hiện ra khi bắt đầu sao chép (kèm thời gian ước tính), khi sao chép xong, khi phát hiện đăng nhập và khi dọn dẹp xong. Khi đặt tốc độ chậm, việc sao chép mất nhiều thời gian hơn, nên người tiếp theo cần đợi thông báo "Đã sao chép xong", hoặc khoảng thời gian ước tính, rồi mới đăng nhập.

Thời gian ước tính được tính từ trạng thái hiện tại của các thư mục và từ tốc độ sao chép, xóa đo được trên máy này (lưu trong `%LOCALAPPDATA%\CCDC\state.json`).

Nếu không thấy biểu tượng trên Windows 11, hãy xem các biểu tượng ẩn (`^`) trên thanh tác vụ. Để luôn hiển thị, bật Windows PowerShell trong Cài đặt > Cá nhân hóa > Thanh tác vụ > Các biểu tượng khay hệ thống khác.

### Mức dùng bộ nhớ

Ứng dụng khay tự theo dõi nhật ký của ứng dụng Claude, chỉ đọc phần được thêm vào kể từ lần kiểm tra trước. Node.js chỉ chạy trong lúc sao chép hoặc dọn dẹp rồi thoát, nên giữa các ca chỉ có ứng dụng khay chạy. Khi nhàn rỗi, ứng dụng khay trả bộ nhớ không dùng đến về cho Windows khoảng mỗi phút một lần (lúc đó chỉ dùng khoảng 15 MB).

## Thiết lập

Thiết lập được lưu trong `ccdc.json` ở thư mục chương trình. Những gì bạn thay đổi trong menu khay hệ thống sẽ được lưu vào tệp này và áp dụng từ lần sao chép hoặc dọn dẹp kế tiếp, không cần khởi động lại.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<8 ký tự đầu của ID tài khoản>": "<tên>" }
}
```

| Khóa | Giá trị |
| --- | --- |
| `keepCopies` | `true` (mặc định): giữ bản sao. `false`: xóa bản sao sau khi đăng nhập |
| `speed` | `"fast"` (mặc định), `"normal"`, `"gentle"` |
| `language` | Để trống để theo ngôn ngữ của Windows, hoặc nhập mã ngôn ngữ như `"ja"` để dùng ngôn ngữ đó |
| `aliases` | Tên hiển thị cho các tài khoản trong menu khay hệ thống |

## Ngôn ngữ

Menu khay hệ thống, thông báo, cảnh báo và cửa sổ cài đặt/gỡ cài đặt theo ngôn ngữ hiển thị của Windows, hoặc ngôn ngữ chọn ở mục **Language** trong menu khay. Hỗ trợ: tiếng Anh, tiếng Hàn, tiếng Nhật, tiếng Trung (giản thể và phồn thể), tiếng Tây Ban Nha, tiếng Pháp, tiếng Đức, tiếng Ý, tiếng Bồ Đào Nha, tiếng Nga, tiếng Ukraina, tiếng Ba Lan, tiếng Hà Lan, tiếng Thổ Nhĩ Kỳ, tiếng Việt, tiếng Indonesia, tiếng Thái, tiếng Ả Rập và tiếng Hindi. Các ngôn ngữ khác hiển thị bằng tiếng Anh.

Toàn bộ văn bản giao diện nằm trong một bảng duy nhất, [`locales.csv`](locales.csv): **mỗi hàng là một ngôn ngữ**, mỗi cột là một thông điệp. Mở bằng Excel hoặc Google Trang tính; để thêm ngôn ngữ, hãy thêm một hàng.

- `lang`: tên văn hóa của Windows (`sv`, `pt-BR`, `zh-Hant`, …). Ưu tiên hàng khớp cả ngôn ngữ và khu vực, rồi đến hàng khớp ngôn ngữ.
- `name`: tên ngôn ngữ viết bằng chính ngôn ngữ đó (`Svenska`); menu Language hiển thị tên này.
- Ký tự xuống dòng trong ô được viết là `\n`, và giữ nguyên `{0}`, `{1}` (số hoặc tên sẽ được điền vào đó).
- Ô trống sẽ hiển thị bằng tiếng Anh. Lưu tệp dưới dạng CSV UTF-8.

Đầu ra dòng lệnh và `watch.log` giữ bằng tiếng Anh vì ứng dụng khay hệ thống đọc chúng.

## Dòng lệnh

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Khi cần, ứng dụng khay hệ thống chạy các tác vụ một lần sau: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` và `node ccdc.mjs startup` (để bắt kịp trạng thái hiện tại của Claude khi ứng dụng khay khởi động hoặc khi bật lại tự động đồng bộ).

## Quy tắc khi có bản trùng

- Khi một phiên có trong nhiều thư mục, bản có `lastActivityAt` mới nhất được chọn. Các bản sao giữ nguyên thời gian sửa đổi của bản gốc, nên bản giống hệt được nhận ra qua kích thước và thời gian mà không cần đọc nội dung.
- Khi một bản mới hơn thay thế tệp, tệp cũ được chuyển vào `ccdc-backups\<thời gian>\` trong thư mục dữ liệu của ứng dụng.
- Các phiên có dấu `deleted_<id>` không được sao chép, và các bản sao của chúng ở thư mục khác bị xóa.
- Tài khoản/tổ chức đăng nhập lần đầu chưa có thư mục, nên lúc đăng xuất không thể sao chép vào đó. Khi đó, các phiên sẽ hiển thị sau lần đăng nhập tiếp theo của tài khoản đó hoặc khi khởi động lại ứng dụng.

## Lưu ý

- CCDC phụ thuộc vào cấu trúc tệp bên trong và câu chữ nhật ký của ứng dụng Claude cho máy tính; bản cập nhật ứng dụng có thể làm thay đổi cách hoạt động.
- Tệp phiên chứa cài đặt trình kết nối (MCP) theo từng tài khoản, nên một số trình kết nối có thể không kết nối được khi mở phiên từ tài khoản khác.
- Tác vụ đã lên lịch không được chia sẻ; mỗi thư mục tài khoản/tổ chức giữ tác vụ riêng.
- Không hỗ trợ đổi tài khoản trong Claude Code CLI. CLI vốn đã mở được mọi phiên trong `~/.claude/projects` bằng `--resume`, bất kể tài khoản nào.

## Giấy phép

[MIT](LICENSE)
