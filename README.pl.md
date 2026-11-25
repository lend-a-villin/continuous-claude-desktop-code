# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | **Polski** | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Aplikacja w zasobniku systemowym Windows, dzięki której lokalne sesje **Code** w aplikacji Claude na komputer są dostępne dla każdego konta Claude, które loguje się na tym komputerze.

Narzędzie nieoficjalne, niepowiązane z firmą Anthropic ani przez nią niezatwierdzone.

## Dla kogo

CCDC powstało z myślą o **zakładach pracujących całą dobę w systemie trzyzmianowym**. Na każdej zmianie przy tym samym stanowisku (jeden komputer, jeden użytkownik Windows) siada inna osoba i loguje się do aplikacji Claude na komputer własnym kontem Claude. Dzięki CCDC osoba z następnej zmiany po zalogowaniu kontynuuje pracę w Code od miejsca, w którym przerwała ją poprzednia zmiana.

Dlatego wszystkie sesje Code na tym komputerze widzi każdy, kto się na nim zaloguje. Nie używaj CCDC tam, gdzie ludzie nie powinni widzieć nawzajem swoich sesji. W przypadku konta firmowego (organizacji) sprawdź też zasady obowiązujące w firmie.

## Jaki problem rozwiązuje

Aplikacja trzyma **listę** sesji Code w osobnym folderze dla każdego konta i organizacji:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

Same **rozmowy** są w `~/.claude/projects\...\<cliSessionId>.jsonl`, czyli w miejscu wspólnym dla wszystkich kont. Dlatego po zalogowaniu innym kontem lista sesji Code wygląda na pustą, choć rozmowy wciąż są na dysku.

## Jak to działa

Aplikacja odczytuje folder sesji **tylko przy uruchomieniu i przy logowaniu** (przy logowaniu dwa razy, w ciągu około dwóch sekund). Nie da się później zmusić jej do ponownego odczytania samej listy. Dziennik aplikacji, `%LOCALAPPDATA%\Claude\Logs\main.log`, zapisuje zarówno wylogowania, jak i logowania:

- wylogowanie: `[account] Login-state transition (loggedOut: false → true ...)`
- logowanie: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC śledzi ten dziennik i działa tak:

1. **Wylogowanie** → najnowsza wersja każdej sesji jest kopiowana do folderów wszystkich kont/organizacji, więc niezależnie od tego, które konto zaloguje się jako następne, sesje będą już w jego folderze. Kopiowane są tylko sesje, których w danym folderze brakuje lub które są nieaktualne.
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

- **Punkty połączenia (junctions) i dowiązania symboliczne**: przed każdym zapisem aplikacja sprawdza, czy folder sesji jest prawdziwym folderem, i odmawia zapisu, jeśli jest dowiązaniem (`Refusing non-directory at private dir path (symlink/file plant)`). W teście z rzeczywistą aplikacją listę dało się odczytać przez punkt połączenia, ale każdy zapis sesji i zaplanowanych zadań kończył się błędem, także zapis zupełnie nowych sesji.
- **Dowiązania twarde**: aplikacja uznaje plik z więcej niż jednym dowiązaniem za niebezpieczny. Odmawia zapisu do takich plików i ostrzega przy ich odczycie. Zapisuje też dane, tworząc plik tymczasowy i zmieniając jego nazwę, co przy każdym zapisie zrywałoby dowiązanie.

Te kontrole chronią przed plikami lub folderami podrzuconymi z zewnątrz, więc CCDC ich nie omija. Kopiuje zwykłe pliki — takie same, jakie zapisuje sama aplikacja.

## Wymagania

- Windows 10 lub 11 i aplikacja Claude na komputer (instalacja MSIX)
- [Node.js](https://nodejs.org) 18 lub nowszy

## Instalacja

Pobierz instalator z sekcji [Releases](../../releases), rozpakuj go i uruchom `setup.cmd`. Uprawnienia administratora nie są potrzebne.

- Instaluje się w `%LOCALAPPDATA%\Programs\CCDC`.
- Dodaje skrót `Continuous Claude Desktop Code` na pulpicie i w menu Start.
- Pojawia się w Ustawienia > Aplikacje > Zainstalowane aplikacje jako „Continuous Claude Desktop Code (CCDC)”; tam można go odinstalować.
- Uruchamia się w zasobniku zaraz po instalacji.
- Aby CCDC uruchamiał się przy logowaniu do Windows, zainstaluj go poleceniem `setup.cmd /startup`; aby wyłączyć autostart, użyj `setup.cmd /nostartup`. Ponowna instalacja bez żadnej z tych opcji zachowuje poprzednie ustawienie.
- Ponowna instalacja zatrzymuje uruchomiony CCDC i zastępuje jego pliki; ustawienia (`ccdc.json`) zostają zachowane.
- Odinstalowanie pozostawia pliki sesji Claude i folder kopii zapasowych CCDC bez zmian.

Aby korzystać z niego prosto z tego repozytorium bez instalacji, uruchom `ccdc-tray.vbs` albo skorzystaj z wiersza poleceń poniżej.

## Aplikacja w zasobniku

W obszarze powiadomień pojawia się pomarańczowa ikona z literą **C** (szara, gdy automatyczna synchronizacja jest wyłączona). Kliknij ją, aby otworzyć menu:

| Pozycja menu | Działanie |
| --- | --- |
| Wiersz stanu | Pokazuje, czy CCDC obserwuje logowania i wylogowania |
| Automatyczna synchronizacja (wykrywaj logowanie/wylogowanie) | Włącza lub wyłącza kopiowanie i porządki |
| Przenieś sesje do konta teraz | Zbiera wszystkie sesje w folderze jednego konta; prosi o potwierdzenie, jeśli to konto nie jest zalogowane |
| Usuwaj kopie po zalogowaniu | Domyślnie wyłączone (kopie zostają; patrz wyżej) |
| Szybkość kopiowania/usuwania | Szybko (bez limitu), Normalnie (60 MB/s) lub Łagodnie (15 MB/s). W trybach „Normalnie” i „Łagodnie” usuwanie jest ograniczone do 500 i 100 plików na sekundę |
| Wiersz szacunku | Pokazuje, ile potrwa kopiowanie przy następnym wylogowaniu, a jeśli kopie są usuwane, także porządki po zalogowaniu |
| Language | Wybór języka wyświetlania. Ta pozycja jest zawsze podpisana po angielsku; każdy język w podmenu jest zapisany w tym języku |
| Pokaż dziennik | Otwiera `%LOCALAPPDATA%\CCDC\watch.log` |
| Nazwij konta (ccdc.json) | Otwiera plik ustawień, aby nadać kontom czytelne nazwy |
| Zakończ | Usuwa ikonę; jeśli trwa kopiowanie, najpierw czeka na jego zakończenie |

Powiadomienia pojawiają się, gdy zaczyna się kopiowanie (z szacowanym czasem), gdy się kończy, gdy wykryto logowanie i gdy kończą się porządki. Przy wolniejszej szybkości kopiowanie trwa dłużej, dlatego następna osoba powinna poczekać na powiadomienie „Kopiowanie zakończone” (albo odczekać mniej więcej szacowany czas) i dopiero wtedy się zalogować.

Szacunek jest liczony na podstawie bieżącego stanu folderów oraz szybkości kopiowania i usuwania zmierzonych na tym komputerze (zapisanych w `%LOCALAPPDATA%\CCDC\state.json`).

Jeśli w Windows 11 nie widać ikony, sprawdź ukryte ikony (`^`) na pasku zadań. Aby była zawsze widoczna, włącz Windows PowerShell w Ustawienia > Personalizacja > Pasek zadań > Inne ikony zasobnika systemowego.

### Użycie pamięci

Aplikacja w zasobniku sama obserwuje dziennik Claude, odczytując tylko to, co dopisano od ostatniego sprawdzenia. Node.js działa tylko podczas kopiowania lub porządków i potem się kończy, więc między zmianami działa wyłącznie aplikacja w zasobniku. W stanie bezczynności mniej więcej raz na minutę oddaje ona Windows nieużywaną pamięć (zajmuje około 15 MB).

## Ustawienia

Ustawienia są przechowywane w pliku `ccdc.json` w folderze programu. Zmiany wprowadzone w menu aplikacji są zapisywane w tym pliku i obowiązują od następnego kopiowania lub porządków, bez ponownego uruchamiania.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Klucz | Wartości |
| --- | --- |
| `keepCopies` | `true` (domyślnie): kopie zostają. `false`: kopie są usuwane po zalogowaniu |
| `speed` | `"fast"` (domyślnie), `"normal"`, `"gentle"` |
| `language` | Puste, aby używać języka Windows, albo kod języka, np. `"ja"`, aby używać tego języka |
| `aliases` | Nazwy kont pokazywane w menu zasobnika |

## Języki

Menu w zasobniku, powiadomienia, ostrzeżenia oraz okna instalacji/dezinstalacji korzystają z języka wyświetlania Windows albo z języka wybranego w pozycji **Language** w menu. Obsługiwane: angielski, koreański, japoński, chiński (uproszczony i tradycyjny), hiszpański, francuski, niemiecki, włoski, portugalski, rosyjski, ukraiński, polski, niderlandzki, turecki, wietnamski, indonezyjski, tajski, arabski i hindi. Inne języki są wyświetlane po angielsku.

Wszystkie teksty interfejsu są w jednym arkuszu, [`locales.csv`](locales.csv): **jeden wiersz to jeden język**, jedna kolumna to jeden komunikat. Otwórz go w Excelu lub Arkuszach Google; aby dodać język, dodaj wiersz.

- `lang`: nazwa ustawień regionalnych Windows (`sv`, `pt-BR`, `zh-Hant`, …). Najpierw używany jest wiersz zgodny z językiem i regionem, potem zgodny z samym językiem.
- `name`: nazwa języka zapisana w tym języku (`Svenska`); pokazuje ją menu Language.
- Podziały wierszy w komórce zapisuj jako `\n` i zachowaj `{0}`, `{1}` (w ich miejsce trafiają liczby lub nazwy).
- Puste komórki są wyświetlane po angielsku. Zapisuj plik jako CSV UTF-8.

Dane wyjściowe wiersza poleceń i `watch.log` pozostają po angielsku, bo aplikacja w zasobniku je odczytuje.

## Wiersz poleceń

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Aplikacja w zasobniku sama uruchamia jednorazowe zadania: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` oraz `node ccdc.mjs startup` (aby uwzględnić bieżący stan Claude przy własnym uruchomieniu lub po ponownym włączeniu automatycznej synchronizacji).

## Zasady dotyczące duplikatów

- Jeśli sesja jest w kilku folderach, pierwszeństwo ma kopia z najnowszym `lastActivityAt`. Kopie zachowują czas modyfikacji źródła, więc identyczne kopie są rozpoznawane po rozmiarze i czasie, bez odczytywania ich.
- Gdy nowsza kopia zastępuje plik, stary plik trafia do `ccdc-backups\<czas>\` w folderze danych aplikacji.
- Sesje z oznaczeniem `deleted_<id>` nie są kopiowane, a ich kopie w innych folderach są usuwane.
- Konto lub organizacja logująca się po raz pierwszy nie ma jeszcze folderu, więc przy wylogowaniu nie dało się tam nic skopiować. W takim przypadku sesje pojawią się po następnym zalogowaniu na to konto lub po ponownym uruchomieniu aplikacji.

## Zastrzeżenia

- CCDC zależy od wewnętrznej struktury plików i sformułowań w dzienniku aplikacji Claude na komputer; aktualizacja aplikacji może zmienić jego działanie.
- Pliki sesji zawierają ustawienia łączników (MCP) poszczególnych kont, więc po otwarciu sesji z innego konta niektóre łączniki mogą się nie połączyć.
- Zaplanowane zadania nie są współdzielone; każdy folder konta/organizacji ma własne.
- Zmiana konta w Claude Code CLI nie jest obsługiwana. CLI i tak może otworzyć dowolną sesję z `~/.claude/projects` za pomocą `--resume`, niezależnie od konta.

## Licencja

[MIT](LICENSE)
