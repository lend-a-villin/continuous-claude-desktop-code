# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | **Français** | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Une application de la zone de notification qui conserve vos sessions **Code** de l'application Claude pour ordinateur lorsque vous changez de compte Claude.

L'application Claude pour ordinateur tient une liste de sessions Code distincte pour chaque compte et chaque organisation. Connectez-vous avec un autre compte et la liste paraît vide, alors que les conversations sont toujours sur l'ordinateur. CCDC recopie les sessions d'un dossier à l'autre : quel que soit le compte qui se connecte ensuite, il les voit toutes.

Outil non officiel, non affilié à Anthropic et non approuvé par Anthropic.

**[Télécharger la dernière version](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · rien d'autre à installer

| Plateforme | Version | État |
| --- | --- | --- |
| Windows | 1.0 | Utilisée au quotidien (prend la suite des scripts 0.3) |
| macOS | 0.1 | Nouvelle ; pas encore testée avec la vraie application Claude sur un Mac |
| Linux | 0.1 | Nouvelle ; installation testée sous Debian 12, Ubuntu 24.04 et Linux Mint 22, mais pas encore avec la vraie application Claude |

## À qui s'adresse CCDC

- **Ordinateurs partagés et travail en équipes.** Dans un lieu de travail qui fonctionne 24 h/24 en trois équipes, une personne différente s'installe au même poste (un ordinateur, un utilisateur système) à chaque relève et se connecte avec son propre compte Claude. Avec CCDC, l'équipe suivante reprend le travail Code là où l'équipe précédente l'a laissé.
- **Une personne, plusieurs comptes.** Si vous passez d'un compte personnel à un compte professionnel, ou d'une organisation à une autre, sur le même ordinateur, vos sessions Code sont là quel que soit le compte avec lequel vous vous connectez.

Toutes les sessions Code de l'ordinateur deviennent visibles par tous les comptes qui s'y connectent. N'utilisez pas CCDC là où les utilisateurs ne doivent pas voir les sessions les uns des autres. Si vous utilisez un compte d'entreprise (organisation), vérifiez d'abord la politique de votre entreprise : CCDC copie aussi les sessions d'un compte professionnel dans le dossier d'un compte personnel, et inversement.

## Le problème résolu

L'application enregistre la **liste** des sessions Code dans un dossier distinct par compte et par organisation :

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Dossier de données de Claude | Journal de Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (ou `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Les organisations qui utilisent Claude via leur propre fournisseur cloud ont un dossier `Claude-3p` au lieu de `Claude` ; CCDC le cherche aussi.)

Les **conversations** elles-mêmes se trouvent dans `~/.claude/projects/.../<cliSessionId>.jsonl`, un emplacement commun à tous les comptes. Après une connexion avec un autre compte, la liste des sessions Code paraît donc vide alors que les conversations sont toujours sur le disque.

## Fonctionnement

L'application lit son dossier de sessions **uniquement au démarrage et à la connexion** (deux fois, en l'espace d'environ deux secondes). Il n'existe aucun moyen de lui faire relire seulement la liste plus tard. Le journal de l'application enregistre les deux événements :

- déconnexion : `[account] Login-state transition (loggedOut: false → true ...)`
- connexion : `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC suit ce journal :

1. **Déconnexion** → il copie la version la plus récente de chaque session dans tous les dossiers de compte/organisation, si bien que le prochain compte à se connecter les trouve déjà dans son propre dossier. Seules les sessions absentes ou obsolètes dans un dossier sont copiées.
2. **Connexion** → par défaut, les copies restent en place. Si l'option « Supprimer les copies après la connexion » est activée, CCDC attend que l'application ait lu sa liste (6 secondes après la dernière ligne de connexion), puis supprime les copies partout sauf dans le dossier du compte connecté.

Une session supprimée dans l'application est aussi supprimée des autres dossiers, pour qu'elle ne réapparaisse pas sous un autre compte.

Avant de copier, CCDC vérifie l'espace disque libre. S'il n'y a pas assez de place pour les copies plus 512 Mo, il ne copie rien et affiche un avertissement.

### Pourquoi les copies sont conservées par défaut

Conserver les copies est l'option qui ménage le plus un SSD :

- **Copies conservées (par défaut)** : la première déconnexion copie toutes les sessions une fois. Ensuite, chaque déconnexion n'écrit que les sessions modifiées depuis la précédente, généralement quelques fichiers et quelques Mo, et se termine presque aussitôt.
- **Copies supprimées après la connexion** : chaque connexion supprime les copies, donc chaque déconnexion doit toutes les réécrire. Avec environ 300 sessions et 10 dossiers de compte/organisation, cela représente environ 600 Mo écrits à chaque changement d'équipe, plus la suppression d'environ 3 000 fichiers ensuite.

La contrepartie est l'espace disque : les copies restent en place et occupent environ (taille totale des fichiers de session × nombre de dossiers de compte/organisation).

### Pourquoi pas de jonctions ni de liens physiques

Faire pointer tous les dossiers de compte vers un seul dossier réel éviterait les copies, mais l'application Claude bloque volontairement les deux façons de le faire :

- **Jonctions et liens symboliques** : avant chaque enregistrement, l'application vérifie que le dossier de sessions est un vrai dossier et refuse d'enregistrer s'il s'agit d'un lien (`Refusing non-directory at private dir path (symlink/file plant)`). Lors d'un test avec l'application réelle, la liste pouvait être lue via une jonction, mais tous les enregistrements de sessions et de tâches planifiées ont échoué, y compris pour de toutes nouvelles sessions.
- **Liens physiques** : l'application considère comme dangereux un fichier qui a plus d'un lien. Elle refuse d'y écrire et émet un avertissement lorsqu'elle le lit. Elle enregistre en outre en écrivant un fichier temporaire puis en le renommant, ce qui romprait le lien à chaque enregistrement.

Ces vérifications protègent contre des fichiers ou dossiers déposés depuis l'extérieur ; CCDC ne les contourne donc pas. Il copie des fichiers ordinaires, du même type que ceux qu'écrit l'application elle-même.

## Installation

Téléchargez le fichier correspondant à votre système depuis [Releases](../../releases). Les fichiers ne sont pas signés numériquement (voir [plus bas](#à-propos-de-lavertissement-de-sécurité)).

**Windows** — lancez `CCDC-<version>-windows-x64.msi`. Aucun droit d'administrateur n'est nécessaire.

- CCDC s'installe dans `%LOCALAPPDATA%\Programs\CCDC`, ajoute un raccourci `Continuous Claude Desktop Code` sur le Bureau et dans le menu Démarrer, puis démarre dans la zone de notification.
- Il se lance au démarrage de l'ordinateur ; pour l'en empêcher, désactivez « Lancer au démarrage de l'ordinateur » dans le menu de la zone de notification.
- Mise à jour : lancez le `.msi` plus récent. Il ferme CCDC, le remplace et le relance ; vos réglages sont conservés.
- Pour le désinstaller, ouvrez Paramètres > Applications > Applications installées > « Continuous Claude Desktop Code (CCDC) ».
- Si vous utilisiez les scripts 0.3 : installez simplement le `.msi`. À son premier démarrage, CCDC arrête l'ancienne version, reprend ses réglages et les noms de comptes, puis la supprime.

**macOS** — ouvrez `CCDC-<version>-macos-universal.dmg`, faites glisser **CCDC** dans Applications, puis ouvrez-le. Un **C** apparaît dans la barre des menus. CCDC se lance au démarrage de l'ordinateur, sauf si vous le désactivez dans son menu. Il fonctionne sur Apple Silicon comme sur Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64) :

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Lancez ensuite **Continuous Claude Desktop Code** depuis le menu des applications. CCDC se lance au démarrage de l'ordinateur, sauf si vous le désactivez dans son menu. Sous GNOME, l'icône ne peut pas s'afficher sans l'extension AppIndicator (Ubuntu et Linux Mint l'incluent).

La désinstallation laisse en place les fichiers de session Claude, les réglages de CCDC et le dossier de sauvegarde.

### À propos de l'avertissement de sécurité

Pour signer un logiciel, il faut un certificat payant établi au nom d'une personne ou d'une entreprise ; CCDC étant un petit projet gratuit, les téléchargements ne sont pas signés. Votre système affiche donc un avertissement la première fois. Il signifie seulement que l'éditeur n'a pas été vérifié, pas que quelque chose de dangereux a été détecté.

CCDC ne contient rien de malveillant. Il ne se connecte pas à Internet, ne collecte aucune donnée et ne modifie rien en dehors des dossiers de sessions de l'application Claude et de son propre dossier. Tout le code se trouve dans ce dépôt, et les fichiers publiés sont compilés à partir de celui-ci par GitHub Actions, dont les journaux sont publics. Vous pouvez vérifier un téléchargement à l'aide du fichier `SHA256SUMS.txt` joint à la version, ou [le compiler vous-même](#compilation).

- **Windows** : si SmartScreen affiche « Windows a protégé votre ordinateur », cliquez sur **Informations complémentaires**, puis sur **Exécuter quand même**.
- **macOS** : si macOS indique qu'il ne peut pas vérifier l'app, ouvrez **Réglages Système > Confidentialité et sécurité**, faites défiler vers le bas et cliquez sur **Ouvrir quand même** à côté de CCDC. (Vous pouvez aussi exécuter `xattr -dr com.apple.quarantine /Applications/CCDC.app` dans le Terminal.)
- **Linux** : les paquets `.deb` ne font pas l'objet de cette vérification.

## Application de la zone de notification

Un **C** orange apparaît dans la zone de notification ou la barre des menus (gris quand la synchronisation automatique est désactivée). Cliquez dessus pour ouvrir le menu :

| Élément du menu | Rôle |
| --- | --- |
| Ligne d'état | Indique si CCDC surveille les connexions et les déconnexions |
| Synchronisation automatique (détecter connexion/déconnexion) | Active ou désactive la copie et le nettoyage |
| Déplacer les sessions vers un compte maintenant | Regroupe toutes les sessions dans le dossier d'un compte ; demande confirmation si ce compte n'est pas connecté |
| Supprimer les copies après la connexion | Désactivé par défaut (les copies sont conservées ; voir plus haut) |
| Vitesse de copie/suppression | Rapide (sans limite), Normale (60 Mo/s) ou Douce (15 Mo/s). La suppression est limitée à 500 fichiers par seconde en Normale et à 100 en Douce |
| Ligne d'estimation | Indique la durée de la prochaine copie à la déconnexion et, si les copies sont supprimées, celle du nettoyage après la connexion |
| Lancer au démarrage de l'ordinateur | Lance CCDC lorsque vous ouvrez votre session sur l'ordinateur |
| Language | Choisit la langue d'affichage. Cet élément est toujours affiché en anglais ; chaque langue du sous-menu est écrite dans cette langue |
| Afficher le journal | Ouvre le fichier `watch.log` de CCDC |
| Nommer les comptes (ccdc.json) | Ouvre le fichier de réglages pour donner aux comptes des noms lisibles |
| Quitter | Retire l'icône ; si une copie est en cours, attend d'abord qu'elle se termine |

Des notifications s'affichent quand la copie commence (avec la durée estimée), quand elle est terminée, quand une connexion est détectée et quand le nettoyage est terminé. Plus la vitesse est lente, plus la copie dure : la personne suivante doit donc attendre la notification « Copie terminée », ou laisser passer à peu près la durée estimée, avant de se connecter.

L'estimation est calculée d'après l'état actuel des dossiers et les vitesses de copie et de suppression mesurées sur cet ordinateur (enregistrées dans `state.json`, dans le dossier de CCDC).

Si l'icône n'apparaît pas sous Windows 11, regardez parmi les icônes cachées (`^`) de la barre des tâches. Pour qu'elle reste visible, activez CCDC dans Paramètres > Personnalisation > Barre des tâches > Autres icônes de la barre d'état système.

CCDC est un programme natif unique d'environ 1 Mo. En attente, il lit toutes les demi-secondes uniquement ce qui a été ajouté au journal de Claude depuis la dernière vérification, et n'utilise qu'environ 4 Mo de mémoire.

## Réglages

Les réglages sont enregistrés dans `ccdc.json`, dans le dossier de CCDC :

| | Dossier de CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Les modifications faites dans le menu de la zone de notification sont enregistrées dans ce fichier et s'appliquent dès la prochaine copie ou le prochain nettoyage, sans redémarrage. `watch.log` et `state.json` se trouvent dans le même dossier.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Clé | Valeurs |
| --- | --- |
| `keepCopies` | `true` (par défaut) : les copies sont conservées. `false` : elles sont supprimées après la connexion |
| `speed` | `"fast"` (par défaut), `"normal"`, `"gentle"` |
| `language` | Vide pour suivre la langue du système, ou un code de langue comme `"ja"` pour utiliser cette langue |
| `startAtLogin` | Modifié par « Lancer au démarrage de l'ordinateur » dans le menu de la zone de notification |
| `aliases` | Noms affichés pour les comptes dans le menu de la zone de notification |
| `dataDir`, `appLog` | Uniquement si CCDC ne trouve pas seul le dossier de données ou le journal de Claude : leurs chemins complets |

## Langues

Le menu de la zone de notification, les notifications et les avertissements suivent la langue d'affichage du système, ou la langue choisie sous **Language** dans le menu de la zone de notification. Langues prises en charge : anglais, coréen, japonais, chinois (simplifié et traditionnel), espagnol, français, allemand, italien, portugais, russe, ukrainien, polonais, néerlandais, turc, vietnamien, indonésien, thaï, arabe et hindi. Les autres langues s'affichent en anglais.

Tous les textes de l'interface se trouvent dans une seule feuille, [`locales.csv`](locales.csv), intégrée au programme : **une ligne par langue**, une colonne par message. Ouvrez-la dans Excel ou Google Sheets ; pour ajouter une langue, ajoutez une ligne et [compilez](#compilation) CCDC.

- `lang` : une étiquette de langue (`sv`, `pt-BR`, `zh-Hant`, …). La ligne qui correspond à la langue et à la région est prioritaire, puis celle qui correspond à la langue seule.
- `name` : le nom de la langue dans cette langue (`Svenska`) ; le menu Language l'affiche.
- Écrivez les retours à la ligne dans une cellule sous la forme `\n`, et conservez `{0}`, `{1}` (des nombres ou des noms y sont insérés).
- Les cellules vides s'affichent en anglais. Enregistrez le fichier au format CSV UTF-8.

La sortie de la ligne de commande et `watch.log` restent en anglais.

## Ligne de commande

Le même programme s'utilise aussi depuis un terminal (sous Windows, `ccdc.exe` dans `%LOCALAPPDATA%\Programs\CCDC` ; sous macOS, `/Applications/CCDC.app/Contents/MacOS/ccdc`) :

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` et `--app-log <path>` lui indiquent manuellement un dossier de données ou un journal de Claude.

## Règles pour les doublons

- Quand une session se trouve dans plusieurs dossiers, la copie dont le `lastActivityAt` est le plus récent l'emporte. Les copies gardent l'heure de modification de la source ; des copies identiques sont donc reconnues d'après la taille et l'heure, sans être lues.
- Quand une copie plus récente remplace un fichier, l'ancien fichier est déplacé vers `ccdc-backups/<time>/` dans le dossier de données de Claude.
- Les sessions portant un marqueur `deleted_<id>` ne sont pas copiées, et leurs copies dans les autres dossiers sont supprimées.
- Un compte ou une organisation qui se connecte pour la toute première fois n'a pas encore de dossier : rien n'a donc pu y être copié à la déconnexion. Ses sessions s'affichent alors après sa prochaine connexion ou un redémarrage de l'application.

## Mises en garde

- CCDC dépend de l'organisation interne des fichiers et du texte du journal de l'application Claude pour ordinateur ; une mise à jour de l'application peut modifier son comportement.
- Les fichiers de session contiennent des réglages de connecteurs (MCP) propres à chaque compte : certains connecteurs peuvent ne pas se connecter quand une session est ouverte sous un autre compte.
- Les tâches planifiées ne sont pas partagées ; chaque dossier de compte/organisation garde les siennes.
- Le changement de compte dans la CLI de Claude Code n'est pas pris en charge. La CLI peut déjà ouvrir n'importe quelle session de `~/.claude/projects` avec `--resume`, quel que soit le compte.

## Compilation

CCDC est écrit en Rust ; [installez Rust](https://rustup.rs), puis :

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

La version de chaque plateforme est définie dans `[package.metadata.ccdc]`, dans `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) compile les trois à chaque push et, pour un tag `v*`, les joint à une version brouillon.

## Licence

[MIT](LICENSE)
