# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | **Türkçe** | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Claude masaüstü uygulamasındaki yerel **Code** oturumlarını, bilgisayarda hangi Claude hesabı oturum açarsa açsın erişilebilir tutan bir Windows sistem tepsisi uygulamasıdır.

Resmî değildir; Anthropic ile bağlantılı değildir ve Anthropic tarafından onaylanmamıştır.

## Kimin için

CCDC, **24 saat üç vardiya çalışan iş yerleri** için geliştirildi. Her vardiyada aynı masada (tek bilgisayar, tek Windows kullanıcısı) farklı bir kişi oturur ve Claude masaüstü uygulamasında kendi Claude hesabıyla oturum açar. CCDC sayesinde sonraki vardiya oturum açıp Code işine önceki vardiyanın bıraktığı yerden devam eder.

Yani bilgisayardaki her Code oturumunu, o bilgisayarda oturum açan herkes görebilir. İnsanların birbirinin oturumlarını görmemesi gereken yerlerde CCDC kullanmayın. Şirket (kuruluş) hesabı kullanıyorsanız şirketinizin politikasını da kontrol edin.

## Çözdüğü sorun

Masaüstü uygulaması Code oturumlarının **listesini** her hesap ve kuruluş için ayrı bir klasörde tutar:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

**Konuşmaların** kendisi ise tüm hesapların ortak kullandığı `~/.claude/projects\...\<cliSessionId>.jsonl` içindedir. Bu yüzden biri başka bir hesapla oturum açtığında, konuşmalar diskte durduğu hâlde Code oturum listesi boş görünür.

## Nasıl çalışır

Uygulama oturum klasörünü **yalnızca açılırken ve biri oturum açarken** okur (yaklaşık iki saniye içinde iki kez). Listeyi sonradan yeniden okutmanın bir yolu yoktur. Uygulamanın kendi günlüğü `%LOCALAPPDATA%\Claude\Logs\main.log` bu iki olayı da kaydeder:

- oturum kapatma: `[account] Login-state transition (loggedOut: false → true ...)`
- oturum açma: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC bu günlüğü izler ve şöyle davranır:

1. **Oturum kapatma** → her oturumun en yeni hâlini tüm hesap/kuruluş klasörlerine kopyalar; böylece ardından hangi hesap oturum açarsa açsın, oturumları kendi klasöründe hazır bulur. Bir klasörde yalnızca eksik veya eski olan oturumlar kopyalanır.
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

## Gereksinimler

- Windows 10 veya 11 ve Claude masaüstü uygulaması (MSIX kurulumu)
- [Node.js](https://nodejs.org) 18 veya üzeri

## Kurulum

Yükleyiciyi [Releases](../../releases) bölümünden indirin, arşivden çıkarın ve `setup.cmd` dosyasını çalıştırın. Yönetici hakları gerekmez.

- `%LOCALAPPDATA%\Programs\CCDC` konumuna kurulur.
- Masaüstüne ve Başlat menüsüne `Continuous Claude Desktop Code` kısayolu ekler.
- Ayarlar > Uygulamalar > Yüklü uygulamalar bölümünde "Continuous Claude Desktop Code (CCDC)" olarak görünür; oradan kaldırabilirsiniz.
- Kurulumdan hemen sonra sistem tepsisinde başlar.
- Windows'ta oturum açtığınızda otomatik başlaması için `setup.cmd /startup` ile, otomatik başlatmayı kapatmak için `setup.cmd /nostartup` ile kurun. İkisi de verilmezse yeniden kurulum önceki ayarı korur.
- Yeniden kurulum, çalışıyorsa CCDC'yi durdurur ve dosyalarını değiştirir; ayarlarınız (`ccdc.json`) korunur.
- Kaldırma işlemi Claude oturum dosyalarını ve CCDC'nin yedek klasörünü olduğu gibi bırakır.

Kurmadan doğrudan bu depodan çalıştırmak için `ccdc-tray.vbs` dosyasını çalıştırın veya aşağıdaki komut satırını kullanın.

## Sistem tepsisi uygulaması

Bildirim alanında turuncu bir **C** görünür (otomatik eşitleme kapalıyken gri). Menüyü açmak için üzerine tıklayın:

| Menü öğesi | Ne yapar |
| --- | --- |
| Durum satırı | CCDC'nin oturum açma ve kapatmaları izleyip izlemediğini gösterir |
| Otomatik eşitleme (oturum açma/kapatmayı algıla) | Kopyalamayı ve temizliği açar veya kapatır |
| Oturumları şimdi bir hesaba taşı | Tüm oturumları tek bir hesabın klasöründe toplar; o hesapta oturum açık değilse önce sorar |
| Oturum açıldıktan sonra kopyaları sil | Varsayılan olarak kapalı (kopyalar tutulur; yukarıya bakın) |
| Kopyalama/silme hızı | Hızlı (sınırsız), Normal (60 MB/sn) veya Yavaş (15 MB/sn). Normal ve Yavaş ayarlarında silme sırasıyla saniyede 500 ve 100 dosyayla sınırlıdır |
| Tahmin satırı | Bir sonraki oturum kapatmada kopyalamanın ne kadar süreceğini, kopyalar siliniyorsa oturum açtıktan sonraki temizliğin süresini de gösterir |
| Language | Görüntüleme dilini seçer. Bu öğe her zaman İngilizce yazılır; alt menüdeki her dil kendi dilinde yazılıdır |
| Günlüğü görüntüle | `%LOCALAPPDATA%\CCDC\watch.log` dosyasını açar |
| Hesaplara ad ver (ccdc.json) | Hesaplara okunabilir adlar vermek için ayar dosyasını açar |
| Çıkış | Simgeyi kaldırır; süren bir kopyalama varsa önce onun bitmesini bekler |

Kopyalama başladığında (tahmini süreyle), bittiğinde, oturum açma algılandığında ve temizlik bittiğinde bildirim görünür. Hız düşük olduğunda kopyalama daha uzun sürer; bu yüzden sonraki kişi oturum açmadan önce "Kopyalama tamamlandı" bildirimini ya da yaklaşık tahmini süre kadar beklemelidir.

Tahmin, klasörlerin mevcut durumuna ve bu bilgisayarda ölçülen kopyalama ile silme hızlarına göre hesaplanır (bu hızlar `%LOCALAPPDATA%\CCDC\state.json` içinde saklanır).

Windows 11'de simge görünmüyorsa görev çubuğundaki gizli simgelere (`^`) bakın. Her zaman görünür kalması için Ayarlar > Kişiselleştirme > Görev çubuğu > Diğer sistem tepsisi simgeleri bölümünde Windows PowerShell'i açın.

### Bellek kullanımı

Sistem tepsisi uygulaması Claude günlüğünü kendisi izler ve yalnızca son denetimden beri eklenenleri okur. Node.js yalnızca kopyalama veya temizlik sırasında çalışır ve ardından çıkar; yani vardiya değişimleri arasında yalnızca tepsi uygulaması çalışır. Tepsi uygulaması boştayken kullanılmayan belleği yaklaşık dakikada bir Windows'a geri verir (yaklaşık 15 MB kullanım).

## Ayarlar

Ayarlar program klasöründeki `ccdc.json` dosyasında tutulur. Tepsi menüsünde yaptığınız değişiklikler bu dosyaya kaydedilir ve yeniden başlatmaya gerek kalmadan bir sonraki kopyalama veya temizlikten itibaren geçerli olur.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Anahtar | Değerler |
| --- | --- |
| `keepCopies` | `true` (varsayılan): kopyalar tutulur. `false`: kopyalar oturum açıldıktan sonra silinir |
| `speed` | `"fast"` (varsayılan), `"normal"`, `"gentle"` |
| `language` | Windows dilini izlemek için boş bırakın veya o dili kullanmak için `"ja"` gibi bir dil kodu yazın |
| `aliases` | Tepsi menüsünde hesaplar için gösterilen adlar |

## Diller

Tepsi menüsü, bildirimler, uyarılar ve kurulum/kaldırma pencereleri Windows görüntüleme dilini ya da tepsi menüsündeki **Language** öğesinden seçilen dili izler. Desteklenen diller: İngilizce, Korece, Japonca, Çince (Basitleştirilmiş ve Geleneksel), İspanyolca, Fransızca, Almanca, İtalyanca, Portekizce, Rusça, Ukraynaca, Lehçe, Felemenkçe, Türkçe, Vietnamca, Endonezce, Tayca, Arapça ve Hintçe. Diğer diller İngilizce gösterilir.

Arayüzün tüm metinleri tek bir tabloda, [`locales.csv`](locales.csv) dosyasındadır: **her satır bir dil**, her sütun bir ileti. Excel'de veya Google E-Tablolar'da açın; dil eklemek için bir satır ekleyin.

- `lang`: Windows kültür adı (`sv`, `pt-BR`, `zh-Hant`, …). Önce dil ve bölgesi eşleşen satır, yoksa dili eşleşen satır kullanılır.
- `name`: dilin kendi dilindeki adı (`Svenska`); Language menüsünde bu ad görünür.
- Hücre içindeki satır sonlarını `\n` olarak yazın ve `{0}`, `{1}` öğelerini koruyun (oralara sayılar veya adlar gelir).
- Boş hücreler İngilizce gösterilir. Dosyayı UTF-8 CSV olarak kaydedin.

Komut satırı çıktısı ve `watch.log`, tepsi uygulaması onları okuduğu için İngilizce kalır.

## Komut satırı

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Tepsi uygulaması gerektiğinde şu tek seferlik işleri çalıştırır: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` ve `node ccdc.mjs startup` (kendisi başladığında veya otomatik eşitleme yeniden açıldığında Claude uygulamasının güncel durumuyla eşitlenmek için).

## Yinelenen oturumlar için kurallar

- Bir oturum birden çok klasörde varsa `lastActivityAt` değeri en yeni olan kopya geçerli olur. Kopyalar kaynağın değiştirilme zamanını korur; böylece özdeş kopyalar okunmadan boyut ve zamana bakılarak tanınır.
- Daha yeni bir kopya bir dosyanın yerini aldığında eski dosya, uygulamanın veri klasöründeki `ccdc-backups\<zaman>\` içine taşınır.
- `deleted_<id>` işareti olan oturumlar kopyalanmaz ve diğer klasörlerdeki kopyaları kaldırılır.
- İlk kez oturum açan bir hesabın/kuruluşun henüz klasörü olmadığından, oturum kapatılırken oraya hiçbir şey kopyalanamaz. Bu durumda oturumlar, o hesap bir sonraki kez oturum açtığında veya uygulama yeniden başlatıldıktan sonra görünür.

## Uyarılar

- CCDC, Claude masaüstü uygulamasının iç dosya yapısına ve günlük metinlerine dayanır; uygulama güncellemesi davranışı değiştirebilir.
- Oturum dosyaları hesaba özgü bağlayıcı (MCP) ayarlarını içerir; bu nedenle bir oturum başka bir hesapta açıldığında bazı bağlayıcılar bağlanamayabilir.
- Zamanlanmış görevler paylaşılmaz; her hesap/kuruluş klasörü kendisininkini tutar.
- Claude Code CLI'da hesap değiştirme kapsam dışıdır. CLI zaten hesaptan bağımsız olarak `~/.claude/projects` içindeki her oturumu `--resume` ile açabilir.

## Lisans

[MIT](LICENSE)
