# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | **Español** | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Una aplicación de bandeja que conserva tus sesiones de **Code** de la aplicación de escritorio de Claude cuando cambias de cuenta de Claude.

La aplicación de escritorio de Claude guarda una lista de sesiones de Code distinta para cada cuenta y organización. Si inicias sesión con otra cuenta, la lista aparece vacía, aunque las conversaciones sigan en el equipo. CCDC copia las sesiones de una cuenta a otra, de modo que la siguiente cuenta que inicie sesión las vea todas.

Herramienta no oficial; no está afiliada a Anthropic ni cuenta con su respaldo.

**[Descargar la última versión](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · no hace falta instalar nada más

| Plataforma | Versión | Estado |
| --- | --- | --- |
| Windows | 1.0 | Nueva; probada con la aplicación real de Claude en Windows, donde funciona como se describe (continuación de los scripts 0.3) |
| macOS | 0.1 | Nueva; aún no se ha probado con la aplicación real de Claude en un Mac |
| Linux | 0.1 | Nueva; instalación probada en Debian 12, Ubuntu 24.04 y Linux Mint 22, pero aún no con la aplicación real de Claude |

## Para quién es

- **Equipos compartidos y trabajo por turnos.** En un lugar de trabajo que funciona las 24 horas en tres turnos, en cada turno se sienta una persona distinta en el mismo puesto (un equipo, un usuario del sistema) e inicia sesión con su propia cuenta de Claude. Con CCDC, el turno siguiente retoma el trabajo de Code que dejó el anterior.
- **Una persona, varias cuentas.** Si en el mismo equipo alternas entre una cuenta personal y una de trabajo, o entre organizaciones, tus sesiones de Code estarán ahí inicies sesión con la cuenta que inicies.

Todas las sesiones de Code del equipo quedan visibles para cualquier cuenta que inicie sesión en él. No uses CCDC donde las personas no deban ver las sesiones de las demás. Si usas una cuenta de empresa (organización), revisa antes la política de tu empresa: CCDC también copia las sesiones de una cuenta de trabajo en la carpeta de una cuenta personal, y viceversa.

## El problema que resuelve

La aplicación de escritorio guarda la **lista** de sesiones de Code en una carpeta distinta para cada cuenta y organización:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Carpeta de datos de Claude | Registro de Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (o `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Las organizaciones que usan Claude a través de su propio proveedor de nube usan `Claude-3p` en lugar de `Claude`; CCDC también busca allí).

Las **conversaciones** en sí están en `~/.claude/projects/.../<cliSessionId>.jsonl`, que comparten todas las cuentas. Por eso, cuando alguien inicia sesión con otra cuenta, la lista de sesiones de Code aparece vacía aunque las conversaciones sigan en el disco.

## Cómo funciona

La aplicación lee su carpeta de sesiones **solo al arrancar y al iniciar sesión** (dos veces, en unos dos segundos). No hay forma de hacer que vuelva a cargar únicamente la lista más tarde. El propio registro de la aplicación deja constancia de ambos eventos:

- cierre de sesión: `[account] Login-state transition (loggedOut: false → true ...)`
- inicio de sesión: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC sigue ese registro:

1. **Cierre de sesión** → copia la versión más reciente de cada sesión en todas las carpetas de cuenta/organización, de modo que la siguiente cuenta que inicie sesión ya las encuentre en su propia carpeta. Solo se copian las sesiones que faltan o están desactualizadas en una carpeta.
2. **Inicio de sesión** → por defecto, las copias se quedan donde están. Si está activada la opción «Eliminar las copias tras iniciar sesión», CCDC espera a que la aplicación haya leído su lista (6 segundos después de la última línea de inicio de sesión) y luego elimina las copias de todas las carpetas excepto la de la cuenta que inició sesión.

Una sesión eliminada en la aplicación también se elimina de las demás carpetas, así que no reaparece con otra cuenta.

Antes de copiar, CCDC comprueba el espacio libre en disco. Si no hay sitio para las copias más 512 MB, no copia nada y muestra un aviso.

### Por qué se conservan las copias por defecto

Conservar las copias es la opción que menos desgasta el SSD:

- **Copias conservadas (por defecto)**: el primer cierre de sesión copia todas las sesiones una vez. Después, cada cierre de sesión escribe solo las sesiones que han cambiado desde el anterior, normalmente unos pocos archivos y unos pocos MB, y termina casi al instante.
- **Copias eliminadas tras iniciar sesión**: cada inicio de sesión elimina las copias, así que cada cierre de sesión tiene que escribirlas todas de nuevo. Con unas 300 sesiones y 10 carpetas de cuenta/organización, son unos 600 MB escritos en cada cambio de turno, más la eliminación posterior de unos 3.000 archivos.

El precio de conservarlas es el espacio en disco: las copias permanecen en él y ocupan aproximadamente (tamaño total de los archivos de sesión × número de carpetas de cuenta/organización).

### Por qué no se usan uniones ni vínculos físicos

Hacer que todas las carpetas de cuenta apunten a una sola carpeta real evitaría copiar, pero la aplicación de Claude bloquea a propósito ambas formas de hacerlo:

- **Uniones y vínculos simbólicos**: antes de cada guardado, la aplicación comprueba que la carpeta de sesiones sea una carpeta real y se niega a guardar si es un vínculo (`Refusing non-directory at private dir path (symlink/file plant)`). En una prueba con la aplicación real, la lista se podía leer a través de una unión, pero fallaron todos los guardados de sesiones y de tareas programadas, incluidos los de sesiones recién creadas.
- **Vínculos físicos**: la aplicación considera inseguro un archivo con más de un vínculo. Se niega a escribir en esos archivos y avisa al leerlos. Además, guarda escribiendo un archivo temporal y renombrándolo, lo que rompería el vínculo en cada guardado.

Estas comprobaciones protegen contra archivos o carpetas colocados desde fuera, así que CCDC no las elude. Copia archivos normales, del mismo tipo que escribe la propia aplicación.

## Instalación

Descarga el archivo para tu sistema desde [Releases](../../releases). Los archivos no tienen firma de código (consulta [más abajo](#acerca-del-aviso-de-seguridad)).

**Windows**: ejecuta `CCDC-<version>-windows-x64.msi`. No necesitas permisos de administrador.

- Se instala en `%LOCALAPPDATA%\Programs\CCDC`, crea un acceso directo `Continuous Claude Desktop Code` en el escritorio y en el menú Inicio, y deja CCDC en marcha en la bandeja.
- Se inicia con el equipo; para desactivarlo, usa «Iniciar con el equipo» en el menú de la bandeja.
- Para actualizar, ejecuta el `.msi` más reciente. Cierra CCDC, lo reemplaza y lo vuelve a iniciar; tus ajustes se conservan.
- Para desinstalar, ve a Configuración > Aplicaciones > Aplicaciones instaladas > «Continuous Claude Desktop Code (CCDC)».
- Si vienes de los scripts 0.3, basta con instalar el `.msi`. La primera vez que se inicia, CCDC detiene la versión anterior, conserva sus ajustes y los nombres de las cuentas, y la elimina.

**macOS**: abre `CCDC-<version>-macos-universal.dmg`, arrastra **CCDC** a Aplicaciones y ábrelo. Aparece una **C** en la barra de menús. Se inicia con el equipo, salvo que lo desactives en su menú. Funciona en Apple Silicon e Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Después, abre **Continuous Claude Desktop Code** desde el menú de aplicaciones. Se inicia con el equipo, salvo que lo desactives en su menú. En GNOME sin la extensión AppIndicator, el icono de la bandeja no puede mostrarse (Ubuntu y Linux Mint la incluyen).

Al desinstalar, los archivos de sesión de Claude, los ajustes de CCDC y la carpeta de copias de seguridad se quedan como están.

### Acerca del aviso de seguridad

Firmar el código requiere un certificado de pago vinculado a una persona o empresa, y CCDC es un pequeño proyecto gratuito, así que las descargas no están firmadas. Tu sistema te avisará la primera vez; eso solo significa que el editor no está verificado, no que se haya encontrado nada dañino.

CCDC no contiene nada dañino. No se conecta a internet, no recopila datos y no modifica nada fuera de las carpetas de sesiones de la aplicación de Claude y de su propia carpeta. Todo el código está en este repositorio, y los archivos de cada versión se compilan a partir de él con GitHub Actions, cuyos registros son públicos. Puedes comprobar una descarga con el archivo `SHA256SUMS.txt` de la versión, o [compilarlo tú mismo](#compilación).

- **Windows**: si SmartScreen muestra «Windows protegió su PC», haz clic en **Más información** y luego en **Ejecutar de todas formas**.
- **macOS**: si macOS indica que no puede verificar la aplicación, abre **Ajustes del Sistema > Privacidad y seguridad**, desplázate hacia abajo y haz clic en **Abrir igualmente** junto a CCDC. (O ejecuta `xattr -dr com.apple.quarantine /Applications/CCDC.app` en Terminal).
- **Linux**: los paquetes `.deb` no se comprueban de esta forma.

## Aplicación de bandeja

Aparece una **C** naranja en el área de notificación o en la barra de menús (gris mientras la sincronización automática está desactivada). Haz clic en ella para abrir el menú:

| Elemento del menú | Qué hace |
| --- | --- |
| Línea de estado | Indica si CCDC está vigilando los inicios y cierres de sesión |
| Sincronización automática (detectar inicio/cierre de sesión) | Activa o desactiva la copia y la limpieza |
| Mover las sesiones a una cuenta ahora | Reúne todas las sesiones en la carpeta de una cuenta; pide confirmación primero si esa cuenta no tiene la sesión iniciada |
| Eliminar las copias tras iniciar sesión | Desactivado por defecto (las copias se conservan; ver arriba) |
| Velocidad de copia/eliminación | Rápida (sin límite), Normal (60 MB/s) o Suave (15 MB/s). En Normal y Suave, la eliminación se limita a 500 y 100 archivos por segundo, respectivamente |
| Línea de estimación | Muestra cuánto tardaría la copia en el próximo cierre de sesión y, si se eliminan las copias, también la limpieza tras el inicio de sesión |
| Iniciar con el equipo | Inicia CCDC al iniciar sesión en el equipo |
| Language | Elige el idioma de la interfaz. Este elemento siempre aparece en inglés; cada idioma del submenú está escrito en ese idioma |
| Ver registro | Abre el `watch.log` de CCDC |
| Poner nombre a las cuentas (ccdc.json) | Abre el archivo de ajustes para dar a las cuentas nombres legibles |
| Salir | Quita el icono; si hay una copia en curso, espera a que termine |

Se muestran notificaciones cuando empieza la copia (con el tiempo estimado), cuando termina, cuando se detecta un inicio de sesión y cuando termina la limpieza. Con una velocidad más baja, la copia tarda más, así que la siguiente persona debe esperar a la notificación «Copia terminada», o aproximadamente el tiempo estimado, antes de iniciar sesión.

La estimación se calcula a partir del estado actual de las carpetas y de las velocidades de copia y eliminación medidas en este equipo (se guardan en `state.json`, en la carpeta de CCDC).

Si el icono no aparece en Windows 11, búscalo entre los iconos ocultos (`^`) de la barra de tareas. Para que se vea siempre, activa CCDC en Configuración > Personalización > Barra de tareas > Otros iconos de la bandeja del sistema.

CCDC es un único programa nativo de aproximadamente 1 MB. Mientras espera, cada medio segundo lee solo lo que se ha añadido al registro de Claude desde la última comprobación, y usa unos 4 MB de memoria propia.

## Ajustes

Los ajustes se guardan en `ccdc.json`, en la carpeta de CCDC:

| | Carpeta de CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Lo que cambies en el menú de la bandeja se guarda en este archivo y se aplica a partir de la siguiente copia o limpieza, sin reiniciar. `watch.log` y `state.json` están en la misma carpeta.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Clave | Valores |
| --- | --- |
| `keepCopies` | `true` (por defecto): se conservan las copias. `false`: se eliminan tras iniciar sesión |
| `speed` | `"fast"` (por defecto), `"normal"`, `"gentle"` |
| `language` | Vacío para seguir el idioma del sistema, o un código de idioma como `"ja"` para usar ese idioma |
| `startAtLogin` | Lo establece «Iniciar con el equipo» en el menú de la bandeja |
| `aliases` | Nombres que se muestran para las cuentas en el menú de la bandeja |
| `dataDir`, `appLog` | Solo si CCDC no encuentra por sí mismo la carpeta de datos o el registro de Claude: sus rutas completas |

## Idiomas

El menú de la bandeja, las notificaciones y las advertencias siguen el idioma para mostrar del sistema, o el idioma elegido en **Language** en el menú de la bandeja. Idiomas disponibles: inglés, coreano, japonés, chino (simplificado y tradicional), español, francés, alemán, italiano, portugués, ruso, ucraniano, polaco, neerlandés, turco, vietnamita, indonesio, tailandés, árabe e hindi. Los demás idiomas se muestran en inglés.

Todos los textos de la interfaz están en una sola hoja, [`locales.csv`](locales.csv), que se integra en el programa: **una fila por idioma** y una columna por mensaje. Ábrela en Excel o Google Sheets; para añadir un idioma, añade una fila y [compila](#compilación) CCDC.

- `lang`: una etiqueta de idioma (`sv`, `pt-BR`, `zh-Hant`…). Se usa primero la fila que coincide en idioma y región, y si no, la que coincide en idioma.
- `name`: el nombre del idioma en ese mismo idioma (`Svenska`); es lo que muestra el menú Language.
- Escribe los saltos de línea dentro de una celda como `\n` y mantén `{0}`, `{1}` (ahí van números o nombres).
- Las celdas vacías se muestran en inglés. Guarda el archivo como CSV UTF-8.

La salida de la línea de comandos y `watch.log` se quedan en inglés.

## Línea de comandos

El mismo programa también funciona desde un terminal (en Windows, `ccdc.exe` en `%LOCALAPPDATA%\Programs\CCDC`; en macOS, `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

Con `--data-dir <path>` y `--app-log <path>` puedes indicarle a mano una carpeta de datos o un registro de Claude.

## Reglas para duplicados

- Cuando una sesión está en varias carpetas, gana la copia con el `lastActivityAt` más reciente. Las copias conservan la fecha de modificación del origen, así que las copias idénticas se reconocen por tamaño y fecha sin necesidad de leerlas.
- Cuando una copia más reciente reemplaza un archivo, el archivo antiguo se mueve a `ccdc-backups/<time>/` en la carpeta de datos de Claude.
- Las sesiones con una marca `deleted_<id>` no se copian, y sus copias en otras carpetas se eliminan.
- Una cuenta/organización que inicia sesión por primera vez aún no tiene carpeta, así que no se pudo copiar nada allí al cerrar sesión. En ese caso, las sesiones aparecen tras el siguiente inicio de sesión de esa cuenta o al reiniciar la aplicación.

## Limitaciones

- CCDC depende de la estructura interna de archivos y del texto del registro de la aplicación de escritorio de Claude; una actualización de la aplicación puede cambiar su comportamiento.
- Los archivos de sesión contienen ajustes de conectores (MCP) de cada cuenta, así que algunos conectores pueden no conectarse al abrir una sesión desde otra cuenta.
- Las tareas programadas no se comparten; cada carpeta de cuenta/organización conserva las suyas.
- CCDC no cubre el cambio de cuenta en la CLI de Claude Code. La CLI ya puede abrir cualquier sesión de `~/.claude/projects` con `--resume`, sea cual sea la cuenta.

## Compilación

CCDC está escrito en Rust; [instala Rust](https://rustup.rs) y luego:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

La versión de cada plataforma se define en `[package.metadata.ccdc]`, en `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) compila las tres en cada push y, con una etiqueta `v*`, las adjunta a una versión en borrador.

## Licencia

[MIT](LICENSE)
