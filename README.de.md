# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | **Deutsch** | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Eine Windows-App für den Infobereich, die die lokalen **Code**-Sitzungen der Claude-Desktop-App für jedes Claude-Konto verfügbar hält, das sich am PC anmeldet.

Inoffiziell; steht in keiner Verbindung zu Anthropic und wird nicht von Anthropic gebilligt.

## Für wen CCDC gedacht ist

CCDC wurde für **Betriebe entwickelt, die rund um die Uhr in drei Schichten arbeiten**. In jeder Schicht sitzt eine andere Person am selben Arbeitsplatz (ein PC, ein Windows-Benutzer) und meldet sich mit dem eigenen Claude-Konto in der Claude-Desktop-App an. Mit CCDC meldet sich die nächste Schicht an und setzt die Code-Arbeit dort fort, wo die vorherige Schicht aufgehört hat.

Das bedeutet: Jede Code-Sitzung auf dem PC ist für alle sichtbar, die sich dort anmelden. Setzen Sie CCDC nicht ein, wo Personen die Sitzungen anderer nicht sehen dürfen. Bei einem Firmenkonto (Organisation) prüfen Sie bitte auch die Richtlinien Ihres Unternehmens.

## Welches Problem CCDC löst

Die Desktop-App legt die **Liste** der Code-Sitzungen für jedes Konto und jede Organisation in einem eigenen Ordner ab:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

Die **Unterhaltungen** selbst liegen in `~/.claude/projects\...\<cliSessionId>.jsonl`, einem Speicherort, den alle Konten gemeinsam nutzen. Wenn sich jemand mit einem anderen Konto anmeldet, wirkt die Liste der Code-Sitzungen deshalb leer, obwohl die Unterhaltungen noch auf dem Datenträger liegen.

## Funktionsweise

Die App liest ihren Sitzungsordner nur **beim Start und bei der Anmeldung** (dabei zweimal innerhalb von etwa zwei Sekunden). Es gibt keine Möglichkeit, die Liste später gezielt neu laden zu lassen. Das eigene Protokoll der App, `%LOCALAPPDATA%\Claude\Logs\main.log`, hält An- und Abmeldungen fest:

- Abmeldung: `[account] Login-state transition (loggedOut: false → true ...)`
- Anmeldung: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC verfolgt dieses Protokoll und reagiert so:

1. **Abmeldung** → CCDC kopiert die neueste Version jeder Sitzung in jeden Konto-/Organisationsordner, sodass das nächste Konto, das sich anmeldet, die Sitzungen bereits im eigenen Ordner vorfindet. Kopiert werden nur Sitzungen, die in einem Ordner fehlen oder veraltet sind.
2. **Anmeldung** → Standardmäßig bleiben die Kopien, wo sie sind. Ist „Kopien nach der Anmeldung löschen“ eingeschaltet, wartet CCDC, bis die App ihre Liste gelesen hat (6 Sekunden nach der letzten Anmeldezeile), und entfernt dann die Kopien überall außer im Ordner des angemeldeten Kontos.

Eine in der App gelöschte Sitzung wird auch aus den anderen Ordnern entfernt, damit sie unter einem anderen Konto nicht wieder auftaucht.

Vor dem Kopieren prüft CCDC den freien Speicherplatz. Reicht er für die Kopien plus 512 MB nicht aus, kopiert CCDC nichts und zeigt eine Warnung an.

### Warum Kopien standardmäßig behalten werden

Die Kopien zu behalten, ist die schonendste Variante für eine SSD:

- **Kopien behalten (Standard)**: Die erste Abmeldung kopiert einmal alle Sitzungen. Danach schreibt jede Abmeldung nur die Sitzungen, die sich seit der letzten Abmeldung geändert haben – meist ein paar Dateien und ein paar MB – und ist fast sofort fertig.
- **Kopien nach der Anmeldung löschen**: Jede Anmeldung löscht die Kopien, sodass jede Abmeldung sie alle erneut schreiben muss. Bei etwa 300 Sitzungen und 10 Konto-/Organisationsordnern werden bei jedem Schichtwechsel rund 600 MB geschrieben; anschließend werden außerdem rund 3.000 Dateien gelöscht.

Der Preis dafür ist Speicherplatz: Die Kopien bleiben auf dem Datenträger und belegen etwa (Gesamtgröße der Sitzungsdateien × Anzahl der Konto-/Organisationsordner).

### Warum keine Junctions oder Hardlinks

Wenn jeder Kontoordner auf einen einzigen echten Ordner zeigte, bräuchte man nicht zu kopieren – doch die Claude-App blockiert beide Wege bewusst:

- **Junctions und symbolische Links**: Vor jedem Speichern prüft die App, ob der Sitzungsordner ein echter Ordner ist, und verweigert das Speichern, wenn er ein Link ist (`Refusing non-directory at private dir path (symlink/file plant)`). In einem Test mit der echten App ließ sich die Liste über eine Junction lesen, aber jedes Speichern von Sitzungen und geplanten Aufgaben schlug fehl – auch bei ganz neuen Sitzungen.
- **Hardlinks**: Die App behandelt eine Datei mit mehr als einem Link als unsicher. Sie weigert sich, in solche Dateien zu schreiben, und warnt beim Lesen. Außerdem speichert sie, indem sie eine temporäre Datei schreibt und umbenennt, wodurch der Link bei jedem Speichern aufgehoben würde.

Diese Prüfungen schützen vor von außen untergeschobenen Dateien oder Ordnern; deshalb umgeht CCDC sie nicht. CCDC kopiert gewöhnliche Dateien – dieselbe Art von Dateien, die auch die App selbst schreibt.

## Voraussetzungen

- Windows 10 oder 11 mit der Claude-Desktop-App (MSIX-Installation)
- [Node.js](https://nodejs.org) 18 oder neuer

## Installation

Laden Sie das Installationspaket unter [Releases](../../releases) herunter, entpacken Sie es und führen Sie `setup.cmd` aus. Administratorrechte sind nicht nötig.

- Wird nach `%LOCALAPPDATA%\Programs\CCDC` installiert.
- Legt die Verknüpfung `Continuous Claude Desktop Code` auf dem Desktop und im Startmenü an.
- Erscheint unter Einstellungen > Apps > Installierte Apps als „Continuous Claude Desktop Code (CCDC)“; dort können Sie es auch deinstallieren.
- Startet direkt nach der Installation im Infobereich.
- Mit `setup.cmd /startup` startet CCDC zusätzlich bei der Anmeldung an Windows; mit `setup.cmd /nostartup` schalten Sie das wieder ab. Ohne eine der beiden Optionen bleibt bei einer erneuten Installation die bisherige Einstellung erhalten.
- Eine erneute Installation beendet CCDC, falls es läuft, und ersetzt die Programmdateien; Ihre Einstellungen (`ccdc.json`) bleiben erhalten.
- Beim Deinstallieren bleiben die Claude-Sitzungsdateien und der Sicherungsordner von CCDC bestehen.

Um CCDC stattdessen direkt aus diesem Repository auszuführen, starten Sie `ccdc-tray.vbs` oder verwenden Sie die Befehlszeile weiter unten.

## Infobereich-App

Im Infobereich erscheint ein orangefarbenes **C** (grau, solange die automatische Synchronisierung aus ist). Ein Klick darauf öffnet das Menü:

| Menüeintrag | Funktion |
| --- | --- |
| Statuszeile | Zeigt, ob CCDC An- und Abmeldungen überwacht |
| Automatische Synchronisierung (An-/Abmeldung erkennen) | Schaltet das Kopieren und Aufräumen ein oder aus |
| Sitzungen jetzt in ein Konto verschieben | Sammelt alle Sitzungen im Ordner eines Kontos; fragt vorher nach, wenn dieses Konto nicht angemeldet ist |
| Kopien nach der Anmeldung löschen | Standardmäßig aus (Kopien bleiben erhalten; siehe oben) |
| Kopier-/Löschgeschwindigkeit | Schnell (ohne Grenze), Normal (60 MB/s) oder Schonend (15 MB/s). Bei Normal und Schonend ist das Löschen auf 500 bzw. 100 Dateien pro Sekunde begrenzt |
| Schätzzeile | Zeigt, wie lange das nächste Kopieren bei der Abmeldung dauern würde und, falls Kopien gelöscht werden, auch das Aufräumen nach der Anmeldung |
| Language | Wählt die Anzeigesprache. Dieser Eintrag ist immer auf Englisch beschriftet; jede Sprache im Untermenü ist in ihrer eigenen Sprache geschrieben |
| Protokoll anzeigen | Öffnet `%LOCALAPPDATA%\CCDC\watch.log` |
| Konten benennen (ccdc.json) | Öffnet die Einstellungsdatei, um Konten lesbare Namen zu geben |
| Beenden | Entfernt das Symbol; läuft gerade ein Kopiervorgang, wird dessen Ende abgewartet |

Benachrichtigungen erscheinen, wenn das Kopieren beginnt (mit der geschätzten Dauer), wenn es fertig ist, wenn eine Anmeldung erkannt wird und wenn das Aufräumen abgeschlossen ist. Bei einer langsameren Geschwindigkeit dauert das Kopieren länger. Die nächste Person sollte daher die Benachrichtigung „Kopieren abgeschlossen“ oder etwa die geschätzte Zeit abwarten, bevor sie sich anmeldet.

Die Schätzung wird aus dem aktuellen Zustand der Ordner und aus den auf diesem PC gemessenen Kopier- und Löschgeschwindigkeiten berechnet (gespeichert in `%LOCALAPPDATA%\CCDC\state.json`).

Wenn das Symbol unter Windows 11 nicht erscheint, suchen Sie es unter den ausgeblendeten Symbolen (`^`) in der Taskleiste. Damit es dauerhaft sichtbar bleibt, schalten Sie unter Einstellungen > Personalisierung > Taskleiste > Weitere Taskleistensymbole Windows PowerShell ein.

### Speicherverbrauch

Die Infobereich-App überwacht das Protokoll der App selbst und liest nur, was seit der letzten Prüfung hinzugekommen ist. Node.js läuft nur während des Kopierens oder Aufräumens und beendet sich danach, sodass zwischen den Schichtwechseln nur die Infobereich-App läuft. Im Leerlauf gibt sie etwa einmal pro Minute ungenutzten Speicher an Windows zurück (rund 15 MB belegt).

## Einstellungen

Die Einstellungen stehen in `ccdc.json` im Programmordner. Was Sie im Menü der Infobereich-App ändern, wird dort gespeichert und gilt ohne Neustart ab dem nächsten Kopieren oder Aufräumen.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Schlüssel | Werte |
| --- | --- |
| `keepCopies` | `true` (Standard): Kopien bleiben erhalten. `false`: Kopien werden nach der Anmeldung gelöscht |
| `speed` | `"fast"` (Standard), `"normal"`, `"gentle"` |
| `language` | Leer, um der Windows-Anzeigesprache zu folgen, oder ein Sprachcode wie `"ja"` für diese Sprache |
| `aliases` | Namen, die im Menü für Konten angezeigt werden |

## Sprachen

Das Menü der Infobereich-App, die Benachrichtigungen, die Warnungen und die Dialoge der Installation/Deinstallation richten sich nach der Windows-Anzeigesprache oder nach der Sprache, die im Menü unter **Language** gewählt wurde. Unterstützt werden: Englisch, Koreanisch, Japanisch, Chinesisch (vereinfacht und traditionell), Spanisch, Französisch, Deutsch, Italienisch, Portugiesisch, Russisch, Ukrainisch, Polnisch, Niederländisch, Türkisch, Vietnamesisch, Indonesisch, Thai, Arabisch und Hindi. Andere Sprachen werden auf Englisch angezeigt.

Der gesamte Oberflächentext steht in einer Tabelle, [`locales.csv`](locales.csv): **eine Zeile pro Sprache**, eine Spalte pro Meldung. Öffnen Sie sie in Excel oder Google Sheets; um eine Sprache hinzuzufügen, fügen Sie eine Zeile hinzu.

- `lang`: ein Windows-Kulturname (`sv`, `pt-BR`, `zh-Hant`, …). Eine Zeile, die zu Sprache und Region passt, hat Vorrang, danach eine, die nur zur Sprache passt.
- `name`: der Name der Sprache in dieser Sprache (`Svenska`); das Language-Menü zeigt ihn an.
- Zeilenumbrüche innerhalb einer Zelle schreiben Sie als `\n`; `{0}`, `{1}` bleiben erhalten (dort werden Zahlen oder Namen eingesetzt).
- Leere Zellen werden auf Englisch angezeigt. Speichern Sie die Datei als UTF-8-CSV.

Die Ausgabe der Befehlszeile und `watch.log` bleiben auf Englisch, weil die Infobereich-App sie ausliest.

## Befehlszeile

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Die Infobereich-App führt diese einmaligen Aufgaben selbst aus: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` und `node ccdc.mjs startup` (gleicht mit dem aktuellen Zustand der Claude-App ab, wenn die Infobereich-App startet oder die automatische Synchronisierung wieder eingeschaltet wird).

## Regeln bei Duplikaten

- Liegt eine Sitzung in mehreren Ordnern, gewinnt die Kopie mit dem neuesten `lastActivityAt`. Kopien behalten die Änderungszeit der Quelle, sodass identische Kopien an Größe und Zeit erkannt werden, ohne dass sie gelesen werden müssen.
- Ersetzt eine neuere Kopie eine Datei, wird die alte Datei nach `ccdc-backups\<Zeit>\` im Datenordner der App verschoben.
- Sitzungen mit einer `deleted_<id>`-Markierung werden nicht kopiert, und ihre Kopien in anderen Ordnern werden entfernt.
- Für ein Konto oder eine Organisation, die sich zum allerersten Mal anmeldet, gibt es noch keinen Ordner, daher konnte bei der Abmeldung nichts dorthin kopiert werden. Die Sitzungen erscheinen dann nach der nächsten Anmeldung dieses Kontos oder nach einem Neustart der App.

## Hinweise und Einschränkungen

- CCDC hängt vom internen Dateiaufbau der Claude-Desktop-App und von den Formulierungen in ihrem Protokoll ab; ein App-Update kann das Verhalten ändern.
- Sitzungsdateien enthalten kontospezifische Connector-Einstellungen (MCP), daher verbinden sich manche Connectors womöglich nicht, wenn eine Sitzung unter einem anderen Konto geöffnet wird.
- Geplante Aufgaben werden nicht geteilt; jeder Konto-/Organisationsordner behält seine eigenen.
- Der Kontowechsel in der Claude-Code-CLI wird nicht unterstützt. Die CLI kann ohnehin jede Sitzung in `~/.claude/projects` mit `--resume` öffnen, unabhängig vom Konto.

## Lizenz

[MIT](LICENSE)
