# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | **Português** | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Um aplicativo de bandeja que mantém suas sessões do **Code** do aplicativo Claude para desktop quando você troca de conta do Claude.

O aplicativo Claude para desktop guarda uma lista separada de sessões do Code para cada conta e organização. Ao fazer login com outra conta, a lista parece vazia, embora as conversas continuem no computador. O CCDC copia as sessões entre as contas, para que qualquer conta que fizer login em seguida veja todas elas.

Não oficial; não é afiliado nem endossado pela Anthropic.

**[Baixe a versão mais recente](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · nada mais para instalar

| Plataforma | Versão | Situação |
| --- | --- | --- |
| Windows | 1.0 | Nova; testada com o aplicativo Claude real no Windows, onde funciona como descrito (sucessora dos scripts da 0.3) |
| macOS | 0.1 | Nova; ainda não testada com o aplicativo Claude real em um Mac |
| Linux | 0.1 | Nova; instalação testada no Debian 12, Ubuntu 24.04 e Linux Mint 22, ainda não com o aplicativo Claude real |

## Para quem é

- **Computadores compartilhados e trabalho em turnos.** Em um local de trabalho que funciona 24 horas em três turnos, uma pessoa diferente ocupa o mesmo posto (um computador, um usuário do sistema) a cada turno e faz login com a própria conta do Claude. Com o CCDC, o turno seguinte continua o trabalho no Code que o turno anterior deixou.
- **Uma pessoa, várias contas.** Se você alterna entre uma conta pessoal e uma do trabalho, ou entre organizações, no mesmo computador, suas sessões do Code estarão lá seja qual for a conta com que você fizer login.

Todas as sessões do Code no computador ficam visíveis para todas as contas que fizerem login nele. Não use o CCDC onde as pessoas não devam ver as sessões umas das outras. Se você usa uma conta da empresa (organização), consulte antes a política da empresa: o CCDC também copia as sessões de uma conta de trabalho para a pasta de uma conta pessoal, e vice-versa.

## O problema que ele resolve

O aplicativo para desktop guarda a **lista** de sessões do Code em uma pasta separada para cada conta e organização:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Pasta de dados do Claude | Log do Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (ou `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Organizações que usam o Claude pelo próprio provedor de nuvem usam `Claude-3p` em vez de `Claude`; o CCDC também procura lá.)

As **conversas** em si ficam em `~/.claude/projects/.../<cliSessionId>.jsonl`, que todas as contas compartilham. Por isso, depois que alguém faz login com outra conta, a lista de sessões do Code parece vazia, embora as conversas continuem no disco.

## Como funciona

O aplicativo lê a pasta de sessões **somente ao ser iniciado e quando alguém faz login** (duas vezes, em cerca de dois segundos). Não há como fazê-lo reler só a lista mais tarde. O log do próprio aplicativo registra os dois eventos:

- logout: `[account] Login-state transition (loggedOut: false → true ...)`
- login: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

O CCDC acompanha esse log:

1. **Logout** → copia a versão mais recente de cada sessão para todas as pastas de conta/organização, de modo que a próxima conta a fazer login já as encontre na própria pasta. Só são copiadas as sessões que faltam ou estão desatualizadas em cada pasta.
2. **Login** → por padrão, as cópias ficam onde estão. Se a opção "Excluir as cópias após o login" estiver ativada, o CCDC espera o aplicativo ler a lista (6 segundos após a última linha de login) e depois remove as cópias de todas as pastas, exceto a da conta conectada.

Uma sessão excluída no aplicativo também é removida das outras pastas, para não reaparecer em outra conta.

Antes de copiar, o CCDC verifica o espaço livre em disco. Se não houver espaço para as cópias mais 512 MB, ele não copia nada e mostra um aviso.

### Por que as cópias são mantidas por padrão

Manter as cópias é a opção que menos desgasta o SSD:

- **Cópias mantidas (padrão)**: o primeiro logout copia todas as sessões uma vez. Depois disso, cada logout grava só as sessões que mudaram desde o anterior, em geral alguns arquivos e alguns MB, e termina quase imediatamente.
- **Cópias excluídas após o login**: cada login apaga as cópias, então cada logout precisa gravar todas as sessões de novo. Com cerca de 300 sessões e 10 pastas de conta/organização, são aproximadamente 600 MB gravados a cada troca de turno, mais a exclusão de uns 3.000 arquivos depois.

Em troca, as cópias ocupam espaço em disco: elas continuam no lugar e ocupam cerca de (tamanho total dos arquivos de sessão × número de pastas de conta/organização).

### Por que não usar junções nem links físicos

Fazer todas as pastas de conta apontarem para uma única pasta real evitaria cópias, mas o aplicativo Claude bloqueia de propósito as duas formas de fazer isso:

- **Junções e links simbólicos**: antes de cada salvamento, o aplicativo verifica se a pasta de sessões é uma pasta real e se recusa a salvar se for um link (`Refusing non-directory at private dir path (symlink/file plant)`). Em um teste com o aplicativo real, a lista podia ser lida por meio de uma junção, mas todos os salvamentos de sessões e de tarefas agendadas falharam, inclusive os de sessões recém-criadas.
- **Links físicos**: o aplicativo considera inseguro um arquivo com mais de um link. Ele se recusa a gravar nesses arquivos e avisa ao lê-los. Além disso, ele salva gravando um arquivo temporário e renomeando-o, o que quebraria o link a cada salvamento.

Essas verificações protegem contra arquivos ou pastas inseridos de fora, então o CCDC não as contorna. Ele copia arquivos comuns, do mesmo tipo que o próprio aplicativo grava.

## Instalação

Baixe o arquivo do seu sistema em [Releases](../../releases). Os arquivos não têm assinatura de código (veja [abaixo](#sobre-o-aviso-de-segurança)).

**Windows** — execute `CCDC-<version>-windows-x64.msi`. Não são necessários direitos de administrador.

- Instala em `%LOCALAPPDATA%\Programs\CCDC`, cria um atalho `Continuous Claude Desktop Code` na área de trabalho e no menu Iniciar e inicia o CCDC na bandeja.
- Inicia junto com o computador; para desativar isso, use "Iniciar com o computador" no menu da bandeja.
- Atualização: execute o `.msi` mais novo. Ele fecha o CCDC, substitui os arquivos e o inicia de novo; suas configurações são mantidas.
- Para desinstalar, vá em Configurações > Aplicativos > Aplicativos instalados > "Continuous Claude Desktop Code (CCDC)".
- Vindo dos scripts da 0.3: basta instalar o `.msi`. Na primeira execução, o CCDC encerra a versão antiga, mantém as configurações e os nomes de conta dela e a remove.

**macOS** — abra `CCDC-<version>-macos-universal.dmg`, arraste o **CCDC** para Aplicativos e abra-o. Um **C** aparece na barra de menus. Ele inicia junto com o computador, a menos que você desative isso no menu dele. Funciona em Apple Silicon e Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Depois, abra o **Continuous Claude Desktop Code** pelo menu de aplicativos. Ele inicia junto com o computador, a menos que você desative isso no menu dele. No GNOME sem a extensão AppIndicator, o ícone da bandeja não aparece (o Ubuntu e o Linux Mint já a incluem).

Desinstalar deixa como estão os arquivos de sessão do Claude, as configurações do CCDC e a pasta de backup.

### Sobre o aviso de segurança

Assinar exige um certificado pago, vinculado a uma pessoa ou empresa, e o CCDC é um projeto pequeno e gratuito, por isso os downloads não são assinados. O sistema mostra um aviso na primeira vez; isso significa apenas que o editor não foi verificado, não que algo prejudicial tenha sido encontrado.

O CCDC não contém nada prejudicial. Ele não se conecta à internet, não coleta dados e não altera nada fora das pastas de sessões do aplicativo Claude e da própria pasta. Todo o código está neste repositório, e os arquivos de cada versão são compilados a partir dele pelo GitHub Actions, cujos logs são públicos. Você pode conferir um download com o `SHA256SUMS.txt` da versão ou [compilá-lo você mesmo](#compilação).

- **Windows**: se o SmartScreen disser "O Windows protegeu o computador", clique em **Mais informações** e depois em **Executar assim mesmo**.
- **macOS**: se o macOS disser que não pode verificar o aplicativo, abra **Ajustes do Sistema > Privacidade e Segurança**, role para baixo e clique em **Abrir Mesmo Assim** ao lado do CCDC. (Ou execute `xattr -dr com.apple.quarantine /Applications/CCDC.app` no Terminal.)
- **Linux**: pacotes `.deb` não passam por essa verificação.

## Aplicativo de bandeja

Um **C** laranja aparece na área de notificação ou na barra de menus (cinza enquanto a sincronização automática está desativada). Clique nele para abrir o menu:

| Item do menu | O que faz |
| --- | --- |
| Linha de status | Mostra se o CCDC está monitorando logins e logouts |
| Sincronização automática (detectar login/logout) | Ativa ou desativa a cópia e a limpeza |
| Mover sessões para uma conta agora | Reúne todas as sessões na pasta de uma conta; pergunta antes se essa conta não estiver conectada |
| Excluir as cópias após o login | Desativado por padrão (as cópias são mantidas; veja acima) |
| Velocidade de cópia/exclusão | Rápida (sem limite), Normal (60 MB/s) ou Lenta (15 MB/s). Em Normal e Lenta, a exclusão é limitada a 500 e 100 arquivos por segundo, respectivamente |
| Linha de estimativa | Mostra quanto levaria a cópia no próximo logout e, se as cópias forem excluídas, também a limpeza após o login |
| Iniciar com o computador | Inicia o CCDC quando você faz login no computador |
| Language | Escolhe o idioma de exibição. Este item sempre aparece em inglês; cada idioma do submenu está escrito nesse idioma |
| Ver log | Abre o `watch.log` do CCDC |
| Dar nome às contas (ccdc.json) | Abre o arquivo de configurações para dar nomes legíveis às contas |
| Sair | Remove o ícone; se houver uma cópia em andamento, aguarda até que ela termine |

São exibidas notificações quando a cópia começa (com o tempo estimado), quando termina, quando um login é detectado e quando a limpeza termina. Uma velocidade mais baixa faz a cópia demorar mais, então a próxima pessoa deve esperar pela notificação "Cópia concluída", ou aproximadamente o tempo estimado, antes de fazer login.

A estimativa é calculada a partir do estado atual das pastas e das velocidades de cópia e exclusão medidas neste computador (guardadas em `state.json`, na pasta do CCDC).

Se o ícone não aparecer no Windows 11, procure-o entre os ícones ocultos (`^`) na barra de tarefas. Para mantê-lo sempre visível, ative o CCDC em Configurações > Personalização > Barra de tarefas > Outros ícones da bandeja do sistema.

O CCDC é um único programa nativo de cerca de 1 MB. Enquanto espera, ele lê a cada meio segundo só o que foi acrescentado ao log do Claude desde a última verificação, e usa por conta própria cerca de 4 MB de memória.

## Configurações

As configurações ficam em `ccdc.json`, na pasta do CCDC:

| | Pasta do CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

O que você mudar no menu da bandeja é salvo nesse arquivo e vale a partir da próxima cópia ou limpeza, sem precisar reiniciar. O `watch.log` e o `state.json` ficam na mesma pasta.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Chave | Valores |
| --- | --- |
| `keepCopies` | `true` (padrão): mantém as cópias. `false`: exclui as cópias após o login |
| `speed` | `"fast"` (padrão), `"normal"`, `"gentle"` |
| `language` | Vazio para seguir o idioma do sistema, ou um código de idioma como `"ja"` para usar esse idioma |
| `startAtLogin` | Definido pela opção "Iniciar com o computador" no menu da bandeja |
| `aliases` | Nomes mostrados para as contas no menu da bandeja |
| `dataDir`, `appLog` | Só se o CCDC não encontrar sozinho a pasta de dados ou o log do Claude: o caminho completo de cada um |

## Idiomas

O menu da bandeja, as notificações e os avisos seguem o idioma de exibição do sistema, ou o idioma escolhido em **Language** no menu da bandeja. Idiomas disponíveis: inglês, coreano, japonês, chinês (simplificado e tradicional), espanhol, francês, alemão, italiano, português, russo, ucraniano, polonês, holandês, turco, vietnamita, indonésio, tailandês, árabe e hindi. Os demais idiomas aparecem em inglês.

Todos os textos da interface ficam em uma única planilha, [`locales.csv`](locales.csv), embutida no programa: **uma linha por idioma**, uma coluna por mensagem. Abra no Excel ou no Planilhas Google; para adicionar um idioma, adicione uma linha e [compile](#compilação) o CCDC.

- `lang`: uma tag de idioma (`sv`, `pt-BR`, `zh-Hant`, …). Primeiro é usada a linha que corresponde ao idioma e à região; se não houver, a que corresponde ao idioma.
- `name`: o nome do idioma escrito nesse idioma (`Svenska`); é o que o menu Language mostra.
- Escreva quebras de linha dentro de uma célula como `\n` e mantenha `{0}`, `{1}` (ali entram números ou nomes).
- Células vazias aparecem em inglês. Salve o arquivo como CSV UTF-8.

A saída da linha de comando e o `watch.log` ficam em inglês.

## Linha de comando

O mesmo programa também funciona em um terminal (no Windows, `ccdc.exe` em `%LOCALAPPDATA%\Programs\CCDC`; no macOS, `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` e `--app-log <path>` indicam manualmente a pasta de dados ou o log do Claude.

## Regras para duplicatas

- Quando uma sessão está em várias pastas, vence a cópia com o `lastActivityAt` mais recente. As cópias mantêm a data de modificação da origem, então cópias idênticas são reconhecidas pelo tamanho e pela data, sem precisar lê-las.
- Quando uma cópia mais nova substitui um arquivo, o arquivo antigo vai para `ccdc-backups/<time>/` na pasta de dados do Claude.
- Sessões com um marcador `deleted_<id>` não são copiadas, e as cópias delas em outras pastas são removidas.
- Uma conta/organização que faz login pela primeira vez ainda não tem pasta, então nada pôde ser copiado para lá no logout. Nesse caso, as sessões aparecem após o próximo login dessa conta ou ao reiniciar o aplicativo.

## Limitações

- O CCDC depende da estrutura interna de arquivos e do texto do log do aplicativo Claude para desktop; uma atualização do aplicativo pode mudar o comportamento do CCDC.
- Os arquivos de sessão guardam configurações de conectores (MCP) de cada conta, então alguns conectores podem não se conectar quando uma sessão é aberta por outra conta.
- As tarefas agendadas não são compartilhadas; cada pasta de conta/organização mantém as suas.
- O CCDC não cobre a troca de conta na CLI do Claude Code. A CLI já consegue abrir qualquer sessão em `~/.claude/projects` com `--resume`, seja qual for a conta.

## Compilação

O CCDC é escrito em Rust; [instale o Rust](https://rustup.rs) e depois:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

A versão de cada plataforma é definida em `[package.metadata.ccdc]` no `Cargo.toml`. O GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) compila as três a cada push e, para uma tag `v*`, anexa os arquivos a um rascunho de versão.

## Licença

[MIT](LICENSE)
