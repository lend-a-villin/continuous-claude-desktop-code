# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | **Nederlands** | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Een Windows-app voor het systeemvak die de lokale **Code**-sessies van de Claude-desktopapp beschikbaar houdt voor elk Claude-account dat zich op de pc aanmeldt.

Onofficieel; niet verbonden aan of goedgekeurd door Anthropic.

## Voor wie CCDC bedoeld is

CCDC is ontwikkeld voor **bedrijven die 24 uur per dag in drie ploegen werken**. In elke ploeg zit iemand anders aan dezelfde werkplek (één pc, één Windows-gebruiker) en meldt zich aan bij de Claude-desktopapp met een eigen Claude-account. Met CCDC meldt de volgende ploeg zich aan en gaat verder met het Code-werk waar de vorige ploeg was gebleven.

Dit betekent dat alle Code-sessies op de pc zichtbaar voor iedereen die zich erop aanmeldt. Gebruik CCDC niet waar mensen elkaars sessies niet mogen zien. Controleer bij een bedrijfsaccount (organisatie) ook het beleid van uw bedrijf.

## Welk probleem CCDC oplost

De desktopapp bewaart de **lijst** met Code-sessies in een aparte map per account en organisatie:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

De **gesprekken** zelf staan in `~/.claude/projects\...\<cliSessionId>.jsonl`, een locatie die alle accounts delen. Wie zich met een ander account aanmeldt, ziet daarom een lege lijst met Code-sessies, terwijl de gesprekken gewoon nog op de schijf staan.

## Werking

De app leest zijn sessiemap **alleen bij het starten en bij het aanmelden** (bij het aanmelden twee keer, binnen ongeveer twee seconden). Er is geen manier om later alleen de lijst opnieuw te laten laden. Het eigen logboek van de app, `%LOCALAPPDATA%\Claude\Logs\main.log`, legt zowel afmeldingen als aanmeldingen vast:

- afmelden: `[account] Login-state transition (loggedOut: false → true ...)`
- aanmelden: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC volgt dat logboek en doet het volgende:

1. **Afmelden** → de nieuwste versie van elke sessie wordt naar alle account-/organisatiemappen gekopieerd, zodat het volgende account dat zich aanmeldt, de sessies al in zijn eigen map vindt. Alleen sessies die in een map ontbreken of verouderd zijn, worden gekopieerd.
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

## Vereisten

- Windows 10 of 11, met de Claude-desktopapp (MSIX-installatie)
- [Node.js](https://nodejs.org) 18 of nieuwer

## Installatie

Download het installatiepakket via [Releases](../../releases), pak het uit en voer `setup.cmd` uit. Beheerdersrechten zijn niet nodig.

- Installeert in `%LOCALAPPDATA%\Programs\CCDC`.
- Maakt een snelkoppeling `Continuous Claude Desktop Code` op het bureaublad en in het menu Start.
- Verschijnt in Instellingen > Apps > Geïnstalleerde apps als "Continuous Claude Desktop Code (CCDC)"; daar kunt u het ook verwijderen.
- Start direct na de installatie in het systeemvak.
- Met `setup.cmd /startup` start CCDC ook wanneer u zich aanmeldt bij Windows; met `setup.cmd /nostartup` zet u dat weer uit. Zonder een van beide opties behoudt een nieuwe installatie de eerdere instelling.
- Opnieuw installeren stopt CCDC als het actief is en vervangt de programmabestanden; uw instellingen (`ccdc.json`) blijven bewaard.
- Bij verwijderen blijven de Claude-sessiebestanden en de back-upmap van CCDC staan.

Om het in plaats daarvan rechtstreeks uit deze repository te gebruiken, voert u `ccdc-tray.vbs` uit of gebruikt u de opdrachtregel hieronder.

## Systeemvak-app

In het systeemvak verschijnt een oranje **C** (grijs zolang automatische synchronisatie uit staat). Klik erop om het menu te openen:

| Menu-item | Wat het doet |
| --- | --- |
| Statusregel | Toont of CCDC aan- en afmeldingen bewaakt |
| Automatische synchronisatie (aan-/afmelden detecteren) | Zet het kopiëren en opruimen aan of uit |
| Sessies nu naar een account verplaatsen | Verzamelt alle sessies in de map van één account; vraagt eerst om bevestiging als dat account niet is aangemeld |
| Kopieën verwijderen na aanmelden | Standaard uit (de kopieën blijven bewaard; zie hierboven) |
| Snelheid van kopiëren/verwijderen | Snel (geen limiet), Normaal (60 MB/s) of Rustig (15 MB/s). Bij Normaal en Rustig is verwijderen beperkt tot respectievelijk 500 en 100 bestanden per seconde |
| Schattingsregel | Toont hoe lang het kopiëren bij de volgende afmelding zou duren en, als kopieën worden verwijderd, ook het opruimen na aanmelden |
| Language | Kiest de weergavetaal. Dit item staat altijd in het Engels; elke taal in het submenu staat in die taal |
| Logboek bekijken | Opent `%LOCALAPPDATA%\CCDC\watch.log` |
| Accounts een naam geven (ccdc.json) | Opent het instellingenbestand om accounts leesbare namen te geven |
| Afsluiten | Verwijdert het pictogram; loopt er een kopieerbewerking, dan wacht het eerst tot die klaar is |

Er verschijnen meldingen wanneer het kopiëren begint (met de geschatte tijd), wanneer het klaar is, wanneer een aanmelding wordt gedetecteerd en wanneer het opruimen klaar is. Bij een lagere snelheid duurt het kopiëren langer. De volgende persoon moet daarom de melding 'Kopiëren voltooid' afwachten, of ongeveer de geschatte tijd, en zich pas daarna aanmelden.

De schatting wordt berekend uit de huidige stand van de mappen en uit de kopieer- en verwijdersnelheden die op deze pc zijn gemeten (opgeslagen in `%LOCALAPPDATA%\CCDC\state.json`).

Ziet u het pictogram niet in Windows 11, kijk dan bij de verborgen pictogrammen (`^`) op de taakbalk. Om het zichtbaar te houden, zet u Windows PowerShell aan via Instellingen > Persoonlijke instellingen > Taakbalk > Andere pictogrammen in het systeemvak.

### Geheugengebruik

De systeemvak-app bewaakt zelf het logboek van de app en leest alleen wat sinds de vorige controle is toegevoegd. Node.js draait alleen tijdens het kopiëren of opruimen en stopt daarna, dus tussen ploegwissels draait alleen de systeemvak-app. In rust geeft die ongeveer eens per minuut ongebruikt geheugen terug aan Windows (zo'n 15 MB in gebruik).

## Instellingen

Instellingen staan in `ccdc.json` in de programmamap. Wat u in het menu van de systeemvak-app wijzigt, wordt hierin opgeslagen en geldt vanaf het volgende kopiëren of opruimen, zonder dat u iets opnieuw hoeft te starten.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Sleutel | Waarden |
| --- | --- |
| `keepCopies` | `true` (standaard): de kopieën blijven bewaard. `false`: ze worden na aanmelden verwijderd |
| `speed` | `"fast"` (standaard), `"normal"`, `"gentle"` |
| `language` | Leeg om de taal van Windows te volgen, of een taalcode zoals `"ja"` om die taal te gebruiken |
| `aliases` | Namen die in het menu in het systeemvak voor accounts worden getoond |

## Talen

Het menu in het systeemvak, de meldingen, waarschuwingen en de installatie-/verwijdervensters volgen de weergavetaal van Windows, of de taal die u kiest onder **Language** in het menu. Ondersteund: Engels, Koreaans, Japans, Chinees (vereenvoudigd en traditioneel), Spaans, Frans, Duits, Italiaans, Portugees, Russisch, Oekraïens, Pools, Nederlands, Turks, Vietnamees, Indonesisch, Thai, Arabisch en Hindi. Andere talen worden in het Engels getoond.

Alle teksten van de interface staan in één werkblad, [`locales.csv`](locales.csv): **één rij per taal**, één kolom per bericht. Open het in Excel of Google Spreadsheets; om een taal toe te voegen, voegt u een rij toe.

- `lang`: een Windows-cultuurnaam (`sv`, `pt-BR`, `zh-Hant`, …). Een rij die bij taal en regio past, heeft de voorkeur, daarna een rij die bij de taal past.
- `name`: de naam van de taal in die taal (`Svenska`); het menu Language toont die.
- Schrijf regeleinden in een cel als `\n` en laat `{0}`, `{1}` staan (daar komen getallen of namen).
- Lege cellen worden in het Engels getoond. Sla het bestand op als UTF-8-CSV.

De uitvoer van de opdrachtregel en `watch.log` blijven in het Engels, omdat de systeemvak-app ze leest.

## Opdrachtregel

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

De systeemvak-app voert zelf deze eenmalige taken uit: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` en `node ccdc.mjs startup` (om de huidige stand van de Claude-app bij te halen wanneer de systeemvak-app start of automatische synchronisatie weer wordt aangezet).

## Regels bij dubbele sessies

- Staat een sessie in meerdere mappen, dan wint de kopie met de nieuwste `lastActivityAt`. Kopieën behouden de wijzigingstijd van de bron, zodat identieke kopieën op grootte en tijd worden herkend zonder dat ze gelezen hoeven te worden.
- Vervangt een nieuwere kopie een bestand, dan wordt het oude bestand verplaatst naar `ccdc-backups\<tijd>\` in de gegevensmap van de app.
- Sessies met een markering `deleted_<id>` worden niet gekopieerd en hun kopieën in andere mappen worden verwijderd.
- Een account of organisatie die zich voor de allereerste keer aanmeldt, heeft nog geen map, dus daar kon bij het afmelden niets naartoe worden gekopieerd. De sessies verschijnen dan na de volgende aanmelding van dat account of na het opnieuw starten van de app.

## Kanttekeningen

- CCDC is afhankelijk van de interne bestandsindeling en de logboektekst van de Claude-desktopapp; een app-update kan het gedrag veranderen.
- Sessiebestanden bevatten connectorinstellingen (MCP) per account, dus sommige connectors maken mogelijk geen verbinding wanneer een sessie onder een ander account wordt geopend.
- Geplande taken worden niet gedeeld; elke account-/organisatiemap behoudt zijn eigen taken.
- Wisselen van account in de Claude Code-CLI wordt niet ondersteund. De CLI kan elke sessie in `~/.claude/projects` al openen met `--resume`, ongeacht het account.

## Licentie

[MIT](LICENSE)
