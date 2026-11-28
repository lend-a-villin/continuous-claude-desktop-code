# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | **Nederlands** | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Een systeemvak-app die uw **Code**-sessies in de Claude-desktopapp bewaart wanneer u van Claude-account wisselt.

De Claude-desktopapp houdt voor elk account en elke organisatie een aparte lijst met Code-sessies bij. Meldt u zich aan met een ander account, dan lijkt die lijst leeg, terwijl de gesprekken gewoon nog op de computer staan. CCDC kopieert de sessies naar alle accounts, zodat het account dat zich daarna aanmeldt ze allemaal ziet.

Onofficieel; niet verbonden aan of goedgekeurd door Anthropic.

**[Download de nieuwste release](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · verder niets te installeren

| Platform | Versie | Status |
| --- | --- | --- |
| Windows | 1.1 | Nieuw; getest met de echte Claude-app op Windows, waar hij werkt zoals beschreven (opvolger van de 0.3-scripts) |
| macOS | 0.2 | Nieuw; nog niet getest met de echte Claude-app op een Mac |
| Linux | 0.2 | Nieuw; installatie getest op Debian 12, Ubuntu 24.04 en Linux Mint 22, nog niet met de echte Claude-app |

## Voor wie CCDC bedoeld is

- **Gedeelde computers en ploegendienst.** In een bedrijf dat 24 uur per dag in drie ploegen werkt, zit in elke ploeg iemand anders aan dezelfde werkplek (één computer, één systeemgebruiker) en meldt zich aan met een eigen Claude-account. Met CCDC gaat de volgende ploeg verder met het Code-werk waar de vorige ploeg was gebleven.
- **Eén persoon, meerdere accounts.** Wisselt u op dezelfde computer tussen een persoonlijk account en een werkaccount, of tussen organisaties, dan zijn uw Code-sessies er, met welk account u zich ook aanmeldt.

Alle Code-sessies op de computer worden zichtbaar voor elk account dat zich erop aanmeldt. Gebruik CCDC niet waar mensen elkaars sessies niet mogen zien. Gebruikt u een bedrijfsaccount (organisatie), controleer dan eerst het beleid van uw bedrijf: CCDC kopieert de sessies van een werkaccount ook naar de map van een persoonlijk account, en omgekeerd.

## Welk probleem CCDC oplost

De desktopapp bewaart de **lijst** met Code-sessies in een aparte map per account en organisatie:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Gegevensmap van Claude | Logboek van Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (of `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Organisaties die Claude via hun eigen cloudprovider gebruiken, hebben `Claude-3p` in plaats van `Claude`; CCDC kijkt ook daar.)

De **gesprekken** zelf staan in `~/.claude/projects/.../<cliSessionId>.jsonl`, een locatie die alle accounts delen. Wie zich met een ander account aanmeldt, ziet daarom een lege lijst met Code-sessies, terwijl de gesprekken gewoon nog op de schijf staan.

## Werking

De app leest zijn sessiemap **alleen bij het starten en bij het aanmelden** (bij het aanmelden twee keer, binnen ongeveer twee seconden). Er is geen manier om later alleen de lijst opnieuw te laten laden. Het eigen logboek van de app legt beide gebeurtenissen vast:

- afmelden: `[account] Login-state transition (loggedOut: false → true ...)`
- aanmelden: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC volgt dat logboek:

1. **Afmelden** → de nieuwste versie van elke sessie wordt naar alle account-/organisatiemappen gekopieerd, zodat het volgende account dat zich aanmeldt de sessies al in zijn eigen map vindt. Alleen sessies die in een map ontbreken of verouderd zijn, worden gekopieerd.
2. **Aanmelden** → standaard blijven de kopieën staan. Staat "Kopieën verwijderen na aanmelden" aan, dan wacht CCDC tot de app zijn lijst heeft gelezen (6 seconden na de laatste aanmeldregel) en verwijdert daarna de kopieën overal behalve in de map van het aangemelde account.

Een sessie die in de app wordt verwijderd, verdwijnt ook uit de andere mappen en komt dus niet terug onder een ander account.

Voor het kopiëren controleert CCDC de vrije schijfruimte. Is er niet genoeg ruimte voor de kopieën plus 512 MB, dan wordt er niets gekopieerd en verschijnt er een waarschuwing.

### Waarom kopieën standaard blijven staan

Kopieën bewaren is de optie die een SSD het minst belast:

- **Kopieën bewaren (standaard)**: bij de eerste afmelding worden alle sessies één keer gekopieerd. Daarna schrijft elke afmelding alleen de sessies die sinds de vorige keer zijn gewijzigd, meestal een paar bestanden en een paar MB, en is het kopiëren bijna meteen klaar.
- **Kopieën verwijderen na aanmelden**: elke aanmelding verwijdert de kopieën, dus elke afmelding moet ze allemaal opnieuw schrijven. Bij ongeveer 300 sessies en 10 account-/organisatiemappen wordt er bij elke ploegwissel zo'n 600 MB geschreven, en daarna worden nog ongeveer 3.000 bestanden verwijderd.

Daar staat wel schijfruimte tegenover: de kopieën blijven staan en nemen ongeveer (totale grootte van de sessiebestanden × aantal account-/organisatiemappen) in beslag.

### Waarom geen mapkoppelingen of harde links

Als alle accountmappen naar één echte map zouden wijzen, was kopiëren niet nodig, maar de Claude-app blokkeert bewust beide manieren:

- **Mapkoppelingen (junctions) en symbolische koppelingen**: telkens voordat de app iets opslaat, controleert hij of de sessiemap een echte map is, en hij weigert op te slaan als het een koppeling is (`Refusing non-directory at private dir path (symlink/file plant)`). In een test met de echte app kon de lijst via een mapkoppeling worden gelezen, maar mislukte het opslaan van sessies en geplande taken telkens, ook bij gloednieuwe sessies.
- **Harde links**: de app beschouwt een bestand met meer dan één link als onveilig. Hij weigert naar zulke bestanden te schrijven en waarschuwt wanneer hij ze leest. Ook slaat hij op door een tijdelijk bestand te schrijven en te hernoemen, waardoor de link bij elke keer opslaan verbroken zou worden.

Deze controles beschermen tegen bestanden of mappen die van buitenaf zijn neergezet, dus CCDC omzeilt ze niet. CCDC kopieert gewone bestanden, van dezelfde soort die de app zelf schrijft.

## Installatie

Download het bestand voor uw systeem via [Releases](../../releases). De bestanden zijn niet digitaal ondertekend (zie [hieronder](#over-de-beveiligingswaarschuwing)).

**Windows**: voer `CCDC-<version>-windows-x64.msi` uit. Beheerdersrechten zijn niet nodig.

- Installeert in `%LOCALAPPDATA%\Programs\CCDC`, maakt een snelkoppeling `Continuous Claude Desktop Code` op het bureaublad en in het menu Start, en start CCDC in het systeemvak.
- Start mee met de computer; dat zet u uit met "Starten met de computer" in het menu van het systeemvak.
- Bijwerken: voer de nieuwere `.msi` uit. Die sluit CCDC, vervangt het en start het opnieuw; uw instellingen blijven bewaard.
- Verwijderen gaat via Instellingen > Apps > Geïnstalleerde apps > "Continuous Claude Desktop Code (CCDC)".
- Komt u van de 0.3-scripts: installeer gewoon de `.msi`. Bij de eerste start stopt CCDC de oude versie, neemt de instellingen en accountnamen over en verwijdert de oude versie.

**macOS**: open `CCDC-<version>-macos-universal.dmg`, sleep **CCDC** naar Apps en open het daarna. In de menubalk verschijnt een **C**. CCDC start mee met de computer, tenzij u dat in het menu uitzet. Werkt op Apple Silicon en Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Start daarna **Continuous Claude Desktop Code** vanuit het programmamenu. CCDC start mee met de computer, tenzij u dat in het menu uitzet. Op GNOME zonder de AppIndicator-extensie kan het systeemvakpictogram niet worden weergegeven (Ubuntu en Linux Mint hebben die extensie standaard).

Bij verwijderen blijven de Claude-sessiebestanden, de instellingen van CCDC en de back-upmap staan.

### Over de beveiligingswaarschuwing

Voor ondertekenen is een betaald certificaat nodig dat aan een persoon of bedrijf is gekoppeld, en CCDC is een klein, gratis project, dus de downloads zijn niet ondertekend. Uw systeem waarschuwt u daarom de eerste keer; dat betekent alleen dat de uitgever niet is geverifieerd, niet dat er iets schadelijks is gevonden.

CCDC bevat niets schadelijks. Het maakt geen verbinding met internet, verzamelt geen gegevens en verandert niets buiten de sessiemappen van de Claude-app en zijn eigen map. Alle code staat in deze repository, en de releasebestanden worden daaruit gebouwd door GitHub Actions, waarvan de logboeken openbaar zijn. U kunt een download controleren aan de hand van `SHA256SUMS.txt` in de release, of [het zelf bouwen](#bouwen).

- **Windows**: meldt SmartScreen "Windows heeft uw pc beschermd", klik dan op **Meer informatie** en daarna op **Toch uitvoeren**.
- **macOS**: meldt macOS dat de app niet kan worden geverifieerd, open dan **Systeeminstellingen > Privacy en beveiliging**, scrol omlaag en klik naast CCDC op **Open toch**. (Of voer `xattr -dr com.apple.quarantine /Applications/CCDC.app` uit in Terminal.)
- **Linux**: `.deb`-pakketten worden niet op deze manier gecontroleerd.

## Systeemvak-app

In het systeemvak of de menubalk verschijnt een oranje **C** (grijs zolang automatische synchronisatie uit staat). Klik erop om het menu te openen:

| Menu-item | Wat het doet |
| --- | --- |
| Statusregel | Toont of CCDC aan- en afmeldingen bewaakt |
| Regels met aanmeldtijd | Eén regel per afgemeld account met hoe lang geleden het zich aanmeldde (bijv. "2 u 13 min geleden aangemeld"), om in te schatten waar het staat in het gebruiksvenster van 5 uur van Claude |
| Automatische synchronisatie (aan-/afmelden detecteren) | Zet het kopiëren en opruimen aan of uit |
| Sessies nu naar een account verplaatsen | Zichtbaar als "Kopieën verwijderen na aanmelden" aan staat. Voor als een afmelding is gemist: verzamelt alle sessies in de map van één account; vraagt eerst om bevestiging als dat account niet is aangemeld |
| Alle accounts nu synchroniseren | Zichtbaar op dezelfde plek als "Kopieën verwijderen na aanmelden" uit staat. Voor als een afmelding is gemist: kopieert alle sessies naar de map van elk account, net als bij afmelden |
| Kopieën verwijderen na aanmelden | Standaard uit (de kopieën blijven bewaard; zie hierboven) |
| Snelheid van kopiëren/verwijderen | Snel (geen limiet), Normaal (60 MB/s) of Rustig (15 MB/s). Bij Normaal en Rustig is verwijderen beperkt tot respectievelijk 500 en 100 bestanden per seconde |
| Schattingsregel | Toont hoe lang het kopiëren bij de volgende afmelding zou duren en, als kopieën worden verwijderd, ook het opruimen na aanmelden |
| Starten met de computer | Start CCDC wanneer u zich aanmeldt bij de computer |
| Language | Kiest de weergavetaal. Dit item staat altijd in het Engels; elke taal in het submenu staat in die taal |
| Logboek bekijken | Opent `watch.log` van CCDC |
| Accounts een naam geven (ccdc.json) | Opent het instellingenbestand om accounts leesbare namen te geven |
| Afsluiten | Verwijdert het pictogram; loopt er een kopieerbewerking, dan wacht het eerst tot die klaar is |

Er verschijnen meldingen wanneer het kopiëren begint (met de geschatte tijd), wanneer het klaar is, wanneer een aanmelding wordt gedetecteerd en wanneer het opruimen klaar is. Bij een lagere snelheid duurt het kopiëren langer. De volgende persoon moet daarom de melding "Kopiëren voltooid" afwachten, of ongeveer de geschatte tijd, en zich pas daarna aanmelden.

De schatting wordt berekend uit de huidige stand van de mappen en uit de kopieer- en verwijdersnelheden die op deze computer zijn gemeten (opgeslagen in `state.json` in de map van CCDC).

Ziet u het pictogram niet in Windows 11, kijk dan bij de verborgen pictogrammen (`^`) op de taakbalk. Om het zichtbaar te houden, zet u CCDC aan via Instellingen > Persoonlijke instellingen > Taakbalk > Andere pictogrammen in het systeemvak.

CCDC is één systeemeigen programma van ongeveer 1 MB. Terwijl het wacht, leest het elke halve seconde alleen wat sinds de vorige controle aan het logboek van Claude is toegevoegd, en gebruikt het zelf ongeveer 4 MB geheugen.

## Instellingen

Instellingen staan in `ccdc.json` in de map van CCDC:

| | Map van CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Wat u in het menu van het systeemvak wijzigt, wordt in dit bestand opgeslagen en geldt vanaf het volgende kopiëren of opruimen, zonder dat u iets opnieuw hoeft te starten. `watch.log` en `state.json` staan in dezelfde map.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Sleutel | Waarden |
| --- | --- |
| `keepCopies` | `true` (standaard): de kopieën blijven bewaard. `false`: ze worden na aanmelden verwijderd |
| `speed` | `"fast"` (standaard), `"normal"`, `"gentle"` |
| `language` | Leeg om de systeemtaal te volgen, of een taalcode zoals `"ja"` om die taal te gebruiken |
| `startAtLogin` | Wordt ingesteld met "Starten met de computer" in het menu van het systeemvak |
| `aliases` | Namen die in het menu van het systeemvak voor accounts worden getoond |
| `dataDir`, `appLog` | Alleen als CCDC de gegevensmap of het logboek van Claude niet zelf kan vinden: het volledige pad ervan |

## Talen

Het menu van het systeemvak, de meldingen en de waarschuwingen volgen de weergavetaal van het systeem, of de taal die u kiest onder **Language** in het menu. Ondersteund: Engels, Koreaans, Japans, Chinees (vereenvoudigd en traditioneel), Spaans, Frans, Duits, Italiaans, Portugees, Russisch, Oekraïens, Pools, Nederlands, Turks, Vietnamees, Indonesisch, Thai, Arabisch en Hindi. Andere talen worden in het Engels getoond.

Alle teksten van de interface staan in één werkblad, [`locales.csv`](locales.csv), dat in het programma is ingebouwd: **één rij per taal**, één kolom per bericht. Open het in Excel of Google Spreadsheets; om een taal toe te voegen, voegt u een rij toe en [bouwt](#bouwen) u CCDC opnieuw.

- `lang`: een taaltag (`sv`, `pt-BR`, `zh-Hant`, …). Een rij die bij taal en regio past, heeft de voorkeur, daarna een rij die bij de taal past.
- `name`: de naam van de taal in die taal (`Svenska`); het menu Language toont die.
- Schrijf regeleinden in een cel als `\n` en laat `{0}`, `{1}` staan (daar komen getallen of namen).
- Lege cellen worden in het Engels getoond. Sla het bestand op als UTF-8-CSV.

De uitvoer van de opdrachtregel en `watch.log` blijven in het Engels.

## Opdrachtregel

Hetzelfde programma werkt ook vanuit een terminal (op Windows `ccdc.exe` in `%LOCALAPPDATA%\Programs\CCDC`; op macOS `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, last sign-in times, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

Met `--data-dir <path>` en `--app-log <path>` geeft u zelf een gegevensmap of logboek van Claude op.

## Regels bij dubbele sessies

- Staat een sessie in meerdere mappen, dan wint de kopie met de nieuwste `lastActivityAt`. Kopieën behouden de wijzigingstijd van de bron, zodat identieke kopieën op grootte en tijd worden herkend zonder dat ze gelezen hoeven te worden.
- Vervangt een nieuwere kopie een bestand, dan wordt het oude bestand verplaatst naar `ccdc-backups/<time>/` in de gegevensmap van Claude.
- Sessies met een markering `deleted_<id>` worden niet gekopieerd en hun kopieën in andere mappen worden verwijderd.
- Een account of organisatie die zich voor de allereerste keer aanmeldt, heeft nog geen map, dus daar kon bij het afmelden niets naartoe worden gekopieerd. De sessies verschijnen dan na de volgende aanmelding van dat account of na het opnieuw starten van de app.

## Kanttekeningen

- CCDC is afhankelijk van de interne bestandsindeling en de logboektekst van de Claude-desktopapp; een app-update kan het gedrag veranderen.
- Sessiebestanden bevatten connectorinstellingen (MCP) per account, dus sommige connectors maken mogelijk geen verbinding wanneer een sessie onder een ander account wordt geopend.
- Geplande taken worden niet gedeeld; elke account-/organisatiemap behoudt zijn eigen taken.
- Wisselen van account in de Claude Code-CLI wordt niet ondersteund. De CLI kan elke sessie in `~/.claude/projects` al openen met `--resume`, ongeacht het account.

## Bouwen

CCDC is geschreven in Rust; [installeer Rust](https://rustup.rs) en voer daarna uit:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

De versie per platform staat in `[package.metadata.ccdc]` in `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) bouwt alle drie bij elke push en voegt ze bij een `v*`-tag toe aan een conceptrelease.

## Licentie

[MIT](LICENSE)
