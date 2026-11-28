# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | **Deutsch** | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Eine Tray-App, die Ihre **Code**-Sitzungen der Claude-Desktop-App erhält, wenn Sie das Claude-Konto wechseln.

Die Claude-Desktop-App führt für jedes Konto und jede Organisation eine eigene Liste der Code-Sitzungen. Wer sich mit einem anderen Konto anmeldet, sieht eine leere Liste, obwohl die Unterhaltungen noch auf dem Computer liegen. CCDC kopiert die Sitzungen hinüber, sodass das nächste Konto, das sich anmeldet, alle Sitzungen sieht.

Inoffiziell; steht in keiner Verbindung zu Anthropic und wird nicht von Anthropic gebilligt.

**[Aktuelle Version herunterladen](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · keine weitere Installation nötig

| Plattform | Version | Status |
| --- | --- | --- |
| Windows | 1.1 | Neu; mit der echten Claude-App unter Windows getestet und funktioniert wie beschrieben (Nachfolger der 0.3-Skripte) |
| macOS | 0.2 | Neu; noch nicht mit der echten Claude-App auf einem Mac getestet |
| Linux | 0.2 | Neu; Installation unter Debian 12, Ubuntu 24.04 und Linux Mint 22 getestet, noch nicht mit der echten Claude-App |

## Für wen CCDC gedacht ist

- **Gemeinsam genutzte Computer und Schichtarbeit.** In einem Betrieb, der rund um die Uhr in drei Schichten arbeitet, sitzt in jeder Schicht eine andere Person am selben Arbeitsplatz (ein Computer, ein Systembenutzer) und meldet sich mit dem eigenen Claude-Konto an. Mit CCDC setzt die nächste Schicht die Code-Arbeit dort fort, wo die vorherige aufgehört hat.
- **Eine Person, mehrere Konten.** Wenn Sie auf demselben Computer zwischen einem privaten und einem beruflichen Konto oder zwischen Organisationen wechseln, sind Ihre Code-Sitzungen da, egal mit welchem Konto Sie sich anmelden.

Jede Code-Sitzung auf dem Computer wird für jedes Konto sichtbar, das sich dort anmeldet. Setzen Sie CCDC nicht ein, wo Personen die Sitzungen anderer nicht sehen dürfen. Wenn Sie ein Firmenkonto (Organisation) verwenden, prüfen Sie vorher die Richtlinien Ihres Unternehmens: CCDC kopiert die Sitzungen eines beruflichen Kontos auch in den Ordner eines privaten Kontos und umgekehrt.

## Welches Problem CCDC löst

Die Desktop-App legt die **Liste** der Code-Sitzungen für jedes Konto und jede Organisation in einem eigenen Ordner ab:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Claude-Datenordner | Claude-Protokoll |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (oder `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Organisationen, die Claude über ihren eigenen Cloud-Anbieter betreiben, verwenden `Claude-3p` statt `Claude`; CCDC sucht auch dort.)

Die **Unterhaltungen** selbst liegen in `~/.claude/projects/.../<cliSessionId>.jsonl`, einem Speicherort, den alle Konten gemeinsam nutzen. Wenn sich jemand mit einem anderen Konto anmeldet, wirkt die Liste der Code-Sitzungen deshalb leer, obwohl die Unterhaltungen noch auf dem Datenträger liegen.

## Funktionsweise

Die App liest ihren Sitzungsordner nur **beim Start und bei der Anmeldung** (dabei zweimal innerhalb von etwa zwei Sekunden). Es gibt keine Möglichkeit, die Liste später gezielt neu laden zu lassen. Das eigene Protokoll der App hält beide Ereignisse fest:

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

Der Preis dafür ist Speicherplatz: Die Kopien bleiben liegen und belegen etwa (Gesamtgröße der Sitzungsdateien × Anzahl der Konto-/Organisationsordner).

### Warum keine Junctions oder Hardlinks

Wenn jeder Kontoordner auf einen einzigen echten Ordner zeigte, bräuchte man nicht zu kopieren – doch die Claude-App blockiert beide Wege bewusst:

- **Junctions und symbolische Links**: Vor jedem Speichern prüft die App, ob der Sitzungsordner ein echter Ordner ist, und verweigert das Speichern, wenn er ein Link ist (`Refusing non-directory at private dir path (symlink/file plant)`). In einem Test mit der echten App ließ sich die Liste über eine Junction lesen, aber jedes Speichern von Sitzungen und geplanten Aufgaben schlug fehl – auch bei ganz neuen Sitzungen.
- **Hardlinks**: Die App behandelt eine Datei mit mehr als einem Link als unsicher. Sie weigert sich, in solche Dateien zu schreiben, und warnt beim Lesen. Außerdem speichert sie, indem sie eine temporäre Datei schreibt und umbenennt, wodurch der Link bei jedem Speichern aufgehoben würde.

Diese Prüfungen schützen vor von außen untergeschobenen Dateien oder Ordnern; deshalb umgeht CCDC sie nicht. CCDC kopiert gewöhnliche Dateien – dieselbe Art von Dateien, die auch die App selbst schreibt.

## Installation

Laden Sie die Datei für Ihr System unter [Releases](../../releases) herunter. Die Dateien sind nicht codesigniert (siehe [unten](#hinweis-zur-sicherheitswarnung)).

**Windows** – Führen Sie `CCDC-<version>-windows-x64.msi` aus. Administratorrechte sind nicht nötig.

- Installiert CCDC nach `%LOCALAPPDATA%\Programs\CCDC`, legt die Verknüpfung `Continuous Claude Desktop Code` auf dem Desktop und im Startmenü an und startet CCDC im Infobereich.
- Startet mit dem Computer; abschalten können Sie das mit „Mit dem Computer starten“ im Menü der Tray-App.
- Aktualisieren: Führen Sie die neuere `.msi` aus. Sie beendet CCDC, ersetzt es und startet es wieder; Ihre Einstellungen bleiben erhalten.
- Deinstallieren unter Einstellungen > Apps > Installierte Apps > „Continuous Claude Desktop Code (CCDC)“.
- Umstieg von den 0.3-Skripten: Installieren Sie einfach die `.msi`. Beim ersten Start beendet CCDC die alte Version, übernimmt deren Einstellungen und Kontonamen und entfernt sie.

**macOS** – Öffnen Sie `CCDC-<version>-macos-universal.dmg`, ziehen Sie **CCDC** in den Ordner „Programme“ und öffnen Sie es dann. In der Menüleiste erscheint ein **C**. CCDC startet mit dem Computer, sofern Sie das nicht in seinem Menü abschalten. Läuft auf Apple Silicon und Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Starten Sie anschließend **Continuous Claude Desktop Code** aus dem Anwendungsmenü. CCDC startet mit dem Computer, sofern Sie das nicht in seinem Menü abschalten. Unter GNOME ohne die Erweiterung AppIndicator kann das Tray-Symbol nicht angezeigt werden (Ubuntu und Linux Mint bringen sie mit).

Beim Deinstallieren bleiben die Claude-Sitzungsdateien, die Einstellungen von CCDC und der Sicherungsordner bestehen.

### Hinweis zur Sicherheitswarnung

Zum Signieren braucht man ein kostenpflichtiges Zertifikat, das an eine Person oder ein Unternehmen gebunden ist. CCDC ist ein kleines kostenloses Projekt, daher sind die Downloads nicht signiert. Ihr System warnt Sie beim ersten Mal; das bedeutet nur, dass der Herausgeber nicht verifiziert ist, nicht, dass etwas Schädliches gefunden wurde.

CCDC enthält nichts Schädliches. Es verbindet sich nicht mit dem Internet, sammelt keine Daten und ändert nichts außerhalb der Sitzungsordner der Claude-App und seines eigenen Ordners. Der gesamte Code liegt in diesem Repository, und die Release-Dateien werden daraus von GitHub Actions gebaut, dessen Protokolle öffentlich sind. Sie können einen Download mit `SHA256SUMS.txt` aus dem Release abgleichen oder [CCDC selbst bauen](#aus-dem-quellcode-bauen).

- **Windows**: Meldet SmartScreen „Der Computer wurde durch Windows geschützt“, klicken Sie auf **Weitere Informationen** und dann auf **Trotzdem ausführen**.
- **macOS**: Meldet macOS, dass die App nicht überprüft werden kann, öffnen Sie **Systemeinstellungen > Datenschutz & Sicherheit**, scrollen Sie nach unten und klicken Sie neben CCDC auf **Dennoch öffnen**. (Oder führen Sie im Terminal `xattr -dr com.apple.quarantine /Applications/CCDC.app` aus.)
- **Linux**: `.deb`-Pakete werden nicht auf diese Weise geprüft.

## Tray-App

Im Infobereich bzw. in der Menüleiste erscheint ein orangefarbenes **C** (grau, solange die automatische Synchronisierung aus ist). Ein Klick darauf öffnet das Menü:

| Menüeintrag | Funktion |
| --- | --- |
| Statuszeile | Zeigt, ob CCDC An- und Abmeldungen überwacht |
| Anmeldezeit-Zeilen | Eine Zeile pro abgemeldetem Konto, wie lange seine Anmeldung her ist (z. B. „vor 2 Std. 13 Min. angemeldet“), um abzuschätzen, wo es im 5-Stunden-Nutzungsfenster von Claude steht |
| Automatische Synchronisierung (An-/Abmeldung erkennen) | Schaltet das Kopieren und Aufräumen ein oder aus |
| Sitzungen jetzt in ein Konto verschieben | Erscheint, wenn „Kopien nach der Anmeldung löschen“ eingeschaltet ist. Für eine nicht erkannte Abmeldung: Sammelt alle Sitzungen im Ordner eines Kontos; fragt vorher nach, wenn dieses Konto nicht angemeldet ist |
| Alle Konten jetzt synchronisieren | Erscheint an derselben Stelle, wenn „Kopien nach der Anmeldung löschen“ ausgeschaltet ist. Für eine nicht erkannte Abmeldung: kopiert alle Sitzungen in den Ordner jedes Kontos, wie bei einer Abmeldung |
| Kopien nach der Anmeldung löschen | Standardmäßig aus (Kopien bleiben erhalten; siehe oben) |
| Kopier-/Löschgeschwindigkeit | Schnell (ohne Grenze), Normal (60 MB/s) oder Schonend (15 MB/s). Bei Normal und Schonend ist das Löschen auf 500 bzw. 100 Dateien pro Sekunde begrenzt |
| Schätzzeile | Zeigt, wie lange das nächste Kopieren bei der Abmeldung dauern würde und, falls Kopien gelöscht werden, auch das Aufräumen nach der Anmeldung |
| Mit dem Computer starten | Startet CCDC, wenn Sie sich am Computer anmelden |
| Language | Wählt die Anzeigesprache. Dieser Eintrag ist immer auf Englisch beschriftet; jede Sprache im Untermenü ist in ihrer eigenen Sprache geschrieben |
| Protokoll anzeigen | Öffnet `watch.log` von CCDC |
| Konten benennen (ccdc.json) | Öffnet die Einstellungsdatei, um Konten lesbare Namen zu geben |
| Beenden | Entfernt das Symbol; läuft gerade ein Kopiervorgang, wird dessen Ende abgewartet |

Benachrichtigungen erscheinen, wenn das Kopieren beginnt (mit der geschätzten Dauer), wenn es fertig ist, wenn eine Anmeldung erkannt wird und wenn das Aufräumen abgeschlossen ist. Bei einer langsameren Geschwindigkeit dauert das Kopieren länger. Die nächste Person sollte daher die Benachrichtigung „Kopieren abgeschlossen“ oder etwa die geschätzte Zeit abwarten, bevor sie sich anmeldet.

Die Schätzung wird aus dem aktuellen Zustand der Ordner und aus den auf diesem Computer gemessenen Kopier- und Löschgeschwindigkeiten berechnet (gespeichert in `state.json` im Ordner von CCDC).

Wenn das Symbol unter Windows 11 nicht erscheint, suchen Sie es unter den ausgeblendeten Symbolen (`^`) in der Taskleiste. Damit es dauerhaft sichtbar bleibt, schalten Sie unter Einstellungen > Personalisierung > Taskleiste > Weitere Taskleistensymbole CCDC ein.

CCDC ist ein einzelnes natives Programm von etwa 1 MB. Während es wartet, liest es jede halbe Sekunde nur das, was seit der letzten Prüfung im Claude-Protokoll hinzugekommen ist, und belegt selbst etwa 4 MB Arbeitsspeicher.

## Einstellungen

Die Einstellungen stehen in `ccdc.json` im Ordner von CCDC:

| | Ordner von CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Was Sie im Menü der Tray-App ändern, wird in dieser Datei gespeichert und gilt ohne Neustart ab dem nächsten Kopieren oder Aufräumen. `watch.log` und `state.json` liegen im selben Ordner.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Schlüssel | Werte |
| --- | --- |
| `keepCopies` | `true` (Standard): Kopien bleiben erhalten. `false`: Kopien werden nach der Anmeldung gelöscht |
| `speed` | `"fast"` (Standard), `"normal"`, `"gentle"` |
| `language` | Leer, um der Systemsprache zu folgen, oder ein Sprachcode wie `"ja"` für diese Sprache |
| `startAtLogin` | Wird über „Mit dem Computer starten“ im Menü der Tray-App gesetzt |
| `aliases` | Namen, die im Menü der Tray-App für Konten angezeigt werden |
| `dataDir`, `appLog` | Nur nötig, wenn CCDC den Claude-Datenordner oder das Protokoll nicht selbst findet: deren vollständige Pfade |

## Sprachen

Das Menü der Tray-App, die Benachrichtigungen und die Warnungen richten sich nach der Anzeigesprache des Systems oder nach der Sprache, die im Menü unter **Language** gewählt wurde. Unterstützt werden: Englisch, Koreanisch, Japanisch, Chinesisch (vereinfacht und traditionell), Spanisch, Französisch, Deutsch, Italienisch, Portugiesisch, Russisch, Ukrainisch, Polnisch, Niederländisch, Türkisch, Vietnamesisch, Indonesisch, Thai, Arabisch und Hindi. Andere Sprachen werden auf Englisch angezeigt.

Der gesamte Oberflächentext steht in einer Tabelle, [`locales.csv`](locales.csv), die in das Programm eingebaut wird: **eine Zeile pro Sprache**, eine Spalte pro Meldung. Öffnen Sie sie in Excel oder Google Sheets; um eine Sprache hinzuzufügen, ergänzen Sie eine Zeile und [bauen](#aus-dem-quellcode-bauen) CCDC neu.

- `lang`: ein Sprach-Tag (`sv`, `pt-BR`, `zh-Hant`, …). Eine Zeile, die zu Sprache und Region passt, hat Vorrang, danach eine, die nur zur Sprache passt.
- `name`: der Name der Sprache in dieser Sprache (`Svenska`); das Language-Menü zeigt ihn an.
- Zeilenumbrüche innerhalb einer Zelle schreiben Sie als `\n`; `{0}`, `{1}` bleiben erhalten (dort werden Zahlen oder Namen eingesetzt).
- Leere Zellen werden auf Englisch angezeigt. Speichern Sie die Datei als UTF-8-CSV.

Die Ausgabe der Befehlszeile und `watch.log` bleiben auf Englisch.

## Befehlszeile

Dasselbe Programm lässt sich auch im Terminal verwenden (unter Windows `ccdc.exe` in `%LOCALAPPDATA%\Programs\CCDC`, unter macOS `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, last sign-in times, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

Mit `--data-dir <path>` und `--app-log <path>` geben Sie den Claude-Datenordner bzw. das Protokoll von Hand an.

## Regeln bei Duplikaten

- Liegt eine Sitzung in mehreren Ordnern, gewinnt die Kopie mit dem neuesten `lastActivityAt`. Kopien behalten die Änderungszeit der Quelle, sodass identische Kopien an Größe und Zeit erkannt werden, ohne dass sie gelesen werden müssen.
- Ersetzt eine neuere Kopie eine Datei, wird die alte Datei nach `ccdc-backups/<time>/` im Claude-Datenordner verschoben.
- Sitzungen mit einer `deleted_<id>`-Markierung werden nicht kopiert, und ihre Kopien in anderen Ordnern werden entfernt.
- Für ein Konto oder eine Organisation, die sich zum allerersten Mal anmeldet, gibt es noch keinen Ordner, daher konnte bei der Abmeldung nichts dorthin kopiert werden. Die Sitzungen erscheinen dann nach der nächsten Anmeldung dieses Kontos oder nach einem Neustart der App.

## Hinweise und Einschränkungen

- CCDC hängt vom internen Dateiaufbau der Claude-Desktop-App und von den Formulierungen in ihrem Protokoll ab; ein App-Update kann das Verhalten ändern.
- Sitzungsdateien enthalten kontospezifische Connector-Einstellungen (MCP), daher verbinden sich manche Connectors womöglich nicht, wenn eine Sitzung unter einem anderen Konto geöffnet wird.
- Geplante Aufgaben werden nicht geteilt; jeder Konto-/Organisationsordner behält seine eigenen.
- Der Kontowechsel in der Claude-Code-CLI wird nicht unterstützt. Die CLI kann ohnehin jede Sitzung in `~/.claude/projects` mit `--resume` öffnen, unabhängig vom Konto.

## Aus dem Quellcode bauen

CCDC ist in Rust geschrieben; [installieren Sie Rust](https://rustup.rs) und führen Sie dann Folgendes aus:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

Die Version jeder Plattform ist in `[package.metadata.ccdc]` in `Cargo.toml` festgelegt. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) baut bei jedem Push alle drei Pakete und hängt sie bei einem `v*`-Tag an einen Release-Entwurf an.

## Lizenz

[MIT](LICENSE)
