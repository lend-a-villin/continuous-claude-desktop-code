# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | **Español** | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Una aplicación de bandeja para Windows que mantiene las sesiones locales de **Code** de la aplicación de escritorio de Claude disponibles para cualquier cuenta de Claude que inicie sesión en el PC.

Herramienta no oficial; no está afiliada a Anthropic ni cuenta con su respaldo.

## Para quién es

CCDC está pensada para **lugares de trabajo que funcionan las 24 horas en tres turnos**. En cada turno, una persona distinta se sienta en el mismo puesto (un PC, un usuario de Windows) e inicia sesión en la aplicación de escritorio de Claude con su propia cuenta de Claude. Con CCDC, el turno siguiente inicia sesión y retoma el trabajo de Code que dejó el turno anterior.

Por eso, todas las sesiones de Code del PC son visibles para cualquiera que inicie sesión en él. No uses CCDC donde las personas no deban ver las sesiones de las demás. Si usas una cuenta de empresa (organización), revisa también la política de tu empresa.

## El problema que resuelve

La aplicación de escritorio guarda la **lista** de sesiones de Code en una carpeta distinta para cada cuenta y organización:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

Las **conversaciones** en sí están en `~/.claude/projects\...\<cliSessionId>.jsonl`, que comparten todas las cuentas. Por eso, cuando alguien inicia sesión con otra cuenta, la lista de sesiones de Code aparece vacía aunque las conversaciones sigan en el disco.

## Cómo funciona

La aplicación lee su carpeta de sesiones **solo al arrancar y al iniciar sesión** (dos veces, en unos dos segundos). No hay forma de hacer que vuelva a cargar únicamente la lista más tarde. El propio registro de la aplicación, `%LOCALAPPDATA%\Claude\Logs\main.log`, registra tanto los cierres como los inicios de sesión:

- cierre de sesión: `[account] Login-state transition (loggedOut: false → true ...)`
- inicio de sesión: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC sigue ese registro y actúa así:

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

## Requisitos

- Windows 10 u 11, con la aplicación de escritorio de Claude (instalación MSIX)
- [Node.js](https://nodejs.org) 18 o posterior

## Instalación

Descarga el instalador desde [Releases](../../releases), descomprímelo y ejecuta `setup.cmd`. No necesitas permisos de administrador.

- Se instala en `%LOCALAPPDATA%\Programs\CCDC`.
- Crea un acceso directo `Continuous Claude Desktop Code` en el escritorio y en el menú Inicio.
- Aparece en Configuración > Aplicaciones > Aplicaciones instaladas como «Continuous Claude Desktop Code (CCDC)»; desinstálala desde allí.
- Se inicia en la bandeja justo después de instalarse.
- Para que arranque al iniciar sesión en Windows, instala con `setup.cmd /startup`; para desactivarlo, usa `setup.cmd /nostartup`. Si reinstalas sin ninguna de las dos opciones, se conserva el ajuste anterior.
- Al reinstalar se detiene CCDC si está en ejecución y se reemplazan sus archivos; tus ajustes (`ccdc.json`) se conservan.
- Al desinstalar, los archivos de sesión de Claude y la carpeta de copias de seguridad de CCDC se quedan como están.

Para ejecutarla directamente desde este repositorio, ejecuta `ccdc-tray.vbs` o usa la línea de comandos de abajo.

## Aplicación de bandeja

Aparece una **C** naranja en el área de notificación (gris mientras la sincronización automática está desactivada). Haz clic en ella para abrir el menú:

| Elemento del menú | Qué hace |
| --- | --- |
| Línea de estado | Indica si CCDC está vigilando los inicios y cierres de sesión |
| Sincronización automática (detectar inicio/cierre de sesión) | Activa o desactiva la copia y la limpieza |
| Mover las sesiones a una cuenta ahora | Reúne todas las sesiones en la carpeta de una cuenta; pide confirmación primero si esa cuenta no tiene la sesión iniciada |
| Eliminar las copias tras iniciar sesión | Desactivado por defecto (las copias se conservan; ver arriba) |
| Velocidad de copia/eliminación | Rápida (sin límite), Normal (60 MB/s) o Suave (15 MB/s). En Normal y Suave, la eliminación se limita a 500 y 100 archivos por segundo, respectivamente |
| Línea de estimación | Muestra cuánto tardaría la copia en el próximo cierre de sesión y, si se eliminan las copias, también la limpieza tras el inicio de sesión |
| Language | Elige el idioma de la interfaz. Este elemento siempre aparece en inglés; cada idioma del submenú está escrito en ese idioma |
| Ver registro | Abre `%LOCALAPPDATA%\CCDC\watch.log` |
| Poner nombre a las cuentas (ccdc.json) | Abre el archivo de ajustes para dar a las cuentas nombres legibles |
| Salir | Quita el icono; si hay una copia en curso, espera a que termine |

Se muestran notificaciones cuando empieza la copia (con el tiempo estimado), cuando termina, cuando se detecta un inicio de sesión y cuando termina la limpieza. Con una velocidad baja, la copia tarda más, así que la siguiente persona debe esperar a la notificación «Copia terminada», o aproximadamente el tiempo estimado, antes de iniciar sesión.

La estimación se calcula a partir del estado actual de las carpetas y de las velocidades de copia y eliminación medidas en este PC (se guardan en `%LOCALAPPDATA%\CCDC\state.json`).

Si el icono no aparece en Windows 11, búscalo entre los iconos ocultos (`^`) de la barra de tareas. Para que se vea siempre, activa Windows PowerShell en Configuración > Personalización > Barra de tareas > Otros iconos de la bandeja del sistema.

### Uso de memoria

La bandeja vigila por sí misma el registro de la aplicación y lee solo lo que se ha añadido desde la última comprobación. Node.js se ejecuta solo mientras copia o limpia y se cierra después, así que entre cambios de turno solo está en marcha la bandeja. Mientras está inactiva, la bandeja devuelve a Windows la memoria que no usa aproximadamente una vez por minuto (unos 15 MB en uso).

## Ajustes

Los ajustes se guardan en `ccdc.json`, en la carpeta del programa. Lo que cambies en el menú de la bandeja se guarda en este archivo y se aplica a la siguiente copia o limpieza, sin reiniciar.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Clave | Valores |
| --- | --- |
| `keepCopies` | `true` (por defecto): se conservan las copias. `false`: se eliminan tras iniciar sesión |
| `speed` | `"fast"` (por defecto), `"normal"`, `"gentle"` |
| `language` | Vacío para seguir el idioma de Windows, o un código de idioma como `"ja"` para usar ese idioma |
| `aliases` | Nombres que se muestran para las cuentas en el menú de la bandeja |

## Idiomas

El menú de la bandeja, las notificaciones, las advertencias y los cuadros de diálogo de instalación y desinstalación siguen el idioma para mostrar de Windows, o el idioma elegido en **Language** en el menú de la bandeja. Idiomas disponibles: inglés, coreano, japonés, chino (simplificado y tradicional), español, francés, alemán, italiano, portugués, ruso, ucraniano, polaco, neerlandés, turco, vietnamita, indonesio, tailandés, árabe e hindi. Los demás idiomas se muestran en inglés.

Todos los textos de la interfaz están en una sola hoja, [`locales.csv`](locales.csv): **una fila por idioma** y una columna por mensaje. Ábrela en Excel o Google Sheets; para añadir un idioma, añade una fila.

- `lang`: un nombre de referencia cultural de Windows (`sv`, `pt-BR`, `zh-Hant`…). Se usa primero la fila que coincide en idioma y región, y si no, la que coincide en idioma.
- `name`: el nombre del idioma en ese mismo idioma (`Svenska`); es lo que muestra el menú Language.
- Escribe los saltos de línea dentro de una celda como `\n` y mantén `{0}`, `{1}` (ahí van números o nombres).
- Las celdas vacías se muestran en inglés. Guarda el archivo como CSV UTF-8.

La salida de la línea de comandos y `watch.log` se quedan en inglés porque la bandeja los lee.

## Línea de comandos

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Cuando hace falta, la bandeja ejecuta estas tareas puntuales: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` y `node ccdc.mjs startup` (para ponerse al día con el estado actual de la aplicación cuando arranca la bandeja o se vuelve a activar la sincronización automática).

## Reglas para duplicados

- Cuando una sesión está en varias carpetas, gana la copia con el `lastActivityAt` más reciente. Las copias conservan la fecha de modificación del origen, así que las copias idénticas se reconocen por tamaño y fecha sin necesidad de leerlas.
- Cuando una copia más reciente reemplaza un archivo, el archivo antiguo se mueve a `ccdc-backups\<hora>\` en la carpeta de datos de la aplicación.
- Las sesiones con una marca `deleted_<id>` no se copian, y sus copias en otras carpetas se eliminan.
- Una cuenta/organización que inicia sesión por primera vez aún no tiene carpeta, así que no se pudo copiar nada allí al cerrar sesión. En ese caso, las sesiones aparecen tras el siguiente inicio de sesión de esa cuenta o al reiniciar la aplicación.

## Limitaciones

- CCDC depende de la estructura interna de archivos y del texto del registro de la aplicación de escritorio de Claude; una actualización de la aplicación puede cambiar su comportamiento.
- Los archivos de sesión contienen ajustes de conectores (MCP) de cada cuenta, así que algunos conectores pueden no conectarse al abrir una sesión desde otra cuenta.
- Las tareas programadas no se comparten; cada carpeta de cuenta/organización conserva las suyas.
- CCDC no cubre el cambio de cuenta en la CLI de Claude Code. La CLI ya puede abrir cualquier sesión de `~/.claude/projects` con `--resume`, sea cual sea la cuenta.

## Licencia

[MIT](LICENSE)
