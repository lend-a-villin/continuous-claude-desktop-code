# Continuous Claude Desktop Code (CCDC)

[English](README.md) | **한국어** | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Claude 계정을 바꿔도 Claude 데스크톱 앱의 **Code** 세션이 사라지지 않게 해 주는 트레이 앱입니다.

Claude 데스크톱 앱은 계정·조직마다 Code 세션 목록을 따로 관리합니다. 그래서 다른 계정으로 로그인하면, 대화 기록은 컴퓨터에 그대로 남아 있는데도 세션 목록이 비어 보입니다. CCDC가 세션을 모든 계정으로 복사해 두므로, 다음에 어떤 계정으로 로그인하든 모든 세션이 보입니다.

비공식 프로그램으로, Anthropic과 관련이 없으며 Anthropic의 승인을 받지 않았습니다.

**[최신 버전 다운로드](../../releases/latest)** · Windows 10/11 · macOS 11 이상 · Ubuntu/Debian/Linux Mint · 따로 설치할 것 없음

| 플랫폼 | 버전 | 상태 |
| --- | --- | --- |
| Windows | 1.0 | 새로 나왔습니다. Windows에서 실제 Claude 앱으로 시험해 설명대로 동작하는 것을 확인했습니다(0.3 스크립트 버전의 후속). |
| macOS | 0.1 | 새로 나왔습니다. 아직 실제 Mac의 Claude 앱으로는 시험하지 않았습니다. |
| Linux | 0.1 | 새로 나왔습니다. Debian 12, Ubuntu 24.04, Linux Mint 22에서 설치를 시험했지만, 실제 Claude 앱으로는 아직 시험하지 않았습니다. |

## 사용 대상

- **공용 컴퓨터와 교대 근무.** 24시간 3교대로 운영되는 직장에서는 교대 조마다 다른 사람이 같은 자리(컴퓨터 한 대, 시스템 사용자 한 명)에 앉아 각자 자기 Claude 계정으로 로그인합니다. CCDC를 사용하면 다음 근무조가 앞 근무조가 하던 Code 작업을 그대로 이어받을 수 있습니다.
- **한 사람이 여러 계정을 쓰는 경우.** 같은 컴퓨터에서 개인 계정과 회사 계정을, 또는 여러 조직을 오가며 쓴다면 어느 계정으로 로그인하든 같은 Code 세션을 이어서 사용할 수 있습니다.

CCDC를 사용하면 이 컴퓨터에 로그인하는 모든 계정에서 이 컴퓨터의 모든 Code 세션이 보입니다. 서로의 세션을 보면 안 되는 환경에서는 CCDC를 사용하지 마세요. 회사(조직) 계정을 사용한다면 먼저 사내 정책을 확인하세요. CCDC는 회사 계정의 세션을 개인 계정 폴더로도 복사하며, 그 반대도 마찬가지입니다.

## 해결하는 문제

데스크톱 앱은 Code 세션 **목록**을 계정·조직마다 별도의 폴더에 저장합니다.

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Claude 데이터 폴더 | Claude 로그 |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (또는 `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(조직이 자체 클라우드 서비스를 통해 Claude를 쓰는 경우에는 `Claude` 대신 `Claude-3p`라는 이름을 쓰며, CCDC는 이 폴더도 찾아봅니다.)

반면 **대화 내용**은 모든 계정이 함께 쓰는 `~/.claude/projects/.../<cliSessionId>.jsonl`에 저장됩니다. 그래서 다른 계정으로 로그인하면 대화 내용은 디스크에 그대로 있는데도 Code 세션 목록이 비어 보입니다.

## 동작 방식

앱은 **시작할 때와 로그인할 때만** 세션 폴더를 읽습니다(로그인할 때는 약 2초 사이에 두 번). 그 밖의 시점에 목록만 다시 읽게 할 방법은 없습니다. 로그아웃과 로그인은 모두 앱 자체 로그에 다음과 같이 기록됩니다.

- 로그아웃: `[account] Login-state transition (loggedOut: false → true ...)`
- 로그인: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC는 이 로그를 지켜보다가 다음과 같이 동작합니다.

1. **로그아웃할 때**: 각 세션의 최신본을 모든 계정·조직 폴더에 복사합니다. 그래서 다음에 어느 계정으로 로그인하든 그 계정의 폴더에 세션이 이미 들어 있습니다. 이때 폴더에 없거나 오래된 세션만 복사합니다.
2. **로그인할 때**: 기본적으로 사본은 그대로 둡니다. "로그인 후 사본 삭제"를 켜 두었다면 앱이 목록을 다 읽을 때까지(마지막 로그인 기록 뒤 6초) 기다렸다가, 로그인한 계정의 폴더를 뺀 나머지 폴더에서 사본을 삭제합니다.

앱에서 삭제한 세션은 다른 폴더에서도 지워지므로, 다른 계정으로 로그인해도 다시 나타나지 않습니다.

CCDC는 복사하기 전에 디스크 여유 공간을 확인합니다. 복사에 필요한 공간에 512MB를 더한 만큼이 남아 있지 않으면, 아무것도 복사하지 않고 경고를 표시합니다.

### 기본적으로 사본을 남기는 이유

사본을 남기는 방식이 SSD에 가장 부담이 적습니다.

- **사본 유지(기본값)**: 처음 로그아웃할 때 모든 세션을 한 번 복사합니다. 그 뒤로는 로그아웃할 때마다 지난번 이후 바뀐 세션만 기록하므로, 보통 파일 몇 개(몇 MB)로 거의 바로 끝납니다.
- **로그인 후 사본 삭제**: 로그인할 때마다 사본을 지우므로, 로그아웃할 때마다 모든 세션을 다시 기록해야 합니다. 세션이 300개쯤이고 계정·조직 폴더가 10개라면, 교대할 때마다 약 600MB를 기록하고 그 뒤에 파일 약 3,000개를 지웁니다.

그 대가는 디스크 공간입니다. 사본이 계속 남으므로 (세션 파일 크기 합계 × 계정·조직 폴더 수)만큼의 공간을 차지합니다.

### 정션이나 하드링크를 사용하지 않는 이유

모든 계정 폴더가 실제 폴더 하나를 가리키게 하면 복사할 필요가 없겠지만, Claude 앱은 그렇게 하는 두 가지 방법을 모두 의도적으로 막고 있습니다.

- **정션·심볼릭 링크**: 앱은 저장할 때마다 세션 폴더가 실제 폴더인지 확인하고, 링크이면 저장을 거부합니다(`Refusing non-directory at private dir path (symlink/file plant)`). 실제 앱으로 시험해 보니 정션을 거쳐 목록은 읽혔지만, 새로 만든 세션까지 포함해 모든 세션과 예약 작업을 저장하지 못했습니다.
- **하드링크**: 앱은 링크가 둘 이상인 파일을 안전하지 않은 파일로 봅니다. 그런 파일에는 쓰기를 거부하고, 읽을 때는 경고를 남깁니다. 또 앱은 임시 파일에 쓴 뒤 이름을 바꾸는 방식으로 저장하므로, 저장할 때마다 링크가 끊어질 수밖에 없습니다.

이 검사들은 밖에서 심어 놓은 파일이나 폴더를 막기 위한 장치이므로, CCDC는 이를 우회하지 않습니다. 대신 앱이 직접 만드는 것과 똑같은 일반 파일을 복사합니다.

## 설치

[Releases](../../releases)에서 사용하는 시스템에 맞는 파일을 내려받으세요. 설치 파일에는 코드 서명이 없습니다([아래](#보안-경고에-대하여) 참고).

**Windows** — `CCDC-<version>-windows-x64.msi`를 실행하세요. 관리자 권한은 필요하지 않습니다.

- `%LOCALAPPDATA%\Programs\CCDC`에 설치되고, 바탕화면과 시작 메뉴에 `Continuous Claude Desktop Code` 바로가기가 만들어지며, CCDC가 트레이에서 바로 실행됩니다.
- 컴퓨터를 켜면 자동으로 실행됩니다. 원하지 않으면 트레이 메뉴의 "컴퓨터 시작 시 자동 실행"을 끄세요.
- 업데이트: 새 버전의 `.msi`를 실행하면 됩니다. 실행 중인 CCDC를 닫고 교체한 뒤 다시 실행하며, 설정은 그대로 유지됩니다.
- 제거는 설정 > 앱 > 설치된 앱 > "Continuous Claude Desktop Code (CCDC)"에서 합니다.
- 0.3 스크립트 버전을 쓰고 있었다면 `.msi`만 설치하세요. 처음 실행될 때 이전 버전을 종료하고, 설정과 계정 이름을 이어받은 뒤 이전 버전을 삭제합니다.

**macOS** — `CCDC-<version>-macos-universal.dmg`를 열고 **CCDC**를 응용 프로그램(Applications) 폴더로 끌어다 놓은 뒤 실행하세요. 메뉴 막대에 **C**가 나타납니다. 메뉴에서 끄지 않는 한 컴퓨터를 켜면 자동으로 실행됩니다. Apple Silicon과 Intel Mac에서 모두 실행됩니다.

**Linux** (Ubuntu 22.04 이상, Debian 12 이상, Linux Mint 21 이상, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

설치한 뒤 응용 프로그램 메뉴에서 **Continuous Claude Desktop Code**를 실행하세요. 메뉴에서 끄지 않는 한 컴퓨터를 켜면 자동으로 실행됩니다. AppIndicator 확장이 없는 GNOME에서는 트레이 아이콘이 표시되지 않습니다(Ubuntu와 Linux Mint에는 기본으로 들어 있습니다).

제거해도 Claude 세션 파일, CCDC 설정, 백업 폴더는 그대로 남습니다.

### 보안 경고에 대하여

코드 서명을 하려면 개인이나 회사 명의로 유료 인증서를 발급받아야 하는데, CCDC는 작은 무료 프로젝트라서 설치 파일에 서명하지 않았습니다. 그래서 처음 실행할 때 시스템이 경고를 표시합니다. 이 경고는 배포자를 확인할 수 없다는 뜻일 뿐, 해로운 것이 발견되었다는 뜻이 아닙니다.

CCDC에는 해로운 코드가 없습니다. 인터넷에 연결하지 않고, 데이터를 수집하지 않으며, Claude 앱의 세션 폴더와 CCDC 자신의 폴더 밖에서는 아무것도 바꾸지 않습니다. 모든 코드는 이 저장소에 공개되어 있고, 릴리스 파일은 이 코드로 GitHub Actions에서 빌드되며 빌드 기록도 공개되어 있습니다. 내려받은 파일은 릴리스의 `SHA256SUMS.txt`로 확인하거나, [직접 빌드](#직접-빌드하기)할 수도 있습니다.

- **Windows**: SmartScreen이 "Windows의 PC 보호" 창을 띄우면 **추가 정보**를 누른 뒤 **실행**을 누르세요.
- **macOS**: 앱을 확인할 수 없다는 메시지가 나오면 **시스템 설정 > 개인정보 보호 및 보안**에서 아래로 내려 CCDC 옆의 **그래도 열기**를 누르세요. (또는 터미널에서 `xattr -dr com.apple.quarantine /Applications/CCDC.app`을 실행해도 됩니다.)
- **Linux**: `.deb` 패키지는 이런 방식으로 검사하지 않습니다.

## 트레이 앱

알림 영역이나 메뉴 막대에 주황색 **C** 아이콘이 표시됩니다(자동 동기화가 꺼져 있으면 회색). 아이콘을 클릭하면 메뉴가 열립니다.

| 메뉴 | 동작 |
| --- | --- |
| 상태 줄 | CCDC가 로그인·로그아웃을 감시하고 있는지 보여 줍니다. |
| 자동 동기화 (로그인/로그아웃 감지) | 복사와 정리를 켜거나 끕니다. |
| 계정으로 지금 옮기기 | 모든 세션을 한 계정의 폴더로 모읍니다. 그 계정이 로그인한 상태가 아니면 먼저 묻습니다. |
| 로그인 후 사본 삭제 | 기본값은 꺼짐입니다(사본을 남김, 위 설명 참고). |
| 복사·삭제 속도 | 빠르게 (제한 없음), 보통 (60MB/s), 천천히 (15MB/s) 중에서 고릅니다. 보통과 천천히에서는 삭제도 각각 초당 500개, 100개로 제한됩니다. |
| 예상 시간 줄 | 다음 로그아웃 때 복사에 걸릴 시간을 보여 줍니다. 사본을 삭제하도록 설정했다면 로그인 후 정리 시간도 함께 보여 줍니다. |
| 컴퓨터 시작 시 자동 실행 | 컴퓨터에 로그인할 때 CCDC를 자동으로 실행합니다. |
| Language | 표시 언어를 고릅니다. 이 항목은 항상 영어로 표시되고, 하위 메뉴의 각 언어는 그 언어로 적혀 있습니다. |
| 로그 보기 | CCDC의 `watch.log`를 엽니다. |
| 계정 이름 붙이기 (ccdc.json) | 설정 파일을 열어 계정에 알아보기 쉬운 이름을 붙입니다. |
| 종료 | 아이콘을 없애고 종료합니다. 복사 중이면 끝날 때까지 기다립니다. |

복제를 시작할 때(예상 시간 포함), 복제가 끝났을 때, 로그인을 감지했을 때, 정리가 끝났을 때 알림이 표시됩니다. 속도를 느리게 설정할수록 복사가 오래 걸리므로, 다음 사람은 "복제 완료" 알림이 뜰 때까지, 또는 예상 시간만큼 기다렸다가 로그인하세요.

예상 시간은 폴더의 현재 상태와 이 컴퓨터에서 측정한 복사·삭제 속도(CCDC 폴더의 `state.json`에 저장)를 바탕으로 계산합니다.

Windows 11에서 아이콘이 보이지 않으면 작업 표시줄의 `^`(숨겨진 아이콘)를 확인하세요. 아이콘이 항상 보이게 하려면 설정 > 개인 설정 > 작업 표시줄 > 기타 시스템 트레이 아이콘에서 CCDC를 켜세요.

CCDC는 약 1MB 크기의 네이티브 프로그램 하나입니다. 대기 중에는 0.5초마다 Claude 로그에서 지난번 이후 추가된 부분만 읽으며, 자체적으로 쓰는 메모리는 약 4MB입니다.

## 설정

설정은 CCDC 폴더의 `ccdc.json`에 저장됩니다.

| | CCDC 폴더 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

트레이 메뉴에서 바꾼 내용은 이 파일에 저장되고, 프로그램을 다시 시작하지 않아도 다음 복사나 정리부터 적용됩니다. `watch.log`와 `state.json`도 같은 폴더에 있습니다.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| 키 | 값 |
| --- | --- |
| `keepCopies` | `true`(기본값): 사본을 남깁니다. `false`: 로그인 후 사본을 삭제합니다. |
| `speed` | `"fast"`(기본값), `"normal"`, `"gentle"` |
| `language` | 비워 두면 시스템 표시 언어를 따릅니다. `"ja"` 같은 언어 코드를 넣으면 그 언어로 표시됩니다. |
| `startAtLogin` | 트레이 메뉴의 "컴퓨터 시작 시 자동 실행"이 이 값을 설정합니다. |
| `aliases` | 트레이 메뉴에 표시할 계정 이름입니다. |
| `dataDir`, `appLog` | CCDC가 Claude 데이터 폴더나 로그를 스스로 찾지 못할 때만, 그 전체 경로를 적습니다. |

## 언어

트레이 메뉴, 알림, 경고 창은 시스템 표시 언어나, 트레이 메뉴의 **Language**에서 고른 언어로 표시됩니다. 지원 언어는 영어, 한국어, 일본어, 중국어(간체·번체), 스페인어, 프랑스어, 독일어, 이탈리아어, 포르투갈어, 러시아어, 우크라이나어, 폴란드어, 네덜란드어, 튀르키예어, 베트남어, 인도네시아어, 태국어, 아랍어, 힌디어입니다. 그 밖의 언어는 영어로 표시됩니다.

화면에 나오는 모든 문구는 프로그램에 들어 있는 시트 한 장([`locales.csv`](locales.csv))에서 가져옵니다. **한 줄이 한 언어**이고, 열은 문구 항목입니다. Excel이나 Google 시트에서 열어 편집할 수 있으며, 언어를 추가하려면 한 줄을 추가한 뒤 CCDC를 [빌드](#직접-빌드하기)하면 됩니다.

- `lang`: 언어 태그(`sv`, `pt-BR`, `zh-Hant` 등)를 적습니다. 언어와 지역이 모두 맞는 줄을 먼저 사용하고, 없으면 언어만 맞는 줄을 사용합니다.
- `name`: 그 언어로 적은 언어 이름(`Svenska`)입니다. Language 메뉴에 이 이름이 표시됩니다.
- 칸 안에서 줄을 바꾸려면 `\n`으로 적고, `{0}`, `{1}`(숫자나 이름이 들어갈 자리)은 그대로 두세요.
- 빈 칸은 영어 문구로 표시됩니다. 파일은 UTF-8 CSV로 저장하세요.

명령줄 출력과 `watch.log`는 영어로 고정되어 있습니다.

## 명령줄

같은 프로그램을 터미널에서도 쓸 수 있습니다(Windows에서는 `%LOCALAPPDATA%\Programs\CCDC`의 `ccdc.exe`, macOS에서는 `/Applications/CCDC.app/Contents/MacOS/ccdc`).

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>`와 `--app-log <path>`로 Claude 데이터 폴더나 로그를 직접 지정할 수 있습니다.

## 중복 처리 규칙

- 같은 세션이 여러 폴더에 있으면 `lastActivityAt`이 가장 최근인 사본을 남깁니다. 사본은 원본의 수정 시각을 그대로 가지므로, 두 사본이 같은지는 내용을 읽지 않고 크기와 시각만으로 판단합니다.
- 더 새로운 사본으로 파일을 바꿀 때는 이전 파일을 Claude 데이터 폴더의 `ccdc-backups/<time>/`으로 옮깁니다.
- `deleted_<id>` 표시가 있는 세션은 복사하지 않으며, 다른 폴더에 있는 사본은 삭제합니다.
- 처음 로그인하는 계정·조직은 아직 폴더가 없어서, 로그아웃할 때 그곳에는 복사할 수 없습니다. 이 경우 세션은 그 계정의 다음 로그인 때나 앱을 다시 시작한 뒤에 표시됩니다.

## 주의 사항

- CCDC는 Claude 데스크톱 앱의 내부 파일 구조와 로그 문구에 의존합니다. 앱이 업데이트되면 동작이 달라질 수 있습니다.
- 세션 파일에는 계정별 커넥터(MCP) 설정이 들어 있어서, 다른 계정에서 세션을 열면 일부 커넥터가 연결되지 않을 수 있습니다.
- 예약 작업은 공유되지 않으며, 계정·조직 폴더마다 따로 보관됩니다.
- Claude Code CLI의 계정 전환은 다루지 않습니다. CLI는 계정과 상관없이 `~/.claude/projects`에 있는 어떤 세션이든 이미 `--resume`으로 열 수 있습니다.

## 직접 빌드하기

CCDC는 Rust로 작성되었습니다. [Rust를 설치](https://rustup.rs)한 뒤 다음을 실행하세요.

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

플랫폼별 버전은 `Cargo.toml`의 `[package.metadata.ccdc]`에서 정합니다. GitHub Actions([`.github/workflows/build.yml`](.github/workflows/build.yml))는 push할 때마다 세 플랫폼을 모두 빌드하고, `v*` 태그를 push하면 빌드한 파일을 초안 릴리스에 첨부합니다.

## 라이선스

[MIT](LICENSE)
