# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | **Français** | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Application Windows résidant dans la zone de notification, qui rend les sessions **Code** locales de l'application Claude pour ordinateur accessibles à tout compte Claude qui se connecte sur le PC.

Outil non officiel, non affilié à Anthropic et non approuvé par Anthropic.

## À qui s'adresse CCDC

CCDC est conçu pour les **lieux de travail qui fonctionnent 24 h/24 en trois équipes**. À chaque relève, une personne différente s'installe au même poste (un PC, un utilisateur Windows) et se connecte à l'application Claude pour ordinateur avec son propre compte Claude. Avec CCDC, l'équipe suivante se connecte et reprend le travail Code là où l'équipe précédente l'a laissé.

Cela signifie que toutes les sessions Code du PC sont visibles par toute personne qui s'y connecte. N'utilisez pas CCDC là où les utilisateurs ne doivent pas voir les sessions les uns des autres. Avec un compte d'entreprise (organisation), vérifiez aussi la politique de votre entreprise.

## Le problème résolu

L'application enregistre la **liste** des sessions Code dans un dossier distinct par compte et par organisation :

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

Les **conversations** elles-mêmes se trouvent dans `~/.claude/projects\...\<cliSessionId>.jsonl`, un emplacement commun à tous les comptes. Après une connexion avec un autre compte, la liste des sessions Code paraît donc vide alors que les conversations sont toujours sur le disque.

## Fonctionnement

L'application lit son dossier de sessions **uniquement au démarrage et à la connexion** (deux fois, en l'espace d'environ deux secondes). Il n'existe aucun moyen de lui faire relire seulement la liste plus tard. Le journal de l'application, `%LOCALAPPDATA%\Claude\Logs\main.log`, enregistre les déconnexions comme les connexions :

- déconnexion : `[account] Login-state transition (loggedOut: false → true ...)`
- connexion : `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC suit ce journal et agit ainsi :

1. **Déconnexion** → il copie la version la plus récente de chaque session dans tous les dossiers de compte/organisation, si bien que le prochain compte à se connecter les trouve déjà dans son propre dossier. Seules les sessions absentes ou obsolètes dans un dossier sont copiées.
2. **Connexion** → par défaut, les copies restent en place. Si l'option « Supprimer les copies après la connexion » est activée, CCDC attend que l'application ait lu sa liste (6 secondes après la dernière ligne de connexion), puis supprime les copies partout sauf dans le dossier du compte connecté.

Une session supprimée dans l'application est aussi supprimée des autres dossiers, pour qu'elle ne réapparaisse pas sous un autre compte.

Avant de copier, CCDC vérifie l'espace disque libre. S'il n'y a pas assez de place pour les copies plus 512 Mo, il ne copie rien et affiche un avertissement.

### Pourquoi les copies sont conservées par défaut

Conserver les copies est l'option qui ménage le plus un SSD :

- **Copies conservées (par défaut)** : la première déconnexion copie toutes les sessions une fois. Ensuite, chaque déconnexion n'écrit que les sessions modifiées depuis la précédente, généralement quelques fichiers et quelques Mo, et se termine presque aussitôt.
- **Copies supprimées après la connexion** : chaque connexion supprime les copies, donc chaque déconnexion doit toutes les réécrire. Avec environ 300 sessions et 10 dossiers de compte/organisation, cela représente environ 600 Mo écrits à chaque changement d'équipe, plus la suppression d'environ 3 000 fichiers ensuite.

La contrepartie est l'espace disque : les copies restent sur le disque et occupent environ (taille totale des fichiers de session × nombre de dossiers de compte/organisation).

### Pourquoi pas de jonctions ni de liens physiques

Faire pointer tous les dossiers de compte vers un seul dossier réel éviterait les copies, mais l'application Claude bloque volontairement les deux façons de le faire :

- **Jonctions et liens symboliques** : avant chaque enregistrement, l'application vérifie que le dossier de sessions est un vrai dossier et refuse d'enregistrer s'il s'agit d'un lien (`Refusing non-directory at private dir path (symlink/file plant)`). Lors d'un test avec l'application réelle, la liste pouvait être lue via une jonction, mais tous les enregistrements de sessions et de tâches planifiées ont échoué, y compris pour de toutes nouvelles sessions.
- **Liens physiques** : l'application considère comme dangereux un fichier qui a plus d'un lien. Elle refuse d'y écrire et émet un avertissement lorsqu'elle le lit. Elle enregistre en outre en écrivant un fichier temporaire puis en le renommant, ce qui romprait le lien à chaque enregistrement.

Ces vérifications protègent contre des fichiers ou dossiers déposés depuis l'extérieur ; CCDC ne les contourne donc pas. Il copie des fichiers ordinaires, du même type que ceux qu'écrit l'application elle-même.

## Prérequis

- Windows 10 ou 11, avec l'application Claude pour ordinateur (installation MSIX)
- [Node.js](https://nodejs.org) 18 ou version ultérieure

## Installation

Téléchargez le programme d'installation depuis [Releases](../../releases), décompressez-le et lancez `setup.cmd`. Aucun droit d'administrateur n'est nécessaire.

- Installé dans `%LOCALAPPDATA%\Programs\CCDC`.
- Ajoute un raccourci `Continuous Claude Desktop Code` sur le Bureau et dans le menu Démarrer.
- Apparaît dans Paramètres > Applications > Applications installées sous le nom « Continuous Claude Desktop Code (CCDC) » ; c'est là que vous pouvez le désinstaller.
- Démarre dans la zone de notification dès la fin de l'installation.
- Pour qu'il démarre aussi lorsque vous vous connectez à Windows, utilisez `setup.cmd /startup` ; pour annuler ce démarrage automatique, utilisez `setup.cmd /nostartup`. Sans l'une ni l'autre de ces options, une réinstallation conserve le réglage précédent.
- Une réinstallation arrête CCDC s'il est en cours d'exécution et remplace ses fichiers ; vos réglages (`ccdc.json`) sont conservés.
- La désinstallation laisse en place les fichiers de session Claude et le dossier de sauvegarde de CCDC.

Pour l'exécuter plutôt directement depuis ce dépôt, lancez `ccdc-tray.vbs` ou utilisez la ligne de commande ci-dessous.

## Application de la zone de notification

Une icône orange avec un **C** apparaît dans la zone de notification (grise quand la synchronisation automatique est désactivée). Cliquez dessus pour ouvrir le menu :

| Élément du menu | Rôle |
| --- | --- |
| Ligne d'état | Indique si CCDC surveille les connexions et les déconnexions |
| Synchronisation automatique (détecter connexion/déconnexion) | Active ou désactive la copie et le nettoyage |
| Déplacer les sessions vers un compte maintenant | Regroupe toutes les sessions dans le dossier d'un compte ; demande confirmation si ce compte n'est pas connecté |
| Supprimer les copies après la connexion | Désactivé par défaut (les copies sont conservées ; voir plus haut) |
| Vitesse de copie/suppression | Rapide (sans limite), Normale (60 Mo/s) ou Douce (15 Mo/s). La suppression est limitée à 500 fichiers par seconde en Normale et à 100 en Douce |
| Ligne d'estimation | Indique la durée de la prochaine copie à la déconnexion et, si les copies sont supprimées, celle du nettoyage après la connexion |
| Language | Choisit la langue d'affichage. Cet élément est toujours affiché en anglais ; chaque langue du sous-menu est écrite dans cette langue |
| Afficher le journal | Ouvre `%LOCALAPPDATA%\CCDC\watch.log` |
| Nommer les comptes (ccdc.json) | Ouvre le fichier de réglages pour donner aux comptes des noms lisibles |
| Quitter | Retire l'icône ; si une copie est en cours, attend qu'elle se termine |

Des notifications s'affichent quand la copie commence (avec la durée estimée), quand elle est terminée, quand une connexion est détectée et quand le nettoyage est terminé. Plus la vitesse est lente, plus la copie dure : la personne suivante doit donc attendre la notification « Copie terminée », ou laisser passer à peu près la durée estimée, avant de se connecter.

L'estimation est calculée d'après l'état actuel des dossiers et les vitesses de copie et de suppression mesurées sur ce PC (enregistrées dans `%LOCALAPPDATA%\CCDC\state.json`).

Si l'icône n'apparaît pas sous Windows 11, regardez parmi les icônes cachées (`^`) de la barre des tâches. Pour qu'elle reste visible, activez Windows PowerShell dans Paramètres > Personnalisation > Barre des tâches > Autres icônes de la barre d'état système.

### Utilisation de la mémoire

L'application de la zone de notification surveille elle-même le journal de Claude, en ne lisant que ce qui a été ajouté depuis la dernière vérification. Node.js ne s'exécute que pendant une copie ou un nettoyage et se termine ensuite ; entre deux changements d'équipe, seule l'application de la zone de notification s'exécute. Au repos, elle rend la mémoire inutilisée à Windows environ une fois par minute (environ 15 Mo utilisés).

## Réglages

Les réglages sont enregistrés dans `ccdc.json`, dans le dossier du programme. Les modifications faites dans le menu de la zone de notification y sont enregistrées et s'appliquent à la prochaine copie ou au prochain nettoyage, sans redémarrage.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Clé | Valeurs |
| --- | --- |
| `keepCopies` | `true` (par défaut) : les copies sont conservées. `false` : elles sont supprimées après la connexion |
| `speed` | `"fast"` (par défaut), `"normal"`, `"gentle"` |
| `language` | Vide pour suivre la langue de Windows, ou un code de langue comme `"ja"` pour utiliser cette langue |
| `aliases` | Noms affichés pour les comptes dans le menu de la zone de notification |

## Langues

Le menu de la zone de notification, les notifications, les avertissements et les fenêtres d'installation/désinstallation suivent la langue d'affichage de Windows, ou la langue choisie sous **Language** dans le menu de la zone de notification. Langues prises en charge : anglais, coréen, japonais, chinois (simplifié et traditionnel), espagnol, français, allemand, italien, portugais, russe, ukrainien, polonais, néerlandais, turc, vietnamien, indonésien, thaï, arabe et hindi. Les autres langues s'affichent en anglais.

Tous les textes de l'interface se trouvent dans une seule feuille, [`locales.csv`](locales.csv) : **une ligne par langue**, une colonne par message. Ouvrez-la dans Excel ou Google Sheets ; pour ajouter une langue, ajoutez une ligne.

- `lang` : un nom de culture Windows (`sv`, `pt-BR`, `zh-Hant`, …). La ligne qui correspond à la langue et à la région est prioritaire, puis celle qui correspond à la langue seule.
- `name` : le nom de la langue dans cette langue (`Svenska`) ; le menu Language l'affiche.
- Écrivez les retours à la ligne dans une cellule sous la forme `\n`, et conservez `{0}`, `{1}` (des nombres ou des noms y sont insérés).
- Les cellules vides s'affichent en anglais. Enregistrez le fichier au format CSV UTF-8.

La sortie de la ligne de commande et `watch.log` restent en anglais, car l'application de la zone de notification les lit.

## Ligne de commande

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

L'application de la zone de notification lance elle-même ces tâches ponctuelles : `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` et `node ccdc.mjs startup` (pour se mettre à jour selon l'état actuel de l'application Claude, au démarrage de CCDC ou quand la synchronisation automatique est réactivée).

## Règles pour les doublons

- Quand une session se trouve dans plusieurs dossiers, la copie dont le `lastActivityAt` est le plus récent l'emporte. Les copies gardent l'heure de modification de la source ; des copies identiques sont donc reconnues d'après la taille et l'heure, sans être lues.
- Quand une copie plus récente remplace un fichier, l'ancien fichier est déplacé vers `ccdc-backups\<heure>\` dans le dossier de données de l'application.
- Les sessions portant un marqueur `deleted_<id>` ne sont pas copiées, et leurs copies dans les autres dossiers sont supprimées.
- Un compte ou une organisation qui se connecte pour la toute première fois n'a pas encore de dossier : rien n'a donc pu y être copié à la déconnexion. Ses sessions s'affichent alors après sa prochaine connexion ou un redémarrage de l'application.

## Mises en garde

- CCDC dépend de l'organisation interne des fichiers et du texte du journal de l'application Claude pour ordinateur ; une mise à jour de l'application peut modifier son comportement.
- Les fichiers de session contiennent des réglages de connecteurs (MCP) propres à chaque compte : certains connecteurs peuvent ne pas se connecter quand une session est ouverte sous un autre compte.
- Les tâches planifiées ne sont pas partagées ; chaque dossier de compte/organisation garde les siennes.
- Le changement de compte dans la CLI de Claude Code n'est pas pris en charge. La CLI peut déjà ouvrir n'importe quelle session de `~/.claude/projects` avec `--resume`, quel que soit le compte.

## Licence

[MIT](LICENSE)
