# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | **Italiano** | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Un'app per l'area di notifica che conserva le sessioni **Code** dell'app desktop di Claude quando cambi account Claude.

L'app desktop di Claude tiene un elenco separato di sessioni Code per ogni account e organizzazione. Se accedi con un altro account, l'elenco appare vuoto, anche se le conversazioni sono ancora sul computer. CCDC copia le sessioni da una cartella all'altra, così qualunque account acceda dopo le vede tutte.

Strumento non ufficiale; non affiliato ad Anthropic né approvato da Anthropic.

**[Scarica l'ultima versione](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · nient'altro da installare

| Piattaforma | Versione | Stato |
| --- | --- | --- |
| Windows | 1.1 | Nuova; provata con la vera app Claude su Windows, dove funziona come descritto (prosegue gli script 0.3) |
| macOS | 0.2 | Nuova; non ancora provata con la vera app Claude su un Mac |
| Linux | 0.2 | Nuova; installazione provata su Debian 12, Ubuntu 24.04 e Linux Mint 22, non ancora con la vera app Claude |

## A chi è rivolto

- **Computer condivisi e lavoro a turni.** In un luogo di lavoro attivo 24 ore su 24 con tre turni, a ogni turno una persona diversa si siede alla stessa postazione (un computer, un utente di sistema) e accede con il proprio account Claude. Con CCDC, chi arriva al turno successivo riprende il lavoro Code lasciato dal turno precedente.
- **Una persona, più account.** Se sullo stesso computer passi da un account personale a uno di lavoro, o da un'organizzazione all'altra, ritrovi le tue sessioni Code con qualunque account tu acceda.

Tutte le sessioni Code del computer diventano visibili a ogni account che vi accede. Non usare CCDC dove le persone non devono vedere le sessioni degli altri. Se usi un account aziendale (organizzazione), verifica prima i criteri della tua azienda: CCDC copia le sessioni di un account di lavoro anche nella cartella di un account personale, e viceversa.

## Il problema che risolve

L'app desktop salva l'**elenco** delle sessioni Code in una cartella separata per ogni account e organizzazione:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Cartella dati di Claude | Registro di Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (oppure `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Le organizzazioni che usano Claude tramite il proprio provider cloud hanno `Claude-3p` al posto di `Claude`; CCDC cerca anche lì.)

Le **conversazioni** vere e proprie sono in `~/.claude/projects/.../<cliSessionId>.jsonl`, un percorso condiviso da tutti gli account. Perciò, dopo l'accesso con un altro account, l'elenco delle sessioni Code appare vuoto anche se le conversazioni sono ancora sul disco.

## Come funziona

L'app legge la sua cartella delle sessioni **solo all'avvio e all'accesso** (due volte, entro circa due secondi). Non c'è modo di farle rileggere solo l'elenco in un secondo momento. Il registro dell'app stessa riporta entrambi gli eventi:

- disconnessione: `[account] Login-state transition (loggedOut: false → true ...)`
- accesso: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC segue quel registro:

1. **Disconnessione** → copia la versione più recente di ogni sessione in tutte le cartelle di account/organizzazione, così il prossimo account che accede le trova già nella propria cartella. Vengono copiate solo le sessioni mancanti o non aggiornate in una cartella.
2. **Accesso** → per impostazione predefinita le copie restano dove sono. Se è attivata l'opzione "Elimina le copie dopo l'accesso", CCDC aspetta che l'app abbia letto l'elenco (6 secondi dopo l'ultima riga di accesso) e poi rimuove le copie ovunque tranne che nella cartella dell'account connesso.

Una sessione eliminata nell'app viene rimossa anche dalle altre cartelle, così non ricompare con un altro account.

Prima di copiare, CCDC controlla lo spazio libero su disco. Se non c'è spazio per le copie più 512 MB, non copia nulla e mostra un avviso.

### Perché le copie si conservano per impostazione predefinita

Conservare le copie è l'opzione che usura meno l'SSD:

- **Copie conservate (predefinito)**: la prima disconnessione copia tutte le sessioni una volta. Poi ogni disconnessione scrive solo le sessioni cambiate dall'ultima volta, di solito pochi file e pochi MB, e finisce quasi subito.
- **Copie eliminate dopo l'accesso**: ogni accesso elimina le copie, quindi ogni disconnessione deve riscriverle tutte. Con circa 300 sessioni e 10 cartelle di account/organizzazione sono circa 600 MB scritti a ogni cambio turno, più l'eliminazione successiva di circa 3.000 file.

Il prezzo da pagare è lo spazio su disco: le copie restano al loro posto e occupano circa (dimensione totale dei file di sessione × numero di cartelle di account/organizzazione).

### Perché non usare giunzioni o collegamenti reali

Far puntare tutte le cartelle degli account a un'unica cartella reale eviterebbe le copie, ma l'app Claude blocca di proposito entrambi i modi di farlo:

- **Giunzioni e collegamenti simbolici**: prima di ogni salvataggio l'app verifica che la cartella delle sessioni sia una cartella vera e si rifiuta di salvare se è un collegamento (`Refusing non-directory at private dir path (symlink/file plant)`). In un test con l'app reale, l'elenco si leggeva attraverso una giunzione, ma tutti i salvataggi di sessioni e attività pianificate sono falliti, compresi quelli delle sessioni appena create.
- **Collegamenti reali**: l'app considera non sicuro un file con più di un collegamento. Si rifiuta di scriverci e avvisa quando lo legge. Inoltre salva scrivendo un file temporaneo e rinominandolo, cosa che romperebbe il collegamento a ogni salvataggio.

Questi controlli proteggono da file o cartelle inseriti dall'esterno, quindi CCDC non li aggira. Copia file normali, dello stesso tipo di quelli che scrive l'app stessa.

## Installazione

Scarica il file per il tuo sistema da [Releases](../../releases). I file non hanno una firma digitale (vedi [più avanti](#informazioni-sullavviso-di-sicurezza)).

**Windows**: esegui `CCDC-<version>-windows-x64.msi`. Non servono diritti di amministratore.

- Si installa in `%LOCALAPPDATA%\Programs\CCDC`, aggiunge un collegamento `Continuous Claude Desktop Code` sul desktop e nel menu Start e avvia CCDC nell'area di notifica.
- Si avvia insieme al computer; puoi disattivarlo con "Avvia all'accensione del computer" nel menu dell'icona.
- Aggiornamento: esegui il `.msi` più recente. Chiude CCDC, lo sostituisce e lo riavvia; le tue impostazioni vengono conservate.
- Per disinstallarlo, vai in Impostazioni > App > App installate > "Continuous Claude Desktop Code (CCDC)".
- Se usavi gli script 0.3, installa semplicemente il `.msi`. Al primo avvio CCDC arresta la vecchia versione, ne conserva le impostazioni e i nomi degli account e la rimuove.

**macOS**: apri `CCDC-<version>-macos-universal.dmg`, trascina **CCDC** in Applicazioni e poi aprilo. Nella barra dei menu compare una **C**. Si avvia insieme al computer, a meno che tu non lo disattivi dal suo menu. Funziona sia su Apple Silicon sia su Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Poi avvia **Continuous Claude Desktop Code** dal menu delle applicazioni. Si avvia insieme al computer, a meno che tu non lo disattivi dal suo menu. Su GNOME senza l'estensione AppIndicator l'icona nell'area di notifica non può comparire (Ubuntu e Linux Mint la includono).

La disinstallazione lascia al loro posto i file di sessione di Claude, le impostazioni di CCDC e la cartella dei backup.

### Informazioni sull'avviso di sicurezza

Per firmare i file serve un certificato a pagamento intestato a una persona o a un'azienda e CCDC è un piccolo progetto gratuito, quindi i file scaricati non sono firmati. La prima volta il sistema mostra un avviso: significa solo che l'editore non è verificato, non che sia stato trovato qualcosa di dannoso.

CCDC non contiene nulla di dannoso. Non si connette a Internet, non raccoglie dati e non modifica nulla al di fuori delle cartelle delle sessioni dell'app Claude e della propria cartella. Tutto il codice è in questo repository e i file delle release vengono compilati da qui con GitHub Actions, i cui log sono pubblici. Puoi verificare un file scaricato con `SHA256SUMS.txt` nella release, oppure [compilarlo tu stesso](#compilazione).

- **Windows**: se SmartScreen mostra "Windows ha protetto il PC", fai clic su **Ulteriori informazioni** e poi su **Esegui comunque**.
- **macOS**: se macOS dice che non può verificare l'app, apri **Impostazioni di Sistema > Privacy e sicurezza**, scorri verso il basso e fai clic su **Apri comunque** accanto a CCDC. (In alternativa, esegui `xattr -dr com.apple.quarantine /Applications/CCDC.app` nel Terminale.)
- **Linux**: i pacchetti `.deb` non vengono controllati in questo modo.

## App nell'area di notifica

Nell'area di notifica o nella barra dei menu compare una **C** arancione (grigia quando la sincronizzazione automatica è disattivata). Fai clic sull'icona per aprire il menu:

| Voce di menu | Cosa fa |
| --- | --- |
| Riga di stato | Indica se CCDC sta controllando accessi e disconnessioni |
| Righe dell'ora di accesso | Una riga per ogni account disconnesso con quanto tempo fa ha effettuato l'accesso (es. "accesso 2 h 13 min fa"), per stimare a che punto è nella finestra di utilizzo di 5 ore di Claude |
| Sincronizzazione automatica (rileva accesso/disconnessione) | Attiva o disattiva la copia e la pulizia |
| Sposta ora le sessioni in un account | Compare quando "Elimina le copie dopo l'accesso" è attivo. Per una disconnessione non rilevata: raccoglie tutte le sessioni nella cartella di un account; chiede conferma se quell'account non è connesso |
| Sincronizza ora tutti gli account | Compare nello stesso punto quando "Elimina le copie dopo l'accesso" è disattivato. Per una disconnessione non rilevata: copia tutte le sessioni nella cartella di ogni account, come alla disconnessione |
| Elimina le copie dopo l'accesso | Disattivata per impostazione predefinita (le copie si conservano; vedi sopra) |
| Velocità di copia/eliminazione | Veloce (nessun limite), Normale (60 MB/s) o Lenta (15 MB/s). Con Normale e Lenta l'eliminazione è limitata rispettivamente a 500 e 100 file al secondo |
| Riga di stima | Indica quanto richiederebbe la prossima copia alla disconnessione e, se le copie vengono eliminate, anche la pulizia dopo l'accesso |
| Avvia all'accensione del computer | Avvia CCDC quando accedi al computer |
| Language | Sceglie la lingua di visualizzazione. Questa voce è sempre in inglese; ogni lingua nel sottomenu è scritta nella propria lingua |
| Visualizza registro | Apre il file `watch.log` di CCDC |
| Assegna nomi agli account (ccdc.json) | Apre il file delle impostazioni per dare agli account nomi leggibili |
| Esci | Rimuove l'icona; se è in corso una copia, aspetta che finisca |

Vengono visualizzate notifiche quando inizia la copia (con il tempo stimato), quando termina, quando viene rilevato un accesso e quando la pulizia termina. Con una velocità bassa la copia richiede più tempo: la persona successiva deve aspettare la notifica "Copia completata", o all'incirca il tempo stimato, prima di accedere.

La stima si calcola in base allo stato attuale delle cartelle e alle velocità di copia ed eliminazione misurate su questo computer (salvate in `state.json` nella cartella di CCDC).

Se l'icona non si vede in Windows 11, cercala tra le icone nascoste (`^`) sulla barra delle applicazioni. Per tenerla sempre visibile, attiva CCDC in Impostazioni > Personalizzazione > Barra delle applicazioni > Altre icone della barra delle applicazioni.

CCDC è un unico programma nativo di circa 1 MB. Mentre è in attesa, ogni mezzo secondo legge solo ciò che è stato aggiunto al registro di Claude dall'ultimo controllo e usa per sé circa 4 MB di memoria.

## Impostazioni

Le impostazioni sono in `ccdc.json`, nella cartella di CCDC:

| | Cartella di CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Le modifiche fatte dal menu dell'icona vengono salvate in questo file e valgono dalla copia o pulizia successiva, senza riavvio. `watch.log` e `state.json` si trovano nella stessa cartella.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Chiave | Valori |
| --- | --- |
| `keepCopies` | `true` (predefinito): le copie vengono conservate. `false`: vengono eliminate dopo l'accesso |
| `speed` | `"fast"` (predefinito), `"normal"`, `"gentle"` |
| `language` | Vuoto per seguire la lingua del sistema, oppure un codice lingua come `"ja"` per usare quella lingua |
| `startAtLogin` | Viene impostato da "Avvia all'accensione del computer" nel menu dell'icona |
| `aliases` | Nomi mostrati per gli account nel menu dell'icona |
| `dataDir`, `appLog` | Solo se CCDC non trova da solo la cartella dati o il registro di Claude: i rispettivi percorsi completi |

## Lingue

Il menu, le notifiche e gli avvisi seguono la lingua di visualizzazione del sistema, oppure la lingua scelta in **Language** nel menu dell'icona. Lingue supportate: inglese, coreano, giapponese, cinese (semplificato e tradizionale), spagnolo, francese, tedesco, italiano, portoghese, russo, ucraino, polacco, olandese, turco, vietnamita, indonesiano, thailandese, arabo e hindi. Le altre lingue vengono mostrate in inglese.

Tutti i testi dell'interfaccia sono in un unico foglio, [`locales.csv`](locales.csv), incorporato nel programma: **una riga per lingua**, una colonna per messaggio. Aprilo in Excel o Fogli Google; per aggiungere una lingua, aggiungi una riga e [compila](#compilazione) CCDC.

- `lang`: un tag di lingua (`sv`, `pt-BR`, `zh-Hant`, …). Si preferisce la riga che corrisponde a lingua e regione, poi quella che corrisponde alla lingua.
- `name`: il nome della lingua in quella lingua (`Svenska`); lo mostra il menu Language.
- Scrivi gli a capo all'interno di una cella come `\n` e mantieni `{0}`, `{1}` (lì vanno numeri o nomi).
- Le celle vuote vengono mostrate in inglese. Salva il file come CSV UTF-8.

L'output della riga di comando e `watch.log` restano in inglese.

## Riga di comando

Lo stesso programma funziona anche da terminale (su Windows è `ccdc.exe` in `%LOCALAPPDATA%\Programs\CCDC`; su macOS è `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, last sign-in times, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

Con `--data-dir <path>` e `--app-log <path>` puoi indicargli a mano una cartella dati o un registro di Claude.

## Regole per i duplicati

- Se una sessione è in più cartelle, vince la copia con il `lastActivityAt` più recente. Le copie mantengono la data di modifica dell'originale, quindi le copie identiche si riconoscono da dimensione e data senza leggerle.
- Quando una copia più recente sostituisce un file, il vecchio file viene spostato in `ccdc-backups/<time>/` nella cartella dati di Claude.
- Le sessioni con un indicatore `deleted_<id>` non vengono copiate e le loro copie nelle altre cartelle vengono rimosse.
- Una coppia account/organizzazione che accede per la primissima volta non ha ancora una cartella, quindi alla disconnessione non è stato possibile copiarvi nulla. In questo caso le sessioni compaiono dopo il successivo accesso di quell'account o dopo un riavvio dell'app.

## Avvertenze

- CCDC dipende dalla struttura interna dei file e dal testo del registro dell'app desktop di Claude; un aggiornamento dell'app può modificare il comportamento di CCDC.
- I file di sessione contengono impostazioni dei connettori (MCP) di ciascun account, quindi alcuni connettori potrebbero non connettersi quando una sessione viene aperta da un altro account.
- Le attività pianificate non sono condivise; ogni cartella di account/organizzazione mantiene le proprie.
- CCDC non gestisce il cambio di account nella CLI di Claude Code. La CLI può già aprire qualsiasi sessione in `~/.claude/projects` con `--resume`, indipendentemente dall'account.

## Compilazione

CCDC è scritto in Rust; [installa Rust](https://rustup.rs) e poi esegui:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

La versione di ogni piattaforma è indicata in `[package.metadata.ccdc]` nel file `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) compila tutte e tre le versioni a ogni push e, per un tag `v*`, le allega a una bozza di release.

## Licenza

[MIT](LICENSE)
