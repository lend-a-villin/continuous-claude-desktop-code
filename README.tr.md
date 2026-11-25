# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | **Türkçe** | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Claude hesapları arasında geçiş yaptığınızda Claude masaüstü uygulamasındaki **Code** oturumlarınızı kaybetmemenizi sağlayan bir sistem tepsisi uygulamasıdır.

Claude masaüstü uygulaması her hesap ve kuruluş için ayrı bir Code oturum listesi tutar. Başka bir hesapla oturum açtığınızda, konuşmalar bilgisayarda durduğu hâlde liste boş görünür. CCDC oturumları hesaplar arasında kopyalar; böylece sırada hangi hesap oturum açarsa açsın hepsini görür.

Resmî değildir; Anthropic ile bağlantılı değildir ve Anthropic tarafından onaylanmamıştır.

**[En son sürümü indirin](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · başka bir şey kurmanız gerekmez

| Platform | Sürüm | Durum |
| --- | --- | --- |
| Windows | 1.0 | Günlük kullanımda (0.3 betiklerinin devamıdır) |
| macOS | 0.1 | Yeni; henüz bir Mac'te gerçek Claude uygulamasıyla denenmedi |
| Linux | 0.1 | Yeni; kurulumu Debian 12, Ubuntu 24.04 ve Linux Mint 22'de denendi, gerçek Claude uygulamasıyla henüz denenmedi |

## Kimin için

- **Ortak bilgisayarlar ve vardiyalı çalışma.** 24 saat üç vardiya çalışan bir iş yerinde her vardiyada aynı masada (tek bilgisayar, tek sistem kullanıcısı) farklı bir kişi oturur ve kendi Claude hesabıyla oturum açar. CCDC sayesinde sonraki vardiya Code işine önceki vardiyanın bıraktığı yerden devam eder.
- **Tek kişi, birden çok hesap.** Aynı bilgisayarda kişisel ve iş hesabı arasında ya da kuruluşlar arasında geçiş yapıyorsanız, hangi hesapla oturum açarsanız açın Code oturumlarınız orada olur.

Bilgisayardaki her Code oturumunu, o bilgisayarda oturum açan her hesap görebilir. İnsanların birbirinin oturumlarını görmemesi gereken yerlerde CCDC kullanmayın. Şirket (kuruluş) hesabı kullanıyorsanız önce şirketinizin politikasını kontrol edin: CCDC bir iş hesabının oturumlarını kişisel hesabın klasörüne de kopyalar, tersini de yapar.

## Çözdüğü sorun

Masaüstü uygulaması Code oturumlarının **listesini** her hesap ve kuruluş için ayrı bir klasörde tutar:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Claude veri klasörü | Claude günlüğü |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (veya `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Claude'u kendi bulut sağlayıcıları üzerinden kullanan kuruluşlarda `Claude` yerine `Claude-3p` kullanılır; CCDC oraya da bakar.)

**Konuşmaların** kendisi ise tüm hesapların ortak kullandığı `~/.claude/projects/.../<cliSessionId>.jsonl` içindedir. Bu yüzden biri başka bir hesapla oturum açtığında, konuşmalar diskte durduğu hâlde Code oturum listesi boş görünür.

## Nasıl çalışır

Uygulama oturum klasörünü **yalnızca açılırken ve biri oturum açarken** okur (yaklaşık iki saniye içinde iki kez). Listeyi sonradan yeniden okutmanın bir yolu yoktur. Uygulamanın kendi günlüğü bu iki olayı da kaydeder:

- oturum kapatma: `[account] Login-state transition (loggedOut: false → true ...)`
- oturum açma: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC bu günlüğü izler:

1. **Oturum kapatma** → her oturumun en yeni hâlini tüm hesap/kuruluş klasörlerine kopyalar; böylece sırada oturum açan hesap oturumları kendi klasöründe hazır bulur. Bir klasörde yalnızca eksik veya eski olan oturumlar kopyalanır.
2. **Oturum açma** → kopyalar varsayılan olarak yerinde kalır. "Oturum açıldıktan sonra kopyaları sil" açıksa CCDC, uygulamanın listesini okumasını bekler (son oturum açma satırından 6 saniye sonra) ve ardından oturum açan hesabın klasörü dışındaki her yerden kopyaları kaldırır.

Uygulamada silinen bir oturum diğer klasörlerden de kaldırılır; böylece başka bir hesapta geri gelmez.

Kopyalamadan önce CCDC boş disk alanını kontrol eder. Kopyalar artı 512 MB için yer yoksa hiçbir şey kopyalamaz ve bir uyarı gösterir.

### Kopyalar neden varsayılan olarak tutuluyor

Kopyaları tutmak SSD'yi en az yoran seçenektir:

- **Kopyalar tutulur (varsayılan)**: ilk oturum kapatmada tüm oturumlar bir kez kopyalanır. Sonrasında her oturum kapatma yalnızca bir öncekinden beri değişen oturumları yazar; bu genellikle birkaç dosya ve birkaç MB'tır ve neredeyse anında biter.
- **Kopyalar oturum açıldıktan sonra silinir**: her oturum açma kopyaları sildiği için her oturum kapatmada hepsinin yeniden yazılması gerekir. Yaklaşık 300 oturum ve 10 hesap/kuruluş klasörüyle bu, her vardiya değişiminde kabaca 600 MB yazma ve ardından yaklaşık 3.000 dosyanın silinmesi demektir.

Buna karşılık kopyalar disk alanı kaplar: yerlerinde kalırlar ve yaklaşık (oturum dosyalarının toplam boyutu × hesap/kuruluş klasörü sayısı) kadar yer tutarlar.

### Neden bağlantı noktaları (junction) veya sabit bağlantılar kullanılmıyor

Tüm hesap klasörlerini tek bir gerçek klasöre yönlendirmek kopyalamayı gereksiz kılardı; ancak Claude uygulaması her iki yolu da bilerek engeller:

- **Bağlantı noktaları (junction) ve sembolik bağlantılar**: uygulama her kayıttan önce oturum klasörünün gerçek bir klasör olduğunu denetler ve bağlantıysa kaydetmeyi reddeder (`Refusing non-directory at private dir path (symlink/file plant)`). Gerçek uygulamayla yapılan bir testte liste bağlantı noktası üzerinden okunabildi, ancak yeni oluşturulan oturumların kayıtları dahil tüm oturum ve zamanlanmış görev kayıtları başarısız oldu.
- **Sabit bağlantılar**: uygulama birden fazla bağlantısı olan dosyayı güvensiz sayar. Böyle dosyalara yazmayı reddeder ve okurken uyarı verir. Ayrıca geçici bir dosya yazıp yeniden adlandırarak kaydeder; bu da bağlantıyı her kayıtta koparır.

Bu denetimler dışarıdan yerleştirilen dosya veya klasörlere karşı koruma sağlar; bu yüzden CCDC bunları atlatmaz. Uygulamanın kendisinin yazdığı türden sıradan dosyaları kopyalar.

## Kurulum

Sisteminize uygun dosyayı [Releases](../../releases) bölümünden indirin. Dosyalar kod imzalı değildir ([aşağıya](#güvenlik-uyarısı-hakkında) bakın).

**Windows** — `CCDC-<version>-windows-x64.msi` dosyasını çalıştırın. Yönetici hakları gerekmez.

- `%LOCALAPPDATA%\Programs\CCDC` konumuna kurulur, masaüstüne ve Başlat menüsüne `Continuous Claude Desktop Code` kısayolu ekler ve CCDC'yi sistem tepsisinde başlatır.
- Bilgisayarla birlikte başlar; bunu tepsi menüsündeki "Bilgisayarla birlikte başlat" öğesiyle kapatabilirsiniz.
- Güncelleme: daha yeni `.msi` dosyasını çalıştırın. CCDC'yi kapatır, yenisiyle değiştirir ve yeniden başlatır; ayarlarınız korunur.
- Kaldırmak için Ayarlar > Uygulamalar > Yüklü uygulamalar > "Continuous Claude Desktop Code (CCDC)" yolunu izleyin.
- 0.3 betiklerinden geçiyorsanız `.msi` dosyasını kurmanız yeterlidir. CCDC ilk açılışında eski sürümü durdurur, ayarlarını ve hesap adlarını korur ve onu kaldırır.

**macOS** — `CCDC-<version>-macos-universal.dmg` dosyasını açın, **CCDC**'yi Uygulamalar klasörüne sürükleyin ve ardından açın. Menü çubuğunda bir **C** görünür. Menüsünden kapatmadığınız sürece bilgisayarla birlikte başlar. Apple Silicon ve Intel'de çalışır.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Ardından uygulamalar menüsünden **Continuous Claude Desktop Code**'u başlatın. Menüsünden kapatmadığınız sürece bilgisayarla birlikte başlar. AppIndicator uzantısı olmayan GNOME'da tepsi simgesi görünemez (Ubuntu ve Linux Mint bu uzantıyla gelir).

Kaldırma işlemi Claude oturum dosyalarını, CCDC'nin ayarlarını ve yedek klasörünü olduğu gibi bırakır.

### Güvenlik uyarısı hakkında

İmzalama, bir kişiye veya şirkete bağlı ücretli bir sertifika gerektirir; CCDC ise küçük ve ücretsiz bir proje olduğundan indirilen dosyalar imzalı değildir. Sisteminiz ilk seferde sizi uyarır; bu yalnızca yayıncının doğrulanmadığı anlamına gelir, zararlı bir şey bulunduğu anlamına gelmez.

CCDC zararlı hiçbir şey içermez. İnternete bağlanmaz, veri toplamaz ve Claude uygulamasının oturum klasörleri ile kendi klasörü dışında hiçbir şeyi değiştirmez. Kodun tamamı bu depodadır ve sürüm dosyaları bu koddan, günlükleri herkese açık olan GitHub Actions ile derlenir. İndirdiğiniz dosyayı sürümdeki `SHA256SUMS.txt` ile karşılaştırabilir ya da [kendiniz derleyebilirsiniz](#derleme).

- **Windows**: SmartScreen "Windows protected your PC" derse **More info**'ya, ardından **Run anyway**'e tıklayın.
- **macOS**: macOS uygulamayı doğrulayamadığını söylerse **System Settings > Privacy & Security**'yi açın, aşağı kaydırın ve CCDC'nin yanındaki **Open Anyway**'e tıklayın. (Ya da Terminal'de `xattr -dr com.apple.quarantine /Applications/CCDC.app` komutunu çalıştırın.)
- **Linux**: `.deb` paketleri bu şekilde denetlenmez.

## Sistem tepsisi uygulaması

Bildirim alanında veya menü çubuğunda turuncu bir **C** görünür (otomatik eşitleme kapalıyken gri). Menüyü açmak için üzerine tıklayın:

| Menü öğesi | Ne yapar |
| --- | --- |
| Durum satırı | CCDC'nin oturum açma ve kapatmaları izleyip izlemediğini gösterir |
| Otomatik eşitleme (oturum açma/kapatmayı algıla) | Kopyalamayı ve temizliği açar veya kapatır |
| Oturumları şimdi bir hesaba taşı | Tüm oturumları tek bir hesabın klasöründe toplar; o hesapta oturum açık değilse önce sorar |
| Oturum açıldıktan sonra kopyaları sil | Varsayılan olarak kapalıdır (kopyalar tutulur; yukarıya bakın) |
| Kopyalama/silme hızı | Hızlı (sınırsız), Normal (60 MB/sn) veya Yavaş (15 MB/sn). Normal ve Yavaş ayarlarında silme sırasıyla saniyede 500 ve 100 dosyayla sınırlıdır |
| Tahmin satırı | Bir sonraki oturum kapatmada kopyalamanın ne kadar süreceğini, kopyalar siliniyorsa oturum açıldıktan sonraki temizliğin süresini de gösterir |
| Bilgisayarla birlikte başlat | Bilgisayarda oturum açtığınızda CCDC'yi başlatır |
| Language | Görüntüleme dilini seçer. Bu öğe her zaman İngilizce yazılır; alt menüdeki her dil kendi dilinde yazılıdır |
| Günlüğü görüntüle | CCDC'nin `watch.log` dosyasını açar |
| Hesaplara ad ver (ccdc.json) | Hesaplara okunabilir adlar vermek için ayar dosyasını açar |
| Çıkış | Simgeyi kaldırır; süren bir kopyalama varsa önce onun bitmesini bekler |

Kopyalama başladığında (tahmini süreyle), bittiğinde, oturum açma algılandığında ve temizlik bittiğinde bildirim görünür. Hız düşük olduğunda kopyalama daha uzun sürer; bu yüzden sonraki kişi oturum açmadan önce "Kopyalama tamamlandı" bildirimini ya da yaklaşık tahmini süre kadar beklemelidir.

Tahmin, klasörlerin mevcut durumuna ve bu bilgisayarda ölçülen kopyalama ile silme hızlarına göre hesaplanır (bu hızlar CCDC'nin klasöründeki `state.json` içinde saklanır).

Windows 11'de simge görünmüyorsa görev çubuğundaki gizli simgelere (`^`) bakın. Her zaman görünür kalması için Ayarlar > Kişiselleştirme > Görev çubuğu > Diğer sistem tepsisi simgeleri bölümünde CCDC'yi açın.

CCDC yaklaşık 1 MB boyutunda tek bir yerel programdır. Beklerken yarım saniyede bir yalnızca Claude günlüğüne son denetimden beri eklenenleri okur ve kendisi yaklaşık 4 MB bellek kullanır.

## Ayarlar

Ayarlar CCDC'nin klasöründeki `ccdc.json` dosyasında tutulur:

| | CCDC'nin klasörü |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Tepsi menüsünde yaptığınız değişiklikler bu dosyaya kaydedilir ve yeniden başlatmaya gerek kalmadan bir sonraki kopyalama veya temizlikten itibaren geçerli olur. `watch.log` ve `state.json` de aynı klasördedir.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Anahtar | Değerler |
| --- | --- |
| `keepCopies` | `true` (varsayılan): kopyalar tutulur. `false`: kopyalar oturum açıldıktan sonra silinir |
| `speed` | `"fast"` (varsayılan), `"normal"`, `"gentle"` |
| `language` | Sistem dilini izlemek için boş bırakın veya o dili kullanmak için `"ja"` gibi bir dil kodu yazın |
| `startAtLogin` | Tepsi menüsündeki "Bilgisayarla birlikte başlat" öğesiyle ayarlanır |
| `aliases` | Tepsi menüsünde hesaplar için gösterilen adlar |
| `dataDir`, `appLog` | Yalnızca CCDC, Claude veri klasörünü veya günlüğünü kendisi bulamazsa: bunların tam yolları |

## Diller

Tepsi menüsü, bildirimler ve uyarılar sistemin görüntüleme dilini ya da tepsi menüsündeki **Language** öğesinden seçilen dili izler. Desteklenen diller: İngilizce, Korece, Japonca, Çince (Basitleştirilmiş ve Geleneksel), İspanyolca, Fransızca, Almanca, İtalyanca, Portekizce, Rusça, Ukraynaca, Lehçe, Felemenkçe, Türkçe, Vietnamca, Endonezce, Tayca, Arapça ve Hintçe. Diğer diller İngilizce gösterilir.

Arayüzün tüm metinleri programa gömülü tek bir tabloda, [`locales.csv`](locales.csv) dosyasındadır: **her satır bir dil**, her sütun bir ileti. Excel'de veya Google E-Tablolar'da açın; dil eklemek için bir satır ekleyin ve CCDC'yi [derleyin](#derleme).

- `lang`: bir dil etiketi (`sv`, `pt-BR`, `zh-Hant`, …). Önce dil ve bölgesi eşleşen satır, yoksa dili eşleşen satır kullanılır.
- `name`: dilin kendi dilindeki adı (`Svenska`); Language menüsünde bu ad görünür.
- Hücre içindeki satır sonlarını `\n` olarak yazın ve `{0}`, `{1}` öğelerini koruyun (oralara sayılar veya adlar gelir).
- Boş hücreler İngilizce gösterilir. Dosyayı UTF-8 CSV olarak kaydedin.

Komut satırı çıktısı ve `watch.log` İngilizce kalır.

## Komut satırı

Aynı program terminalden de çalışır (Windows'ta `%LOCALAPPDATA%\Programs\CCDC` içindeki `ccdc.exe`; macOS'ta `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` ve `--app-log <path>` ile Claude veri klasörünü veya günlüğünü elle belirtebilirsiniz.

## Yinelenen oturumlar için kurallar

- Bir oturum birden çok klasörde varsa `lastActivityAt` değeri en yeni olan kopya geçerli olur. Kopyalar kaynağın değiştirilme zamanını korur; böylece özdeş kopyalar okunmadan boyut ve zamana bakılarak tanınır.
- Daha yeni bir kopya bir dosyanın yerini aldığında eski dosya, Claude veri klasöründeki `ccdc-backups/<time>/` içine taşınır.
- `deleted_<id>` işareti olan oturumlar kopyalanmaz ve diğer klasörlerdeki kopyaları kaldırılır.
- İlk kez oturum açan bir hesabın/kuruluşun henüz klasörü olmadığından, oturum kapatılırken oraya hiçbir şey kopyalanamaz. Bu durumda oturumlar, o hesap bir sonraki kez oturum açtığında veya uygulama yeniden başlatıldıktan sonra görünür.

## Uyarılar

- CCDC, Claude masaüstü uygulamasının iç dosya yapısına ve günlük metinlerine dayanır; bir uygulama güncellemesi davranışını değiştirebilir.
- Oturum dosyaları hesaba özgü bağlayıcı (MCP) ayarlarını içerir; bu nedenle bir oturum başka bir hesapta açıldığında bazı bağlayıcılar bağlanamayabilir.
- Zamanlanmış görevler paylaşılmaz; her hesap/kuruluş klasörü kendisininkini tutar.
- Claude Code CLI'da hesap değiştirme kapsam dışıdır. CLI zaten hesaptan bağımsız olarak `~/.claude/projects` içindeki her oturumu `--resume` ile açabilir.

## Derleme

CCDC Rust ile yazılmıştır; [Rust'ı kurun](https://rustup.rs), ardından:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

Her platformun sürümü `Cargo.toml` içindeki `[package.metadata.ccdc]` bölümünde belirlenir. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) her push'ta üçünü de derler ve bir `v*` etiketinde bunları taslak bir sürüme ekler.

## Lisans

[MIT](LICENSE)
