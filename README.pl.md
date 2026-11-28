# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | **Polski** | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Aplikacja w zasobniku systemowym, dzięki której sesje **Code** w aplikacji Claude na komputer nie znikają po przełączeniu konta Claude.

Aplikacja Claude na komputer prowadzi osobną listę sesji Code dla każdego konta i każdej organizacji. Po zalogowaniu innym kontem lista wygląda na pustą, choć rozmowy wciąż są na komputerze. CCDC kopiuje sesje między kontami, więc niezależnie od tego, kto zaloguje się jako następny, zobaczy je wszystkie.

Narzędzie nieoficjalne, niepowiązane z firmą Anthropic ani przez nią niezatwierdzone.

**[Pobierz najnowszą wersję](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · nie trzeba niczego doinstalowywać

| Platforma | Wersja | Stan |
| --- | --- | --- |
| Windows | 1.1 | Nowa; przetestowana z prawdziwą aplikacją Claude w systemie Windows, gdzie działa zgodnie z opisem (kontynuacja skryptów 0.3) |
| macOS | 0.2 | Nowa; jeszcze nie testowana z prawdziwą aplikacją Claude na Macu |
| Linux | 0.2 | Nowa; instalacja przetestowana w Debianie 12, Ubuntu 24.04 i Linux Mint 22, ale jeszcze nie z prawdziwą aplikacją Claude |

## Dla kogo

- **Wspólne komputery i praca zmianowa.** W zakładzie pracującym całą dobę w systemie trzyzmianowym na każdej zmianie przy tym samym stanowisku (jeden komputer, jeden użytkownik systemu) siada inna osoba i loguje się własnym kontem Claude. Dzięki CCDC następna zmiana kontynuuje pracę w Code od miejsca, w którym przerwała ją poprzednia.
- **Jedna osoba, kilka kont.** Jeśli na tym samym komputerze przełączasz się między kontem prywatnym a służbowym albo między organizacjami, Twoje sesje Code są dostępne bez względu na to, którym kontem się zalogujesz.

Wszystkie sesje Code na komputerze widzi każde konto, które się na nim zaloguje. Nie używaj CCDC tam, gdzie ludzie nie powinni widzieć nawzajem swoich sesji. Jeśli korzystasz z konta firmowego (organizacji), najpierw sprawdź zasady obowiązujące w firmie: CCDC kopiuje sesje konta służbowego także do folderu konta prywatnego i odwrotnie.

## Jaki problem rozwiązuje

Aplikacja na komputer trzyma **listę** sesji Code w osobnym folderze dla każdego konta i organizacji:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Folder danych Claude | Dziennik Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (lub `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Organizacje, które korzystają z Claude przez własnego dostawcę chmury, mają folder `Claude-3p` zamiast `Claude`; CCDC sprawdza również go).

Same **rozmowy** są w `~/.claude/projects/.../<cliSessionId>.jsonl`, czyli w miejscu wspólnym dla wszystkich kont. Dlatego po zalogowaniu innym kontem lista sesji Code wygląda na pustą, choć rozmowy wciąż są na dysku.

## Jak to działa

Aplikacja odczytuje folder sesji **tylko przy uruchomieniu i przy logowaniu** (przy logowaniu dwa razy, w ciągu około dwóch sekund). Nie da się później zmusić jej do ponownego odczytania samej listy. Oba zdarzenia zapisuje dziennik aplikacji:

- wylogowanie: `[account] Login-state transition (loggedOut: false → true ...)`
- logowanie: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC śledzi ten dziennik i działa tak:

1. **Wylogowanie** → najnowsza wersja każdej sesji jest kopiowana do folderów wszystkich kont/organizacji, więc konto, które zaloguje się jako następne, zastanie sesje już we własnym folderze. Kopiowane są tylko sesje, których w danym folderze brakuje lub które są nieaktualne.
2. **Logowanie** → domyślnie kopie zostają tam, gdzie są. Jeśli włączono „Usuwaj kopie po zalogowaniu”, CCDC czeka, aż aplikacja odczyta listę (6 sekund po ostatnim wierszu o logowaniu), a potem usuwa kopie wszędzie poza folderem zalogowanego konta.

Sesja usunięta w aplikacji jest usuwana także z pozostałych folderów, więc nie pojawi się ponownie na innym koncie.

Przed kopiowaniem CCDC sprawdza wolne miejsce na dysku. Jeśli nie ma miejsca na kopie i dodatkowe 512 MB, nie kopiuje niczego i wyświetla ostrzeżenie.

### Dlaczego kopie są domyślnie zachowywane

Zachowywanie kopii to opcja najmniej obciążająca dysk SSD:

- **Kopie zachowane (domyślnie)**: pierwsze wylogowanie jednorazowo kopiuje wszystkie sesje. Potem każde wylogowanie zapisuje tylko sesje zmienione od poprzedniego — zwykle kilka plików i kilka MB — i kończy się niemal od razu.
- **Kopie usuwane po zalogowaniu**: każde logowanie usuwa kopie, więc każde wylogowanie musi zapisać je wszystkie od nowa. Przy około 300 sesjach i 10 folderach kont/organizacji to mniej więcej 600 MB zapisu przy każdym przekazaniu zmiany, a potem jeszcze usunięcie około 3000 plików.

Ceną jest miejsce na dysku: kopie zostają i zajmują około (łączny rozmiar plików sesji × liczba folderów kont/organizacji).

### Dlaczego nie punkty połączenia ani dowiązania twarde

Gdyby wszystkie foldery kont wskazywały jeden prawdziwy folder, kopiowanie nie byłoby potrzebne, ale aplikacja Claude celowo blokuje oba sposoby:

- **Punkty połączenia (junctions) i dowiązania symboliczne**: przed każdym zapisem aplikacja sprawdza, czy folder sesji jest prawdziwym folderem, i odmawia zapisu, jeśli jest dowiązaniem (`Refusing non-directory at private dir path (symlink/file plant)`). W teście z prawdziwą aplikacją listę dało się odczytać przez punkt połączenia, ale każdy zapis sesji i zaplanowanych zadań kończył się błędem, także zapis zupełnie nowych sesji.
- **Dowiązania twarde**: aplikacja uznaje plik z więcej niż jednym dowiązaniem za niebezpieczny. Odmawia zapisu do takich plików i ostrzega przy ich odczycie. Zapisuje też dane, tworząc plik tymczasowy i zmieniając jego nazwę, co przy każdym zapisie zrywałoby dowiązanie.

Te kontrole chronią przed plikami lub folderami podrzuconymi z zewnątrz, więc CCDC ich nie omija. Kopiuje zwykłe pliki — takie same, jakie zapisuje sama aplikacja.

## Instalacja

Pobierz plik dla swojego systemu z sekcji [Releases](../../releases). Pliki nie są podpisane cyfrowo (zob. [niżej](#o-ostrzeżeniu-bezpieczeństwa)).

**Windows** — uruchom `CCDC-<version>-windows-x64.msi`. Uprawnienia administratora nie są potrzebne.

- Instaluje się w `%LOCALAPPDATA%\Programs\CCDC`, dodaje skrót `Continuous Claude Desktop Code` na pulpicie i w menu Start, a następnie uruchamia CCDC w zasobniku.
- Uruchamia się razem z komputerem; możesz to wyłączyć pozycją „Uruchamiaj razem z komputerem” w menu zasobnika.
- Aktualizacja: uruchom nowszy plik `.msi`. Zamknie on CCDC, zastąpi go nową wersją i uruchomi ponownie; ustawienia zostają zachowane.
- Odinstalowanie: Ustawienia > Aplikacje > Zainstalowane aplikacje > „Continuous Claude Desktop Code (CCDC)”.
- Przejście ze skryptów 0.3: wystarczy zainstalować plik `.msi`. Przy pierwszym uruchomieniu CCDC zatrzyma starą wersję, przejmie jej ustawienia i nazwy kont, a następnie ją usunie.

**macOS** — otwórz `CCDC-<version>-macos-universal.dmg`, przeciągnij **CCDC** do folderu Aplikacje i uruchom go. Na pasku menu pojawi się litera **C**. CCDC uruchamia się razem z komputerem, chyba że wyłączysz to w jego menu. Działa na komputerach z Apple Silicon i z procesorami Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Następnie uruchom **Continuous Claude Desktop Code** z menu aplikacji. CCDC uruchamia się razem z komputerem, chyba że wyłączysz to w jego menu. W środowisku GNOME bez rozszerzenia AppIndicator ikona w zasobniku nie może się pojawić (Ubuntu i Linux Mint mają je w zestawie).

Odinstalowanie pozostawia bez zmian pliki sesji Claude, ustawienia CCDC i folder kopii zapasowych.

### O ostrzeżeniu bezpieczeństwa

Do podpisywania potrzebny jest płatny certyfikat wystawiony na osobę lub firmę, a CCDC to mały, darmowy projekt, dlatego pobierane pliki nie są podpisane. System ostrzega przy pierwszym uruchomieniu, ale oznacza to tylko, że wydawca nie został zweryfikowany, a nie że wykryto coś szkodliwego.

CCDC nie zawiera niczego szkodliwego. Nie łączy się z internetem, nie zbiera danych i nie zmienia niczego poza folderami sesji aplikacji Claude i własnym folderem. Cały kod znajduje się w tym repozytorium, a pliki wydań są z niego budowane przez GitHub Actions, których dzienniki są publiczne. Pobrany plik możesz sprawdzić z `SHA256SUMS.txt` dołączonym do wydania albo [zbudować program samodzielnie](#budowanie).

- **Windows**: jeśli SmartScreen wyświetli komunikat „System Windows ochronił ten komputer”, kliknij **Więcej informacji**, a potem **Uruchom mimo to**.
- **macOS**: jeśli macOS zgłosi, że nie może zweryfikować aplikacji, otwórz **Ustawienia systemowe > Prywatność i ochrona**, przewiń w dół i kliknij **Otwórz mimo to** obok CCDC. (Możesz też uruchomić w Terminalu `xattr -dr com.apple.quarantine /Applications/CCDC.app`).
- **Linux**: pakiety `.deb` nie są sprawdzane w ten sposób.

## Aplikacja w zasobniku

W obszarze powiadomień lub na pasku menu pojawia się pomarańczowa litera **C** (szara, gdy automatyczna synchronizacja jest wyłączona). Kliknij ją, aby otworzyć menu:

| Pozycja menu | Działanie |
| --- | --- |
| Wiersz stanu | Pokazuje, czy CCDC obserwuje logowania i wylogowania |
| Wiersze czasu logowania | Po jednym wierszu na każde wylogowane konto z tym, ile czasu minęło od jego zalogowania (np. „zalogowano 2 godz. 13 min temu”), aby ocenić, gdzie jest w 5-godzinnym oknie limitu Claude |
| Automatyczna synchronizacja (wykrywaj logowanie/wylogowanie) | Włącza lub wyłącza kopiowanie i porządki |
| Przenieś sesje do konta teraz | Widoczne, gdy „Usuwaj kopie po zalogowaniu” jest włączone. Na wypadek niewykrytego wylogowania: zbiera wszystkie sesje w folderze jednego konta; prosi o potwierdzenie, jeśli to konto nie jest zalogowane |
| Synchronizuj wszystkie konta teraz | Widoczne w tym samym miejscu, gdy „Usuwaj kopie po zalogowaniu” jest wyłączone. Na wypadek niewykrytego wylogowania: kopiuje wszystkie sesje do folderu każdego konta, tak jak przy wylogowaniu |
| Usuwaj kopie po zalogowaniu | Domyślnie wyłączone (kopie zostają; zob. wyżej) |
| Szybkość kopiowania/usuwania | Szybko (bez limitu), Normalnie (60 MB/s) lub Łagodnie (15 MB/s). W trybach „Normalnie” i „Łagodnie” usuwanie jest ograniczone do 500 i 100 plików na sekundę |
| Wiersz szacunku | Pokazuje, ile potrwa kopiowanie przy następnym wylogowaniu, a jeśli kopie są usuwane, także porządki po zalogowaniu |
| Uruchamiaj razem z komputerem | Uruchamia CCDC, gdy logujesz się do komputera |
| Language | Wybór języka wyświetlania. Ta pozycja jest zawsze podpisana po angielsku; każdy język w podmenu jest zapisany w tym języku |
| Pokaż dziennik | Otwiera plik `watch.log` CCDC |
| Nazwij konta (ccdc.json) | Otwiera plik ustawień, aby nadać kontom czytelne nazwy |
| Zakończ | Usuwa ikonę; jeśli trwa kopiowanie, najpierw czeka na jego zakończenie |

Powiadomienia pojawiają się, gdy zaczyna się kopiowanie (z szacowanym czasem), gdy się kończy, gdy wykryto logowanie i gdy kończą się porządki. Przy wolniejszej szybkości kopiowanie trwa dłużej, dlatego następna osoba powinna poczekać na powiadomienie „Kopiowanie zakończone” (albo odczekać mniej więcej szacowany czas) i dopiero wtedy się zalogować.

Szacunek jest liczony na podstawie bieżącego stanu folderów oraz szybkości kopiowania i usuwania zmierzonych na tym komputerze (zapisanych w pliku `state.json` w folderze CCDC).

Jeśli w Windows 11 nie widać ikony, sprawdź ukryte ikony (`^`) na pasku zadań. Aby była zawsze widoczna, włącz CCDC w Ustawienia > Personalizacja > Pasek zadań > Inne ikony zasobnika systemowego.

CCDC to jeden natywny program o rozmiarze około 1 MB. W trakcie oczekiwania co pół sekundy odczytuje tylko to, co dopisano do dziennika Claude od ostatniego sprawdzenia, i sam zajmuje około 4 MB pamięci.

## Ustawienia

Ustawienia są przechowywane w pliku `ccdc.json` w folderze CCDC:

| | Folder CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Zmiany wprowadzone w menu zasobnika są zapisywane w tym pliku i obowiązują od następnego kopiowania lub porządków, bez ponownego uruchamiania. W tym samym folderze znajdują się `watch.log` i `state.json`.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Klucz | Wartości |
| --- | --- |
| `keepCopies` | `true` (domyślnie): kopie zostają. `false`: kopie są usuwane po zalogowaniu |
| `speed` | `"fast"` (domyślnie), `"normal"`, `"gentle"` |
| `language` | Puste, aby używać języka systemu, albo kod języka, np. `"ja"`, aby używać tego języka |
| `startAtLogin` | Ustawiane pozycją „Uruchamiaj razem z komputerem” w menu zasobnika |
| `aliases` | Nazwy kont pokazywane w menu zasobnika |
| `dataDir`, `appLog` | Tylko jeśli CCDC nie potrafi sam znaleźć folderu danych lub dziennika Claude: ich pełne ścieżki |

## Języki

Menu w zasobniku, powiadomienia i ostrzeżenia korzystają z języka wyświetlania systemu albo z języka wybranego w pozycji **Language** w menu zasobnika. Obsługiwane: angielski, koreański, japoński, chiński (uproszczony i tradycyjny), hiszpański, francuski, niemiecki, włoski, portugalski, rosyjski, ukraiński, polski, niderlandzki, turecki, wietnamski, indonezyjski, tajski, arabski i hindi. Inne języki są wyświetlane po angielsku.

Wszystkie teksty interfejsu są w jednym arkuszu, [`locales.csv`](locales.csv), wbudowanym w program: **jeden wiersz to jeden język**, jedna kolumna to jeden komunikat. Otwórz go w Excelu lub Arkuszach Google; aby dodać język, dodaj wiersz i [zbuduj](#budowanie) CCDC.

- `lang`: znacznik języka (`sv`, `pt-BR`, `zh-Hant`, …). Najpierw używany jest wiersz zgodny z językiem i regionem, potem zgodny z samym językiem.
- `name`: nazwa języka zapisana w tym języku (`Svenska`); pokazuje ją menu Language.
- Podziały wierszy w komórce zapisuj jako `\n` i zachowaj `{0}`, `{1}` (w ich miejsce trafiają liczby lub nazwy).
- Puste komórki są wyświetlane po angielsku. Zapisuj plik jako CSV UTF-8.

Dane wyjściowe wiersza poleceń i `watch.log` pozostają po angielsku.

## Wiersz poleceń

Ten sam program działa też w terminalu (w Windows to `ccdc.exe` w `%LOCALAPPDATA%\Programs\CCDC`, w macOS `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, last sign-in times, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

Opcje `--data-dir <path>` i `--app-log <path>` pozwalają ręcznie wskazać folder danych lub dziennik Claude.

## Zasady dotyczące duplikatów

- Jeśli sesja jest w kilku folderach, pierwszeństwo ma kopia z najnowszym `lastActivityAt`. Kopie zachowują czas modyfikacji źródła, więc identyczne kopie są rozpoznawane po rozmiarze i czasie, bez odczytywania ich.
- Gdy nowsza kopia zastępuje plik, stary plik trafia do `ccdc-backups/<time>/` w folderze danych Claude.
- Sesje z oznaczeniem `deleted_<id>` nie są kopiowane, a ich kopie w innych folderach są usuwane.
- Konto lub organizacja logująca się po raz pierwszy nie ma jeszcze folderu, więc przy wylogowaniu nie dało się tam nic skopiować. W takim przypadku sesje pojawią się po następnym zalogowaniu na to konto lub po ponownym uruchomieniu aplikacji.

## Zastrzeżenia

- CCDC zależy od wewnętrznej struktury plików i sformułowań w dzienniku aplikacji Claude na komputer; aktualizacja aplikacji może zmienić jego działanie.
- Pliki sesji zawierają ustawienia łączników (MCP) poszczególnych kont, więc po otwarciu sesji z innego konta niektóre łączniki mogą się nie połączyć.
- Zaplanowane zadania nie są współdzielone; każdy folder konta/organizacji ma własne.
- Zmiana konta w Claude Code CLI nie jest obsługiwana. CLI i tak może otworzyć dowolną sesję z `~/.claude/projects` za pomocą `--resume`, niezależnie od konta.

## Budowanie

CCDC jest napisany w Rust; [zainstaluj Rust](https://rustup.rs), a następnie:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

Wersję dla każdej platformy ustawia się w sekcji `[package.metadata.ccdc]` pliku `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) buduje wszystkie trzy przy każdym wypchnięciu zmian, a w przypadku tagu `v*` dołącza je do szkicu wydania.

## Licencja

[MIT](LICENSE)
