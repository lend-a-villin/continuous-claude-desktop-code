# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | **Português** | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | [Bahasa Indonesia](README.id.md) | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Um aplicativo de bandeja para Windows que mantém as sessões locais do **Code** do aplicativo Claude para desktop disponíveis para qualquer conta do Claude que fizer login no PC.

Não oficial; não é afiliado nem endossado pela Anthropic.

## Para quem é

O CCDC foi feito para **locais de trabalho que funcionam 24 horas em três turnos**. Em cada turno, uma pessoa diferente ocupa o mesmo posto de trabalho (um PC, um usuário do Windows) e faz login no aplicativo Claude para desktop com a própria conta do Claude. Com o CCDC, quem entra no turno seguinte faz login e continua o trabalho no Code que o turno anterior deixou.

Por isso, todas as sessões do Code no PC ficam visíveis para qualquer pessoa que fizer login nele. Não use o CCDC onde as pessoas não devam ver as sessões umas das outras. Se você usa uma conta da empresa (organização), consulte também a política da empresa.

## O problema que ele resolve

O aplicativo guarda a **lista** de sessões do Code em uma pasta separada para cada conta e organização:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

As **conversas** em si ficam em `~/.claude/projects\...\<cliSessionId>.jsonl`, que todas as contas compartilham. Por isso, depois que alguém faz login com outra conta, a lista de sessões do Code parece vazia, embora as conversas continuem no disco.

## Como funciona

O aplicativo lê a pasta de sessões **somente ao ser iniciado e quando alguém faz login** (duas vezes, em cerca de dois segundos). Não há como fazê-lo reler só a lista mais tarde. O log do próprio aplicativo, `%LOCALAPPDATA%\Claude\Logs\main.log`, registra tanto os logouts quanto os logins:

- logout: `[account] Login-state transition (loggedOut: false → true ...)`
- login: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

O CCDC acompanha esse log e age assim:

1. **Logout** → copia a versão mais recente de cada sessão para todas as pastas de conta/organização, de modo que a próxima conta a fazer login já as encontre na própria pasta. Só são copiadas as sessões que faltam ou estão desatualizadas em cada pasta.
2. **Login** → por padrão, as cópias ficam onde estão. Se a opção "Excluir as cópias após o login" estiver ativada, o CCDC espera o aplicativo ler a lista (6 segundos após a última linha de login) e depois remove as cópias de todas as pastas, exceto a da conta conectada.

Uma sessão excluída no aplicativo também é removida das outras pastas, para não reaparecer em outra conta.

Antes de copiar, o CCDC verifica o espaço livre em disco. Se não houver espaço para as cópias mais 512 MB, ele não copia nada e mostra um aviso.

### Por que as cópias são mantidas por padrão

Manter as cópias é a opção que menos desgasta o SSD:

- **Cópias mantidas (padrão)**: o primeiro logout copia todas as sessões uma vez. Depois disso, cada logout grava só as sessões que mudaram desde o anterior, em geral alguns arquivos e alguns MB, e termina quase imediatamente.
- **Cópias excluídas após o login**: cada login apaga as cópias, então cada logout precisa gravar todas as sessões de novo. Com cerca de 300 sessões e 10 pastas de conta/organização, são aproximadamente 600 MB gravados a cada troca de turno, mais a exclusão de uns 3.000 arquivos depois.

O custo de mantê-las é o espaço em disco: as cópias continuam no disco e ocupam cerca de (tamanho total dos arquivos de sessão × número de pastas de conta/organização).

### Por que não usar junções nem links físicos

Fazer todas as pastas de conta apontarem para uma única pasta real evitaria cópias, mas o aplicativo Claude bloqueia de propósito as duas formas de fazer isso:

- **Junções e links simbólicos**: antes de cada salvamento, o aplicativo verifica se a pasta de sessões é uma pasta real e se recusa a salvar se for um link (`Refusing non-directory at private dir path (symlink/file plant)`). Em um teste com o aplicativo real, a lista podia ser lida por meio de uma junção, mas todos os salvamentos de sessões e de tarefas agendadas falharam, inclusive os de sessões novas.
- **Links físicos**: o aplicativo considera inseguro um arquivo com mais de um link. Ele se recusa a gravar nesses arquivos e avisa ao lê-los. Além disso, ele salva gravando um arquivo temporário e renomeando-o, o que quebraria o link a cada salvamento.

Essas verificações protegem contra arquivos ou pastas inseridos de fora, então o CCDC não as contorna. Ele copia arquivos comuns, do mesmo tipo que o próprio aplicativo grava.

## Requisitos

- Windows 10 ou 11, com o aplicativo Claude para desktop (instalação MSIX)
- [Node.js](https://nodejs.org) 18 ou mais recente

## Instalação

Baixe o instalador em [Releases](../../releases), descompacte e execute `setup.cmd`. Não são necessários direitos de administrador.

- Instala em `%LOCALAPPDATA%\Programs\CCDC`.
- Cria um atalho `Continuous Claude Desktop Code` na área de trabalho e no menu Iniciar.
- Aparece em Configurações > Aplicativos > Aplicativos instalados como "Continuous Claude Desktop Code (CCDC)"; desinstale-o por lá.
- Inicia na bandeja logo após a instalação.
- Para que ele inicie quando você fizer login no Windows, instale com `setup.cmd /startup`; para desativar isso, use `setup.cmd /nostartup`. Sem nenhuma das duas opções, reinstalar mantém a configuração anterior.
- Reinstalar encerra o CCDC, se estiver em execução, e substitui os arquivos; suas configurações (`ccdc.json`) são mantidas.
- Desinstalar deixa os arquivos de sessão do Claude e a pasta de backup do CCDC como estão.

Para executar direto deste repositório, use `ccdc-tray.vbs` ou a linha de comando abaixo.

## Aplicativo de bandeja

Um **C** laranja aparece na área de notificação (cinza enquanto a sincronização automática está desativada). Clique nele para abrir o menu:

| Item do menu | O que faz |
| --- | --- |
| Linha de status | Mostra se o CCDC está monitorando logins e logouts |
| Sincronização automática (detectar login/logout) | Ativa ou desativa a cópia e a limpeza |
| Mover sessões para uma conta agora | Reúne todas as sessões na pasta de uma conta; pergunta antes se essa conta não estiver conectada |
| Excluir as cópias após o login | Desativado por padrão (as cópias são mantidas; veja acima) |
| Velocidade de cópia/exclusão | Rápida (sem limite), Normal (60 MB/s) ou Lenta (15 MB/s). Em Normal e Lenta, a exclusão é limitada a 500 e 100 arquivos por segundo, respectivamente |
| Linha de estimativa | Mostra quanto levaria a cópia no próximo logout e, se as cópias forem excluídas, também a limpeza após o login |
| Language | Escolhe o idioma de exibição. Este item sempre aparece em inglês; cada idioma do submenu está escrito nesse idioma |
| Ver log | Abre `%LOCALAPPDATA%\CCDC\watch.log` |
| Dar nome às contas (ccdc.json) | Abre o arquivo de configurações para dar nomes legíveis às contas |
| Sair | Remove o ícone; se houver uma cópia em andamento, aguarda até que ela termine |

São exibidas notificações quando a cópia começa (com o tempo estimado), quando termina, quando um login é detectado e quando a limpeza termina. Uma velocidade mais baixa faz a cópia demorar mais, então a próxima pessoa deve esperar pela notificação "Cópia concluída", ou aproximadamente o tempo estimado, antes de fazer login.

A estimativa é calculada a partir do estado atual das pastas e das velocidades de cópia e exclusão medidas neste PC (guardadas em `%LOCALAPPDATA%\CCDC\state.json`).

Se o ícone não aparecer no Windows 11, procure-o entre os ícones ocultos (`^`) na barra de tarefas. Para mantê-lo sempre visível, ative o Windows PowerShell em Configurações > Personalização > Barra de tarefas > Outros ícones da bandeja do sistema.

### Uso de memória

O aplicativo de bandeja monitora o log do Claude por conta própria, lendo só o que foi acrescentado desde a última verificação. O Node.js só roda enquanto copia ou limpa e é encerrado depois, então, entre as trocas de turno, apenas o aplicativo de bandeja fica em execução. Ele devolve a memória não utilizada ao Windows cerca de uma vez por minuto enquanto está ocioso (uns 15 MB em uso).

## Configurações

As configurações ficam em `ccdc.json`, na pasta do programa. O que você mudar no menu da bandeja é salvo nesse arquivo e vale a partir da próxima cópia ou limpeza, sem precisar reiniciar.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Chave | Valores |
| --- | --- |
| `keepCopies` | `true` (padrão): mantém as cópias. `false`: exclui as cópias após o login |
| `speed` | `"fast"` (padrão), `"normal"`, `"gentle"` |
| `language` | Vazio para seguir o idioma do Windows, ou um código de idioma como `"ja"` para usar esse idioma |
| `aliases` | Nomes mostrados para as contas no menu da bandeja |

## Idiomas

O menu da bandeja, as notificações, os avisos e as caixas de diálogo de instalação e desinstalação seguem o idioma de exibição do Windows, ou o idioma escolhido em **Language** no menu da bandeja. Idiomas disponíveis: inglês, coreano, japonês, chinês (simplificado e tradicional), espanhol, francês, alemão, italiano, português, russo, ucraniano, polonês, holandês, turco, vietnamita, indonésio, tailandês, árabe e hindi. Os demais idiomas aparecem em inglês.

Todos os textos da interface ficam em uma única planilha, [`locales.csv`](locales.csv): **uma linha por idioma**, uma coluna por mensagem. Abra no Excel ou no Planilhas Google; para adicionar um idioma, adicione uma linha.

- `lang`: um nome de cultura do Windows (`sv`, `pt-BR`, `zh-Hant`, …). Primeiro é usada a linha que corresponde ao idioma e à região; se não houver, a que corresponde ao idioma.
- `name`: o nome do idioma escrito nesse idioma (`Svenska`); é o que o menu Language mostra.
- Escreva quebras de linha dentro de uma célula como `\n` e mantenha `{0}`, `{1}` (ali entram números ou nomes).
- Células vazias aparecem em inglês. Salve o arquivo como CSV UTF-8.

A saída da linha de comando e o `watch.log` ficam em inglês porque o aplicativo de bandeja os lê.

## Linha de comando

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Quando necessário, o aplicativo de bandeja executa estas tarefas pontuais: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>` e `node ccdc.mjs startup` (para se pôr em dia com o estado atual do aplicativo Claude quando o aplicativo de bandeja é iniciado ou a sincronização automática é reativada).

## Regras para duplicatas

- Quando uma sessão está em várias pastas, vence a cópia com o `lastActivityAt` mais recente. As cópias mantêm a data de modificação da origem, então cópias idênticas são reconhecidas pelo tamanho e pela data, sem precisar lê-las.
- Quando uma cópia mais nova substitui um arquivo, o arquivo antigo vai para `ccdc-backups\<hora>\` na pasta de dados do aplicativo.
- Sessões com um marcador `deleted_<id>` não são copiadas, e as cópias delas em outras pastas são removidas.
- Uma conta/organização que faz login pela primeira vez ainda não tem pasta, então nada pôde ser copiado para lá no logout. Nesse caso, as sessões aparecem após o próximo login dessa conta ou ao reiniciar o aplicativo.

## Limitações

- O CCDC depende da estrutura interna de arquivos e do texto do log do aplicativo Claude para desktop; uma atualização do aplicativo pode mudar o comportamento do CCDC.
- Os arquivos de sessão guardam configurações de conectores (MCP) de cada conta, então alguns conectores podem não se conectar quando uma sessão é aberta por outra conta.
- As tarefas agendadas não são compartilhadas; cada pasta de conta/organização mantém as suas.
- O CCDC não cobre a troca de conta na CLI do Claude Code. A CLI já consegue abrir qualquer sessão em `~/.claude/projects` com `--resume`, seja qual for a conta.

## Licença

[MIT](LICENSE)
