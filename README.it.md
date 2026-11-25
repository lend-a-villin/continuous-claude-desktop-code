# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | **Italiano** | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Un'app per l'area di notifica di Windows che rende disponibili le sessioni **Code** locali dell'app desktop di Claude a qualunque account Claude con cui si acceda sul PC.

Strumento non ufficiale; non affiliato ad Anthropic né approvato da Anthropic.

## A chi è rivolto

CCDC è pensato per i **luoghi di lavoro attivi 24 ore su 24 con tre turni**. A ogni turno una persona diversa si siede alla stessa postazione (un PC, un utente Windows) e accede all'app desktop di Claude con il proprio account Claude. Con CCDC, chi arriva al turno successivo accede e riprende il lavoro Code lasciato dal turno precedente.

Di conseguenza, tutte le sessioni Code del PC sono visibili a chiunque acceda. Non usare CCDC dove le persone non devono vedere le sessioni degli altri. Se usi un account aziendale (organizzazione), verifica anche i criteri della tua azienda.

## Il problema che risolve

L'app desktop salva l'**elenco** delle sessioni Code in una cartella separata per ogni account e organizzazione:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

Le **conversazioni** vere e proprie sono in `~/.claude/projects\...\<cliSessionId>.jsonl`, un percorso condiviso da tutti gli account. Perciò, dopo l'accesso con un altro account, l'elenco delle sessioni Code appare vuoto anche se le conversazioni sono ancora sul disco.

## Come funziona

L'app legge la sua cartella delle sessioni **solo all'avvio e all'accesso** (due volte, entro circa due secondi). Non c'è modo di farle rileggere solo l'elenco in un secondo momento. Il registro dell'app stessa, `%LOCALAPPDATA%\Claude\Logs\main.log`, riporta sia le disconnessioni sia gli accessi:

- disconnessione: `[account] Login-state transition (loggedOut: false → true ...)`
- accesso: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC segue quel registro e agisce così:

1. **Disconnessione** → copia la versione più recente di ogni sessione in tutte le cartelle di account/organizzazione, così il prossimo account che accede le trova già nella propria cartella. Vengono copiate solo le sessioni mancanti o non aggiornate in una cartella.
2. **Accesso** → per impostazione predefinita le copie restano dove sono. Se è attivata l'opzione "Elimina le copie dopo l'accesso", CCDC aspetta che l'app abbia letto l'elenco (6 secondi dopo l'ultima riga di accesso) e poi rimuove le copie ovunque tranne che nella cartella dell'account connesso.

Una sessione eliminata nell'app viene rimossa anche dalle altre cartelle, così non ricompare con un altro account.

Prima di copiare, CCDC controlla lo spazio libero su disco. Se non c'è spazio per le copie più 512 MB, non copia nulla e mostra un avviso.

### Perché le copie si conservano per impostazione predefinita

Conservare le copie è l'opzione che usura meno l'SSD:

- **Copie conservate (predefinito)**: la prima disconnessione copia tutte le sessioni una volta. Poi ogni disconnessione scrive solo le sessioni cambiate dall'ultima volta, di solito pochi file e pochi MB, e finisce quasi subito.
- **Copie eliminate dopo l'accesso**: ogni accesso elimina le copie, quindi ogni disconnessione deve riscriverle tutte. Con circa 300 sessioni e 10 cartelle di account/organizzazione sono circa 600 MB scritti a ogni cambio turno, più l'eliminazione successiva di circa 3.000 file.

Il prezzo da pagare è lo spazio su disco: le copie restano sul disco e occupano circa (dimensione totale dei file di sessione × numero di cartelle di account/organizzazione).

### Perché non usare giunzioni o collegamenti reali

Far puntare tutte le cartelle degli account a un'unica cartella reale eviterebbe le copie, ma l'app Claude blocca di proposito entrambi i modi di farlo:

- **Giunzioni e collegamenti simbolici**: prima di ogni salvataggio l'app verifica che la cartella delle sessioni sia una cartella vera e si rifiuta di salvare se è un collegamento (`Refusing non-directory at private dir path (symlink/file plant)`). In un test con l'app reale, l'elenco si leggeva attraverso una giunzione, ma tutti i salvataggi di sessioni e attività pianificate sono falliti, compresi quelli delle sessioni appena create.
- **Collegamenti reali**: l'app considera non sicuro un file con più di un collegamento. Si rifiuta di scriverci e avvisa quando lo legge. Inoltre salva scrivendo un file temporaneo e rinominandolo, cosa che romperebbe il collegamento a ogni salvataggio.

Questi controlli proteggono da file o cartelle inseriti dall'esterno, quindi CCDC non li aggira. Copia file normali, dello stesso tipo di quelli che scrive l'app stessa.

## Requisiti

- Windows 10 o 11, con l'app desktop di Claude (installazione MSIX)
- [Node.js](https://nodejs.org) 18 o successivo

## Installazione

Scarica il programma di installazione da [Releases](../../releases), estrailo ed esegui `setup.cmd`. Non servono diritti di amministratore.

- Si installa in `%LOCALAPPDATA%\Programs\CCDC`.
- Aggiunge un collegamento `Continuous Claude Desktop Code` sul desktop e nel menu Start.
- Compare in Impostazioni > App > App installate come "Continuous Claude Desktop Code (CCDC)"; disinstallalo da lì.
- Si avvia nell'area di notifica subito dopo l'installazione.
- Per avviarlo all'accesso a Windows, installa con `setup.cmd /startup`; per disattivare l'avvio automatico, usa `setup.cmd /nostartup`. Senza nessuna delle due opzioni, una reinstallazione mantiene l'impostazione precedente.
- Una reinstallazione arresta CCDC, se è in esecuzione, e ne sostituisce i file; le tue impostazioni (`ccdc.json`) vengono conservate.
- La disinstallazione lascia al loro posto i file di sessione di Claude e la cartella dei backup di CCDC.

Per usarlo direttamente da questo repository, esegui `ccdc-tray.vbs` oppure usa la riga di comando qui sotto.

## App nell'area di notifica

Nell'area di notifica compare una **C** arancione (grigia quando la sincronizzazione automatica è disattivata). Fai clic sull'icona per aprire il menu:

| Voce di menu | Cosa fa |
| --- | --- |
| Riga di stato | Indica se CCDC sta controllando accessi e disconnessioni |
| Sincronizzazione automatica (rileva accesso/disconnessione) | Attiva o disattiva la copia e la pulizia |
| Sposta ora le sessioni in un account | Raccoglie tutte le sessioni nella cartella di un account; chiede conferma se quell'account non è connesso |
| Elimina le copie dopo l'accesso | Disattivata per impostazione predefinita (le copie si conservano; vedi sopra) |
| Velocità di copia/eliminazione | Veloce (nessun limite), Normale (60 MB/s) o Lenta (15 MB/s). Con Normale e Lenta l'eliminazione è limitata rispettivamente a 500 e 100 file al secondo |
| Riga di stima | Indica quanto richiederebbe la prossima copia alla disconnessione e, se le copie vengono eliminate, anche la pulizia dopo l'accesso |
| Language | Sceglie la lingua di visualizzazione. Questa voce è sempre in inglese; ogni lingua nel sottomenu è scritta nella propria lingua |
| Visualizza registro | Apre `%LOCALAPPDATA%\CCDC\watch.log` |
| Assegna nomi agli account (ccdc.json) | Apre il file delle impostazioni per dare agli account nomi leggibili |
| Esci | Rimuove l'icona; se è in corso una copia, aspetta che finisca |

Vengono visualizzate notifiche quando inizia la copia (con il tempo stimato), quando termina, quando viene rilevato un accesso e quando la pulizia termina. Con una velocità bassa la copia richiede più tempo: la persona successiva deve aspettare la notifica "Copia completata", o all'incirca il tempo stimato, prima di accedere.

La stima si calcola in base allo stato attuale delle cartelle e alle velocità di copia ed eliminazione misurate su questo PC (salvate in `%LOCALAPPDATA%\CCDC\state.json`).

Se l'icona non si vede in Windows 11, cercala tra le icone nascoste (`^`) sulla barra delle applicazioni. Per tenerla sempre visibile, attiva Windows PowerShell in Impostazioni > Personalizzazione > Barra delle applicazioni > Altre icone della barra delle applicazioni.

### Uso della memoria

L'app nell'area di notifica controlla da sola il registro di Claude, leggendo solo ciò che è stato aggiunto dall'ultimo controllo. Node.js è in esecuzione solo durante la copia o la pulizia e poi termina, quindi tra un cambio turno e l'altro resta attiva solo l'app nell'area di notifica. Quando è inattiva restituisce a Windows la memoria inutilizzata circa una volta al minuto (circa 15 MB in uso).

## Impostazioni

Le impostazioni sono in `ccdc.json`, nella cartella del programma. Le modifiche fatte dal menu nell'area di notifica vengono salvate qui e valgono dalla copia o pulizia successiva, senza riavvio.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Chiave | Valori |
| --- | --- |
| `keepCopies` | `true` (predefinito): le copie vengono conservate. `false`: vengono eliminate dopo l'accesso |
| `speed` | `"fast"` (predefinito), `"normal"`, `"gentle"` |
| `language` | Vuoto per seguire la lingua di Windows, oppure un codice lingua come `"ja"` per usare quella lingua |
| `aliases` | Nomi mostrati per gli account nel menu |

## Lingue

Il menu, le notifiche, gli avvisi e le finestre di dialogo di installazione e disinstallazione seguono la lingua di visualizzazione di Windows, oppure la lingua scelta in **Language** nel menu. Lingue supportate: inglese, coreano, giapponese, cinese (semplificato e tradizionale), spagnolo, francese, tedesco, italiano, portoghese, russo, ucraino, polacco, olandese, turco, vietnamita, indonesiano, thailandese, arabo e hindi. Le altre lingue vengono mostrate in inglese.

Tutti i testi dell'interfaccia sono in un unico foglio, [`locales.csv`](locales.csv): **una riga per lingua**, una colonna per messaggio. Aprilo in Excel o Fogli Google; per aggiungere una lingua, aggiungi una riga.

- `lang`: il nome delle impostazioni cultura di Windows (`sv`, `pt-BR`, `zh-Hant`, …). Si preferisce la riga che corrisponde a lingua e regione, poi quella che corrisponde alla lingua.
- `name`: il nome della lingua in quella lingua (`Svenska`); lo mostra il menu Language.
- Scrivi gli a capo all'interno di una cella come `\n` e mantieni `{0}`, `{1}` (lì vanno numeri o nomi).
- Le celle vuote vengono mostrate in inglese. Salva il file come CSV UTF-8.

L'output della riga di comando e `watch.log` restano in inglese, perché l'app nell'area di notifica li legge.

## Riga di comando

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Quando serve, l'app nell'area di notifica esegue questi comandi una tantum: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` e `node ccdc.mjs startup` (per allinearsi allo stato attuale dell'app Claude al proprio avvio o quando la sincronizzazione automatica viene riattivata).

## Regole per i duplicati

- Se una sessione è in più cartelle, vince la copia con il `lastActivityAt` più recente. Le copie mantengono la data di modifica dell'originale, quindi le copie identiche si riconoscono da dimensione e data senza leggerle.
- Quando una copia più recente sostituisce un file, il vecchio file viene spostato in `ccdc-backups\<ora>\` nella cartella dati dell'app.
- Le sessioni con un indicatore `deleted_<id>` non vengono copiate e le loro copie nelle altre cartelle vengono rimosse.
- Una coppia account/organizzazione che accede per la primissima volta non ha ancora una cartella, quindi alla disconnessione non è stato possibile copiarvi nulla. In questo caso le sessioni compaiono dopo il successivo accesso di quell'account o dopo un riavvio dell'app.

## Avvertenze

- CCDC dipende dalla struttura interna dei file e dal testo del registro dell'app desktop di Claude; un aggiornamento dell'app può modificare il comportamento di CCDC.
- I file di sessione contengono impostazioni dei connettori (MCP) di ciascun account, quindi alcuni connettori potrebbero non connettersi quando una sessione viene aperta da un altro account.
- Le attività pianificate non sono condivise; ogni cartella di account/organizzazione mantiene le proprie.
- CCDC non gestisce il cambio di account nella CLI di Claude Code. La CLI può già aprire qualsiasi sessione in `~/.claude/projects` con `--resume`, indipendentemente dall'account.

## Licenza

[MIT](LICENSE)
