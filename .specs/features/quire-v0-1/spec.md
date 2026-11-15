# QUIRE v0.1 — Leitura de specs em vários repositórios e worktrees

**Idioma:** pt-BR

**Repositório:** `quire`: `crates/quire-core` (scan, parsers, índice), `apps/desktop` (UI React e shell Tauri), `fixtures/`
**Data:** 2026-11-15
**Status:** rascunho, aguardando aprovação do autor
**Complexidade:** Large. Verificação: `full` (acesso ao sistema de arquivos do usuário, watcher, CLI, empacotamento).

## Fontes

- **Ideação e MoSCoW:** `docs/ideation/moscow.md` (decisões fechadas, modelo de descoberta e deduplicação, riscos, fatiamento)
- **Formato das pastas de spec:** `docs/spec-format.md` (preset `exban`: cabeçalho em negrito, `tasks.md`, JSONL, IDs)
- **Design:** `docs/design/README.md`, `docs/design/tokens.css` e `docs/design/wireframes/*.html`
- **Convenções do projeto:** `AGENTS.md`

## Problema

Cada issue gera uma pasta de spec (`spec.md`, `design.md`, `tasks.md`, `summary.md`, `validation.md`, `gates.jsonl`, `lessons.jsonl`) dentro de um repositório, e a mesma spec pode existir em várias worktrees. Hoje essas pastas são abertas em clientes genéricos de markdown que não sabem o que é uma spec: não agrupam por projeto, não deduplicam cópias, não mostram progresso de tasks nem os logs de gates, e não reagem às mudanças que os agentes fazem nos arquivos.

## Objetivos

- Descobrir automaticamente as specs de vários repositórios e worktrees e listá-las num só lugar, deduplicadas e agrupadas por projeto.
- Ler cada spec com uma visão por arquivo (markdown, tasks por fase, gates e lessons) e um fallback para qualquer outro arquivo.
- Manter a leitura sempre atual enquanto agentes e editores alteram os arquivos.
- Abrir pelo terminal (`quire .`, `quire open <chave>`) e ser leve e rápido.

## Fora do escopo

Entram na v0.2 ou depois (ver `docs/ideation/moscow.md`):

- Edição de arquivos, busca full-text, command palette, checkbox clicável do `tasks.md`, criação de spec por template.
- `arquivo:linha` clicável, progresso ao vivo com alerta de divergência, journal com quick-add, diff entre instâncias, abrir no `$EDITOR`.
- Lint de estrutura, backlinks, histórico git por spec, export PDF/HTML.
- Kanban, leitura de dados do Jira por API, servidor MCP, sync em nuvem, mobile, chat de IA embutido.

## Premissas e perguntas em aberto

- **A1:** a UI nasce em pt-BR, como no wireframe, com as strings num catálogo único para permitir inglês depois.
- **A2:** a configuração do usuário fica em `config.toml` no diretório de configuração do sistema; o cache de índice no diretório de dados e pode ser apagado a qualquer momento.
- **A3:** "modificada mais recentemente" = maior `mtime` entre os arquivos primários da instância.
- **A4:** o histórico git é lido chamando o `git` do sistema (ver design, TD-03). Sem `git` instalado, o agrupamento cai para o nome da pasta e a v0.1 avisa.
- **P1 (aberta):** tamanho aceitável do bundle com Mermaid carregado sob demanda. Medir no spike (T2).
- **P2 (aberta):** comportamento exato do CLI e da janela no Windows nativo (console anexado ao binário GUI). Medir em T22; Linux é o alvo principal.

## Histórias de usuário

### P1: Descoberta de specs ⭐ MVP

**User Story**: Como dev com vários repos e worktrees, quero apontar pastas para o Quire varrer e ver todas as specs, para não precisar saber onde cada uma mora.

**Critérios de aceite**:

1. QUANDO o usuário configura um scan path ENTÃO o sistema DEVE varrê-lo recursivamente atrás de diretórios `.specs`, respeitando `.gitignore` e ignorando `node_modules`, `target`, `dist` e `.git`. <!-- DSC-01 -->
2. QUANDO encontra `.specs/features/<pasta>/` com ao menos um arquivo primário do preset ativo ENTÃO o sistema DEVE registrar a pasta como uma spec. <!-- DSC-02 -->
3. QUANDO um repo encontrado tem worktrees fora dos scan paths (`git worktree list`) ENTÃO o sistema DEVE varrê-las também. <!-- DSC-03 -->
4. QUANDO o scan roda ENTÃO o sistema NÃO DEVE bloquear a interface: as specs aparecem à medida que são achadas e um indicador mostra o andamento. <!-- DSC-04 -->
5. QUANDO um scan path não existe ou não é legível ENTÃO o sistema DEVE sinalizar esse path com o motivo e continuar com os demais. <!-- DSC-05 -->

### P1: Projetos e deduplicação ⭐ MVP

**User Story**: Como dev, quero ver cada spec uma vez só, agrupada pelo projeto, mesmo que ela exista em várias worktrees.

**Critérios de aceite**:

1. QUANDO duas pastas pertencem ao mesmo repositório git (mesmo git common dir) ENTÃO o sistema DEVE agrupá-las no mesmo projeto, cujo nome vem do `origin` (sem `.git`), com fallback para o nome da pasta do repo principal e alias configurável. <!-- DUP-01 -->
2. QUANDO pastas de spec têm o mesmo nome dentro do mesmo projeto ENTÃO o sistema DEVE tratá-las como instâncias da mesma spec. <!-- DUP-02 -->
3. QUANDO todas as instâncias têm conteúdo idêntico nos arquivos primários ENTÃO o sistema DEVE mostrar uma entrada sem indicação de versões; QUANDO divergem ENTÃO DEVE mostrar o badge "N versões". <!-- DUP-03 -->
4. QUANDO uma spec tem várias instâncias ENTÃO o sistema DEVE abrir por padrão a modificada mais recentemente (A3) e DEVE permitir trocar de instância, valendo a escolha até o fim da sessão. <!-- DUP-04 -->
5. QUANDO o usuário abre o seletor ENTÃO o sistema DEVE listar, por instância: branch, tipo (worktree ou checkout), caminho, há quanto tempo foi modificada e estado (aberta, diverge, idêntica). <!-- DUP-05 -->
6. QUANDO uma spec está aberta ENTÃO o cabeçalho DEVE mostrar a branch e a worktree da instância exibida. <!-- DUP-06 -->

### P1: Leitura da spec ⭐ MVP

**User Story**: Como dev, quero abrir uma spec e ler seus arquivos com abas, outline e markdown bem renderizado, sem abrir outro cliente.

**Critérios de aceite**:

1. QUANDO uma spec é aberta ENTÃO o sistema DEVE mostrar uma aba por arquivo primário existente (`spec`, `design`, `tasks`, `summary`, `validation`, `gates`, `lessons`) e uma aba "outros" com os demais arquivos da pasta. <!-- VIW-01 -->
2. QUANDO um arquivo markdown é exibido ENTÃO o sistema DEVE renderizar GFM (tabelas, listas de tarefas, links, código com realce de sintaxe). <!-- VIW-02 -->
3. QUANDO há bloco `mermaid` ENTÃO o sistema DEVE renderizar o diagrama; QUANDO a sintaxe é inválida ENTÃO DEVE mostrar o código com a mensagem de erro, sem quebrar o resto da página. <!-- VIW-03 -->
4. QUANDO há bloco de código ENTÃO o sistema DEVE usar fonte monoespaçada e preservar o alinhamento (diagramas ASCII). <!-- VIW-04 -->
5. QUANDO um arquivo markdown é exibido ENTÃO o sistema DEVE mostrar um outline com os cabeçalhos, destacar a seção visível e rolar até a seção ao clicar. <!-- VIW-05 -->
6. QUANDO o usuário recolhe um cabeçalho ENTÃO o sistema DEVE ocultar o conteúdo da seção até ser expandida de novo. <!-- VIW-06 -->
7. QUANDO o `spec.md` ou o `summary.md` tem linhas `**Chave:** valor` no cabeçalho ENTÃO o sistema DEVE exibi-las como chips na faixa de metadados e DEVE abrir a URL da issue no navegador ao clicar. <!-- VIW-07 -->
8. QUANDO o arquivo é outro `.md` ENTÃO renderizar como markdown; `.json` ENTÃO texto com realce; demais textos ENTÃO texto puro; binário ou ilegível ENTÃO mensagem informativa. <!-- VIW-08 -->

### P1: Tasks por fase ⭐ MVP

**User Story**: Como dev, quero ver o `tasks.md` organizado por fase, com progresso e dependências, para saber onde a implementação está.

**Critérios de aceite**:

1. QUANDO a aba `tasks` é exibida ENTÃO o sistema DEVE listar as fases com título, contagem concluídas/total, barra de progresso e estado (completa, em andamento, pendente). <!-- TSK-01 -->
2. QUANDO uma fase tem diagrama de dependência ENTÃO o sistema DEVE exibi-lo como chips ordenados com setas, coloridos por estado. <!-- TSK-02 -->
3. QUANDO uma fase está expandida ENTÃO cada tarefa DEVE mostrar estado, id, título, IDs de requisito e hash curto do commit, quando houver. <!-- TSK-03 -->
4. QUANDO o usuário seleciona uma tarefa ENTÃO o painel lateral DEVE mostrar os campos do detalhamento (onde, depende de, requisito, pronto quando, testes, gate). <!-- TSK-04 -->
5. QUANDO a v0.1 exibe tarefas ENTÃO o estado DEVE ser somente leitura, com aviso de que a marcação é feita pelo agente. <!-- TSK-05 -->

### P1: Gates e lessons ⭐ MVP

**User Story**: Como dev, quero ver o histórico de gates e as lições registradas por agentes, sem abrir JSONL cru.

**Critérios de aceite**:

1. QUANDO a aba `gates` é exibida ENTÃO o sistema DEVE mostrar uma tabela do mais recente ao mais antigo (quando, nível, status, head, duração, comando) com filtros por nível e status e a contagem de execuções, passes e falhas. <!-- LOG-01 -->
2. QUANDO o usuário expande uma execução com falha ENTÃO o sistema DEVE mostrar o `tail`, o código de saída e o diretório. <!-- LOG-02 -->
3. QUANDO a aba `lessons` é exibida ENTÃO o sistema DEVE listar cada lição com sinal, escopo, texto e fonte. <!-- LOG-03 -->
4. QUANDO uma linha do JSONL é inválida ENTÃO o sistema DEVE ignorá-la e mostrar quantas linhas foram puladas. <!-- LOG-04 -->

### P1: Atualização ao vivo ⭐ MVP

**User Story**: Como dev, quero que o Quire acompanhe o que agentes e outros editores escrevem nos arquivos, sem eu recarregar.

**Critérios de aceite**:

1. QUANDO um arquivo de spec muda, ou uma pasta de spec é criada ou removida ENTÃO a lista e a visão aberta DEVEM refletir a mudança em até 2 s em sistemas com eventos de arquivo. <!-- WAT-01 -->
2. QUANDO a visão aberta é recarregada ENTÃO o sistema DEVE manter a aba, a seção e a posição de rolagem. <!-- WAT-02 -->
3. QUANDO o sistema de arquivos não entrega eventos (ex.: WSL em caminho montado do Windows) ENTÃO o sistema DEVE usar polling, com intervalo configurável (padrão 3 s). <!-- WAT-03 -->
4. QUANDO a instância aberta é removida ENTÃO o sistema DEVE passar para outra instância da mesma spec, com aviso, ou voltar à lista se não houver outra. <!-- WAT-04 -->

### P1: CLI ⭐ MVP

**User Story**: Como dev, quero abrir specs direto do terminal.

**Critérios de aceite**:

1. QUANDO `quire .` roda dentro de uma worktree ENTÃO o sistema DEVE abrir a spec dessa worktree se houver uma única, ou a lista do projeto com as da worktree em destaque. <!-- CLI-01 -->
2. QUANDO `quire open <chave>` roda ENTÃO o sistema DEVE abrir a spec cuja pasta começa com a chave; SE houver mais de uma (projetos diferentes) ENTÃO DEVE mostrar um seletor. <!-- CLI-02 -->
3. QUANDO `quire open <caminho>` aponta para uma pasta de spec ou para um arquivo dela ENTÃO o sistema DEVE abri-la. <!-- CLI-03 -->
4. QUANDO o app já está aberto e o CLI é chamado de novo ENTÃO o sistema DEVE focar a janela existente e navegar, sem abrir segunda instância. <!-- CLI-04 -->
5. QUANDO `quire --help` ou `quire --version` roda ENTÃO o sistema DEVE responder no terminal sem abrir a janela. <!-- CLI-05 -->

### P1: Índice e presets ⭐ MVP

**User Story**: Como dev, quero abrir rápido mesmo com muitos repos, e usar a convenção da minha equipe sem o app depender dela.

**Critérios de aceite**:

1. QUANDO o app inicia ENTÃO o sistema DEVE usar o cache de índice (SQLite) no diretório de dados; apagar o cache DEVE levar a uma reconstrução completa sem perda de informação. <!-- IDX-01 -->
2. QUANDO o scan reconcilia o cache com o disco ENTÃO o sistema DEVE reprocessar apenas arquivos cujo `mtime` ou tamanho mudou. <!-- IDX-02 -->
3. QUANDO há cache e cerca de 300 specs ENTÃO a lista DEVE aparecer em menos de 1 s, antes de a reconciliação terminar. <!-- IDX-03 -->
4. QUANDO um projeto não tem preset definido ENTÃO o sistema DEVE usar o preset `generic` (qualquer `.md` e `.jsonl`) e DEVE sugerir o `exban` ao detectar a estrutura de arquivos dele. <!-- PST-01 -->
5. QUANDO o usuário escolhe um preset por projeto ENTÃO o sistema DEVE aplicá-lo: arquivos primários, parser de cabeçalho, gramática do `tasks.md` e padrão de IDs. <!-- PST-02 -->
6. QUANDO um arquivo não segue o formato do preset ENTÃO o sistema DEVE exibi-lo como markdown ou texto puro, sem erro. <!-- PST-03 -->

### P2: Projeto open source e distribuição

**User Story**: Como mantenedor, quero um repositório público saudável e builds reproduzíveis.

**Critérios de aceite**:

1. O repositório DEVE ter README, CONTRIBUTING e licença MIT. <!-- OSS-01 -->
2. QUANDO há push ou PR ENTÃO o CI DEVE rodar `fmt`, `clippy`, `typecheck` e testes em Linux e Windows. <!-- OSS-02 -->
3. O repositório DEVE conter apenas fixtures sintéticas; nenhuma spec, log ou caminho real de empresa. <!-- OSS-03 -->
4. QUANDO uma tag de release é criada ENTÃO o CI DEVE gerar artefatos para Linux (AppImage e deb) e Windows (msi). <!-- OSS-04 -->

## Casos de borda

- Duas worktrees com a mesma spec e `mtime` idêntico: desempate determinístico pelo caminho em ordem alfabética.
- Pasta `.specs` dentro de `node_modules` ou de um submódulo: ignorada (DSC-01); submódulo com `.specs` próprio é tratado como projeto separado.
- Repo sem `origin`: nome do projeto vem da pasta do repo principal (DUP-01).
- Spec parcial (sem `validation.md` ou `lessons.jsonl`): só as abas existentes aparecem (VIW-01).
- Arquivo muito grande (> 2 MB): carregado com aviso e sem realce de sintaxe.
- JSONL com linha truncada no fim (agente escrevendo): linha ignorada e relida na próxima mudança (LOG-04, WAT-01).
- Arquivo substituído por rename atômico (editores): tratado como mudança, não como remoção.
- Dois scan paths que se sobrepõem: cada pasta de spec é registrada uma vez.
- Caminho com acentos, espaços ou em unidade do Windows: tratado sem erro.

## Rastreabilidade de requisitos

| Requisito | Tarefas |
| --- | --- |
| DSC-01, DSC-02 | T6 |
| DSC-03 | T7 |
| DSC-04 | T12, T15 |
| DSC-05 | T6, T23 |
| DUP-01 | T7 |
| DUP-02, DUP-03 | T8 |
| DUP-04, DUP-05, DUP-06 | T8, T20 |
| VIW-01, VIW-08 | T17 |
| VIW-02 a VIW-06 | T16 |
| VIW-07 | T10, T17 |
| TSK-01 a TSK-05 | T10, T18 |
| LOG-01 a LOG-04 | T11, T19 |
| WAT-01, WAT-02, WAT-04 | T21 |
| WAT-03 | T3, T21 |
| CLI-01 a CLI-05 | T22 |
| IDX-01 a IDX-03 | T12 |
| PST-01 a PST-03 | T9, T10, T23 |
| OSS-01 | T24 |
| OSS-02 | T1, T4 |
| OSS-03 | T5 |
| OSS-04 | T24 |

## Critérios de sucesso

- Em um workspace real com vários repos e worktrees, todas as specs aparecem deduplicadas e agrupadas por projeto.
- Ler uma spec completa (7 arquivos) no Quire dispensa abrir outro cliente de markdown.
- Uma mudança feita por um agente num arquivo aparece na tela sem ação do usuário.
- `quire .` e `quire open <chave>` levam à spec certa.
- A lista de ~300 specs abre em menos de 1 s com cache.
