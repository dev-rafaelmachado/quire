# Quire — ideação e MoSCoW

Resumo das decisões da fase de ideação. A v0.1 está especificada em `.specs/features/quire-v0-1/`.

## Decisões fechadas

- **Unidade do app = pasta da spec**, não o arquivo `.md`: status, progresso e vínculo com a issue vivem na spec inteira.
- **Descoberta automática:** o app varre pastas configuradas atrás de `.specs` em qualquer profundidade e lista as specs deduplicadas, agrupadas por projeto (repositório git).
- **Núcleo agnóstico + preset:** o núcleo lê qualquer `.md` e `.jsonl`; o preset `exban` (spec, design, tasks, summary, validation, gates, lessons) vem no MVP como opção por projeto.
- **Checkbox do `tasks.md`:** opt-in, desligado por padrão e configurável por projeto. No fluxo de origem quem marca é o agente, depois dos gates, com o hash do commit.
- **`gates.jsonl` e `lessons.jsonl` são somente leitura:** são escritos por agente.
- **Plataforma:** Tauri 2. Linux é o alvo principal e Windows nativo o segundo, com CI nos dois. No interim o app pode rodar dentro do WSL (WSLg).
- **Open source desde o dia 1:** MIT, README, CONTRIBUTING e fixtures sintéticas.
- **Formato estável:** o `tasks.md` e o journal do `summary.md` são tratados como estáveis. A URL do Jira vem do `spec.md`, com config por projeto como fallback.
- **Nome:** Quire (CLI `quire`).
- **Could escolhidos para a primeira versão:** lint de estrutura, backlinks entre specs, histórico/diff via git por spec e export PDF/HTML.

## Modelo de descoberta e deduplicação

Hierarquia: **Projeto → Spec → Instâncias (worktrees/branches) → Arquivos**.

- **Scan paths:** o usuário configura onde varrer. O walker respeita `.gitignore`, ignora `node_modules`, `target`, `dist` e procura `.specs/`. Para cada repo achado roda `git worktree list`, porque worktrees podem estar fora do path varrido.
- **Identidade do projeto = repositório git** (git common dir), não o nome da pasta. O nome exibido vem do `origin`, com alias configurável.
- **Identidade da spec = (projeto, nome da pasta da spec).** Conteúdo idêntico em várias worktrees vira uma entrada. Conteúdo divergente vira uma entrada com badge "N versões" e seletor de instância.
- **Instância padrão = a modificada mais recentemente.** Dentro de uma worktree, `quire .` abre a instância dela. O cabeçalho sempre mostra branch e worktree.
- **Chave de issue em mais de um projeto:** `quire open <chave>` abre um picker quando houver ambiguidade.
- **Ciclo de vida:** worktree criada ou removida atualiza a lista; se a instância aberta sumir, fallback sem perder o buffer editado.
- **Índice em cache** (SQLite) com varredura incremental, em background e progressiva.

## MoSCoW v1

### Must have

- Scan paths com auto-descoberta de `.specs`; config por projeto como override.
- Agrupamento por projeto e deduplicação com seletor de instância.
- Visão unificada da spec: abas dos arquivos primários e fallback para qualquer outro arquivo.
- Parser de cabeçalho em negrito (status, issue, idioma), configurável.
- Visão do `tasks.md` por fase, com dependências, status e detalhe expansível.
- Visão somente leitura de `gates.jsonl` e `lessons.jsonl`.
- Render de markdown: GFM, tabelas, highlight, Mermaid, code blocks monoespaçados.
- Outline lateral e seções colapsáveis.
- Edição com preview (CodeMirror 6), salvando direto no arquivo.
- File watcher que reage a mudanças externas sem perder o buffer.
- CLI sensível ao diretório atual: `quire .`, `quire open <chave|path>`, single-instance, picker em ambiguidade.
- Cache de índice (SQLite) com scan incremental.
- Progresso x/y do `tasks.md`, com checkbox clicável opt-in por projeto.
- Núcleo genérico + preset `exban` selecionável por projeto.
- Linux como alvo principal, Windows nativo como segundo, CI nos dois.
- Licença, README, CONTRIBUTING e fixtures sintéticas desde o dia 1.

### Should have

- Busca full-text em todos os projetos.
- Progresso e status por spec na listagem.
- Progresso ao vivo do `tasks.md`, com alerta se divergir da tabela do `summary.md`.
- Visão rica de gates por fase e nível, com o `head` linkando ao commit.
- Journal como timeline, com quick-add.
- `arquivo:linha` clicável, abrindo no editor na linha.
- Chave do Jira clicável (URL base configurável por projeto).
- Diff entre instâncias divergentes e badge de branch/worktree na lista.
- Command palette (Ctrl+K), abrir no `$EDITOR`/terminal, criar spec a partir de template.
- Indicadores de git por projeto, copiar contexto da spec para um agente.
- Tema claro/escuro, recentes, instaladores (Windows e Linux) e CLI no PATH.

### Could have (primeira versão)

- **Lint de estrutura:** ID sem task, task sem "Pronto quando", dependência inexistente, progresso do summary diferente do tasks, ID sem evidência no validation. Regras configuráveis.
- **Backlinks por ID:** `CCR-03` leva à spec, às tasks que o cobrem e à evidência. Depois: chave da issue, links relativos, `AD-xxx` para `STATE.md`, inclusive entre projetos.
- **Histórico e diff via git por spec**, usando os hashes das tasks.
- **Export** para PDF ou HTML estático (Mermaid pré-renderizado no PDF).

### Won't have (v1)

- Backlog: kanban por status, leitura de dados do Jira via API, servidor MCP, botão "implementar task", tray e atalho global.
- Fora de escopo: sync em nuvem ou colaboração em tempo real, app de notas genérico, editor WYSIWYG, chat de IA embutido, escrita no Jira, mobile.
- Edição de `gates.jsonl` e `lessons.jsonl`.

## Riscos

| Risco | Impacto | Mitigação |
| --- | --- | --- |
| Agentes em WSL e app no Windows | Watcher não recebe eventos via `\\wsl$`; scan lento | Priorizar Linux; no interim rodar o app dentro do WSL (WSLg) com polling como fallback, validado no spike |
| Tauri no Linux usa WebKitGTK | Quirks de renderização e performance (Mermaid, fontes, scroll) | Spike cedo e CI nos dois sistemas |
| Escrita concorrente com agentes | Gravar na cópia velha ou sobrescrever o agente | Watcher preserva buffer sujo, branch/worktree visíveis, checkbox opt-in, gates e lessons read-only |
| Formato acoplado a um fluxo | Parsers frágeis se a convenção mudar | Núcleo agnóstico + preset; parsers tolerantes com fallback para markdown puro |
| Custo do scan | Startup lento | Walker com `.gitignore`, cache SQLite incremental, UI progressiva |
| Worktrees fora do path varrido | Specs não aparecem | `git worktree list` em cada repo |
| "Por que não Obsidian + plugin?" | Um vault cobriria ~60% | Diferencial: spec como unidade, descoberta multi-worktree, CLI e leveza |

## Fatiamento

| Release | Foco | Critério de pronto |
| --- | --- | --- |
| v0.1 | Leitura: scan, projetos, dedup, seletor de instância, abas, fallback, outline, Mermaid, watcher, CLI, cache, CI, licença, fixtures | O time lê specs de vários repos e worktrees sem abrir outro cliente |
| v0.2 | Escrita e busca: edição com preview, busca full-text, command palette, progresso x/y, checkbox opt-in | Editar uma spec do começo ao fim só no app |
| v0.3+ | Visões ricas e Could | Seguir um critério da spec até a evidência sem sair do app |
