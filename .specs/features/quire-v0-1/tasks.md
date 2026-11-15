# QUIRE v0.1 — Tarefas

## Execução

- Repositório: quire
- Base: main
- Verificação: full
- Paralelo: não
- E2E: manual (roteiro em T24)
- Idioma: pt-BR

## Execution Protocol (MANDATORY -- do not skip)

Siga o `AGENTS.md`. Implemente **uma tarefa por vez, na ordem das fases**. Antes de começar uma tarefa, releia os critérios de aceite dos IDs em `Requirement`. Rode o gate indicado; só depois marque `- [x]` e acrescente o hash curto do commit ao fim da linha. Se algo na spec ou no design não funcionar, **pare e registre no `summary.md` (Journal)**; não ajuste o teste ao resultado. Commits em Conventional Commits, sem `Co-Authored-By`. Nunca faça push, abra PR ou crie tag sem o autor pedir.

---

**Spec**: `.specs/features/quire-v0-1/spec.md`
**Design**: `.specs/features/quire-v0-1/design.md`
**Status**: rascunho, aguardando aprovação do autor

**Condições obrigatórias da execução:**

1. O spike (Fase 0) vem antes de qualquer feature. Os resultados de T2 e T3 podem mudar o design (TD-05, virtualização do render); registre-os no `summary.md` antes de seguir.
2. Fixtures são sintéticas (T5). Nenhum dado real de empresa entra no repositório.
3. `quire-core` não depende de Tauri.

## Comandos de gate

- **fase:** `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && pnpm -r typecheck && pnpm -r test`
- **final:** o gate de fase + `pnpm --filter desktop build` + `cargo build --workspace --release`
- **e2e:** roteiro manual de T24 sobre `fixtures/workspace`

Confirmados em T1 (Linux, Rust 1.100, Node 26, pnpm 12), rodando da raiz do repositório:

- **setup:** `pnpm install` (o `@tauri-apps/cli` vem como devDependency de `apps/desktop`; não precisa de `cargo install tauri-cli`)
- **dev:** `pnpm --filter desktop tauri dev` (ou `pnpm dev` na raiz); Vite fixo na porta 1420
- `pnpm -r test` usa `vitest run --passWithNoTests`, então passa enquanto não houver testes de frontend
- No gate final, `pnpm --filter desktop build` precisa rodar **antes** de `cargo build --workspace --release`: o `tauri::generate_context!` embute `apps/desktop/dist` no binário de release
- Pré-requisitos no Linux (Arch/CachyOS): `webkit2gtk-4.1` e `rustup` com os componentes `clippy` e `rustfmt`

## Plano de execução

### Fase 0: Spike e fundação

```
T1 → T2 → T3 → T4
```

- [x] T1: Scaffold do monorepo (Cargo + pnpm, Tauri 2, Vite React TS estrito) (1d09a59)
- [x] T2: Spike do WebKitGTK: markdown grande, Mermaid e rolagem (cbb0913)
- [x] T3: Spike do watcher e do custo de scan (ext4, WSL e `\\wsl$`) (242c3d8)
- [ ] T4: CI no GitHub Actions (Linux e Windows)

### Fase 1: Núcleo (`quire-core`)

```
T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12
```

- [ ] T5: Fixtures sintéticas do workspace
- [ ] T6: Scanner de `.specs`
- [ ] T7: Identidade do projeto via git
- [ ] T8: Modelo de spec e instâncias com deduplicação
- [ ] T9: Framework de presets (`generic` e `exban`)
- [ ] T10: Parsers (cabeçalho, tasks, progresso, IDs)
- [ ] T11: Parsers JSONL tolerantes (gates e lessons)
- [ ] T12: Índice SQLite com scan incremental

### Fase 2: Shell e base da UI

```
T13 → T14 → T15
```

- [ ] T13: Comandos e eventos Tauri com tipos compartilhados
- [ ] T14: Tokens, tema, layout de três colunas e catálogo de strings
- [ ] T15: Sidebar de projetos e specs

### Fase 3: Visões

```
T16 → T17 → T18 → T19 → T20
```

- [ ] T16: Pipeline de markdown (GFM, código, Mermaid, outline, colapsar)
- [ ] T17: Abas, faixa de metadados e fallback de outros arquivos
- [ ] T18: Visão de tasks por fase
- [ ] T19: Visões de gates e lessons
- [ ] T20: Seletor de instâncias

### Fase 4: Ao vivo e CLI

```
T21 → T22
```

- [ ] T21: Watcher ligado ao índice e à UI
- [ ] T22: CLI com single-instance e seletor

### Fase 5: Fechamento da v0.1

```
T23 → T24
```

- [ ] T23: Preferências (scan paths, preset por projeto, tema)
- [ ] T24: Documentação de uso, empacotamento e roteiro E2E

## Detalhamento das tarefas

### T1: Scaffold do monorepo

**What**: Criar o workspace Cargo (`crates/quire-core`, `apps/desktop/src-tauri`) e o pnpm workspace (`apps/desktop` com Vite, React, TypeScript `strict`, Tailwind). Janela Tauri vazia abre. Registrar os comandos reais de gate em "Comandos de gate".
**Onde**: raiz, `crates/quire-core`, `apps/desktop`
**Depende de**: nenhuma
**Requirement**: OSS-02
**Pronto quando**:

- `pnpm install` e `pnpm --filter desktop tauri dev` abrem a janela no Linux
- `cargo test --workspace` e `pnpm -r typecheck` passam (mesmo sem testes ainda)
- `tsconfig` com `strict: true` e `noUncheckedIndexedAccess`

**Testes**: manual
**Gate**: fase

---

### T2: Spike do WebKitGTK

**What**: Página de teste que renderiza um markdown de ~40 KB com tabelas, blocos de código e Mermaid, com rolagem e colapso de seções. Medir tempo de render, fluidez da rolagem e tamanho do bundle (P1). Rodar em Linux nativo e no WSLg.
**Onde**: `apps/desktop/src/spike/` (descartável)
**Depende de**: T1
**Requirement**: VIW-02, VIW-03 (risco)
**Pronto quando**:

- Números e observações registrados no `summary.md` (Journal)
- Decisão registrada: render direto ou virtualizado por seção
- Spike removido ou isolado antes do commit final da tarefa

**Testes**: manual
**Gate**: fase

---

### T3: Spike do watcher e do custo de scan

**What**: Binário de teste no `quire-core` que usa `notify` em ext4, em `\\wsl$` e DrvFs, e mede a entrega de eventos; mede o tempo de varrer ~50 repos com `ignore`. Validar polling como fallback.
**Onde**: `crates/quire-core/examples/watch_spike.rs` (descartável)
**Depende de**: T1
**Requirement**: WAT-03 (risco), DSC-01
**Pronto quando**:

- Tabela de resultados (FS × eventos recebidos × latência) no `summary.md`
- Regra de detecção de "sem eventos" decidida (TD-05)
- Custo do polling com ~2000 arquivos medido

**Testes**: manual
**Gate**: fase

---

### T4: CI no GitHub Actions

**What**: Workflow com `fmt`, `clippy`, `typecheck` e testes em `ubuntu-latest` e `windows-latest`, com cache de Cargo e pnpm; dependências de sistema do Tauri no Linux.
**Onde**: `.github/workflows/ci.yml`
**Depende de**: T1
**Requirement**: OSS-02
**Pronto quando**:

- O workflow passa em Linux e Windows num PR de teste
- Falha de `clippy` ou de `typecheck` derruba o job

**Testes**: manual
**Gate**: fase

---

### T5: Fixtures sintéticas do workspace

**What**: `fixtures/workspace/` com 2 projetos git de teste (`acme_gateway`, `acme_billing`), worktrees (uma com conteúdo divergente), specs completas (7 arquivos) e parciais, `STATE.md`, e um script `scripts/make-fixtures.sh` que cria os repos git e worktrees (git não versiona repos aninhados).
**Onde**: `fixtures/`, `scripts/make-fixtures.sh`
**Depende de**: T1
**Requirement**: OSS-03
**Pronto quando**:

- O script gera o workspace do zero e é idempotente
- Nenhum nome, caminho ou texto real de empresa nos arquivos
- Cobre: spec idêntica em 2 worktrees, spec divergente, spec parcial, JSONL com linha inválida, repo sem `origin`

**Testes**: manual
**Gate**: fase

---

### T6: Scanner de `.specs`

**What**: Módulo `scan` com a crate `ignore`: varre scan paths, ignora `node_modules`, `target`, `dist`, `.git`, emite `Discovery` por canal; registra erro por path inválido.
**Onde**: `crates/quire-core/src/scan.rs`
**Depende de**: T5
**Requirement**: DSC-01, DSC-02, DSC-05
**Pronto quando**:

- Teste sobre as fixtures acha todas as specs esperadas
- `.specs` dentro de `node_modules` é ignorada
- Path inexistente gera erro por path e não derruba o scan

**Testes**: unit, integration
**Gate**: fase

---

### T7: Identidade do projeto via git

**What**: Módulo `git`: common dir, nome via `origin` (sem `.git`) com fallbacks, branch atual e `worktree list --porcelain`; timeout por chamada; descoberta de worktrees fora do scan path.
**Onde**: `crates/quire-core/src/git.rs`
**Depende de**: T6
**Requirement**: DSC-03, DUP-01
**Pronto quando**:

- Duas worktrees do mesmo repo resolvem para o mesmo projeto
- Repo sem `origin` usa o nome da pasta do repo principal
- Worktree fora dos scan paths é incluída (DSC-03)
- Sem `git` instalado: erro tipado e fallback, sem pânico

**Testes**: unit, integration
**Gate**: fase

---

### T8: Modelo de spec e instâncias com deduplicação

**What**: Tipos `Project`, `Spec`, `Instance`; agrupamento por `(projeto, pasta)`; hash de conteúdo por instância; instância padrão por `mtime` com desempate pelo caminho; estado (diverge, idêntica).
**Onde**: `crates/quire-core/src/model.rs`
**Depende de**: T7
**Requirement**: DUP-02, DUP-03, DUP-04
**Pronto quando**:

- Cópias idênticas viram uma entrada sem badge; divergentes têm `instance_count > 1` e estado por instância
- Desempate determinístico em `mtime` igual
- Dois scan paths sobrepostos não duplicam pastas

**Testes**: unit
**Gate**: fase

---

### T9: Framework de presets

**What**: Struct `Preset` (arquivos primários, parser de cabeçalho, gramática de tasks, regex de IDs, padrão da chave); presets `generic` e `exban`; override por `config.toml`; detecção para sugerir o `exban`.
**Onde**: `crates/quire-core/src/preset.rs`, `config.rs`
**Depende de**: T8
**Requirement**: PST-01, PST-02
**Pronto quando**:

- Projeto sem preset usa `generic`; estrutura do `exban` nas fixtures dispara a sugestão
- Trocar o preset de um projeto muda os arquivos primários reconhecidos
- `config.toml` inválido gera erro tipado com linha, sem pânico

**Testes**: unit
**Gate**: fase

---

### T10: Parsers

**What**: Módulo `parse`: cabeçalho `**Chave:** valor`, `tasks.md` (fases, tarefas, diagrama, detalhamento), tabela de progresso do `summary.md`, extração de IDs `<!-- XXX-00 -->`. Funções puras e tolerantes.
**Onde**: `crates/quire-core/src/parse/`
**Depende de**: T9
**Requirement**: VIW-07, TSK-01, TSK-02, TSK-03, TSK-04, PST-03
**Pronto quando**:

- Testes com as fixtures e com casos de borda (ids fora de ordem, fase sem diagrama, hash ausente)
- Entrada vazia ou fora do formato devolve vazio, sem erro
- Contagem x/y confere com os checkboxes

**Testes**: unit
**Gate**: fase

---

### T11: Parsers JSONL tolerantes

**What**: `jsonl<T>` que devolve itens e contagem de linhas puladas; tipos `GateRun` e `Lesson` com campos opcionais tolerantes.
**Onde**: `crates/quire-core/src/parse/jsonl.rs`
**Depende de**: T10
**Requirement**: LOG-01, LOG-02, LOG-03, LOG-04
**Pronto quando**:

- Linha inválida e linha truncada no fim são puladas e contadas
- Campos extras desconhecidos não quebram o parse

**Testes**: unit
**Gate**: fase

---

### T12: Índice SQLite com scan incremental

**What**: Schema do design, reconciliação por `mtime` e tamanho, tabela `spec_meta` com progresso e status derivados, remoção de specs fantasmas, scan em background com eventos de progresso.
**Onde**: `crates/quire-core/src/index.rs`
**Depende de**: T11
**Requirement**: IDX-01, IDX-02, IDX-03, DSC-04
**Pronto quando**:

- Apagar o cache e reescanear reproduz o mesmo resultado
- Segundo scan sem mudanças não reprocessa arquivos
- Spec removida do disco some do índice
- Lista de ~300 specs sintéticas sai do cache em < 1 s (medido)

**Testes**: unit, integration
**Gate**: fase

---

### T13: Comandos e eventos Tauri com tipos compartilhados

**What**: Comandos do design (`list_projects`, `get_spec`, `read_file`, `get_tasks`, `get_gates`, `get_lessons`, config) e eventos; geração de tipos TS com `tauri-specta`; camada `lib/ipc` no frontend.
**Onde**: `apps/desktop/src-tauri/`, `apps/desktop/src/lib/ipc/`
**Depende de**: T12
**Requirement**: contrato interno (suporta DSC-04, DUP-05, VIW-01)
**Pronto quando**:

- Tipos TS são gerados no build e checados pelo `typecheck`
- Chamada ponta a ponta de `list_projects` sobre as fixtures

**Testes**: integration
**Gate**: fase

---

### T14: Tokens, tema, layout e catálogo de strings

**What**: Tailwind apontando para as variáveis de `docs/design/tokens.css`; tema escuro padrão e claro por classe; layout de três colunas com barra superior e de status; catálogo único de strings (pt-BR); componentes base (chip, botão, segmentado, barra de progresso).
**Onde**: `apps/desktop/src/ui/`, `apps/desktop/src/app/`, `apps/desktop/src/lib/i18n/`
**Depende de**: T13
**Requirement**: VIW-01 (estrutura), A1
**Pronto quando**:

- Layout igual ao wireframe 1 (estrutura, não conteúdo) nos dois temas
- Contraste dos textos ≥ 4.5:1 nos dois temas
- Nenhuma string de UI fora do catálogo

**Testes**: manual, unit
**Gate**: fase

---

### T15: Sidebar de projetos e specs

**What**: Árvore de projetos e specs com filtro por chave/título, filtro de status, barra de progresso, badge de "N versões", rodapé com o estado do scan e indicador de progresso.
**Onde**: `apps/desktop/src/features/workspace/`
**Depende de**: T14
**Requirement**: DSC-04, DSC-05
**Pronto quando**:

- Specs aparecem progressivamente durante o scan
- Scan path com erro mostra o motivo no rodapé
- Selecionar uma spec abre a visão (placeholder até T17)

**Testes**: unit, manual
**Gate**: fase

---

### T16: Pipeline de markdown

**What**: `unified` + `remark-gfm` + `rehype`; realce com `shiki` sob demanda; Mermaid com import dinâmico e tratamento de erro por bloco; outline com seção ativa; seções colapsáveis; layout de leitura (~736 px).
**Onde**: `apps/desktop/src/lib/markdown/`, `features/spec-view/`
**Depende de**: T15
**Requirement**: VIW-02, VIW-03, VIW-04, VIW-05, VIW-06
**Pronto quando**:

- Tabela, lista de tarefas, código e diagrama ASCII renderizam como no wireframe 1
- Mermaid inválido mostra o código e o erro sem quebrar a página
- Outline destaca a seção visível e rola ao clicar
- Documento de 40 KB rola sem travar (meta definida em T2)

**Testes**: unit, manual
**Gate**: fase

---

### T17: Abas, metadados e fallback

**What**: Abas por arquivo primário existente com contagens, aba "outros", faixa de metadados (chips do cabeçalho e link da issue), fallback `.md`/`.json`/texto/binário.
**Onde**: `apps/desktop/src/features/spec-view/`
**Depende de**: T16
**Requirement**: VIW-01, VIW-07, VIW-08, DUP-06
**Pronto quando**:

- Spec parcial mostra só as abas existentes
- Chips e link da issue funcionam; o link abre no navegador
- Arquivo de 2 MB+ abre com aviso e sem realce; binário mostra mensagem
- Cabeçalho mostra branch e worktree

**Testes**: unit, manual
**Gate**: fase

---

### T18: Visão de tasks por fase

**What**: Fases colapsáveis com contagem e barra, diagrama de dependência em chips, linhas de tarefa e painel de detalhe; aviso de somente leitura.
**Onde**: `apps/desktop/src/features/tasks-view/`
**Depende de**: T17
**Requirement**: TSK-01, TSK-02, TSK-03, TSK-04, TSK-05
**Pronto quando**:

- Layout igual ao wireframe 2 com os dados das fixtures
- Selecionar tarefa mostra o detalhamento no painel
- Nenhuma ação escreve no disco

**Testes**: unit, manual
**Gate**: fase

---

### T19: Visões de gates e lessons

**What**: Tabela de gates com filtros e contagens, linha de falha expansível com `tail`; lista de lessons com sinal, escopo, texto e fonte; aviso de linhas puladas.
**Onde**: `apps/desktop/src/features/logs-view/`
**Depende de**: T18
**Requirement**: LOG-01, LOG-02, LOG-03, LOG-04
**Pronto quando**:

- Layout igual ao wireframe 3
- Filtros combinam nível e status; contagens conferem com o arquivo
- JSONL com linha inválida mostra o aviso

**Testes**: unit, manual
**Gate**: fase

---

### T20: Seletor de instâncias

**What**: Popover do badge com branch, tipo, caminho, tempo e estado; troca de instância valendo na sessão; abertura padrão pela mais recente.
**Onde**: `apps/desktop/src/features/instances/`
**Depende de**: T19
**Requirement**: DUP-04, DUP-05, DUP-06
**Pronto quando**:

- Popover igual ao wireframe 4 (sem o diff, que é v0.2+)
- Trocar de instância recarrega as abas mantendo a aba atual
- Spec com instância única não mostra badge

**Testes**: unit, manual
**Gate**: fase

---

### T21: Watcher ligado ao índice e à UI

**What**: `watch` com `notify` e polling por detecção ou config; debounce; atualização do índice e eventos para a UI; recarga que mantém aba, seção e rolagem; fallback quando a instância some.
**Onde**: `crates/quire-core/src/watch.rs`, `apps/desktop/src/features/*`
**Depende de**: T20
**Requirement**: WAT-01, WAT-02, WAT-03, WAT-04
**Pronto quando**:

- Editar um arquivo fora do app atualiza a visão em até 2 s
- Criar e remover pasta de spec atualiza a lista
- Rename atômico de editor não vira "removido"
- Em caminho sem eventos, o polling assume com o intervalo configurado

**Testes**: integration, manual
**Gate**: fase

---

### T22: CLI com single-instance e seletor

**What**: `quire .`, `quire open <chave|caminho>`, `--help`, `--version`; segunda chamada foca a janela e navega; seletor quando a chave é ambígua; instalação do binário no PATH documentada.
**Onde**: `apps/desktop/src-tauri/`, `apps/desktop/src/features/workspace/`
**Depende de**: T21
**Requirement**: CLI-01, CLI-02, CLI-03, CLI-04, CLI-05
**Pronto quando**:

- Dentro de uma worktree das fixtures, `quire .` abre a spec certa
- Chave presente em 2 projetos mostra o seletor
- Segunda chamada não abre segunda janela
- `--help` e `--version` respondem sem janela (verificar no Linux; Windows registrado como P2)

**Testes**: integration, manual
**Gate**: fase

---

### T23: Preferências

**What**: Tela de preferências: scan paths (adicionar, remover, status), preset por projeto, alias do projeto, URL base do tracker, intervalo do polling, tema.
**Onde**: `apps/desktop/src/features/settings/`
**Depende de**: T22
**Requirement**: DSC-05, PST-02
**Pronto quando**:

- Alterações persistem em `config.toml` e valem sem reiniciar
- Path inválido mostra o motivo na lista

**Testes**: unit, manual
**Gate**: fase

---

### T24: Documentação, empacotamento e roteiro E2E

**What**: README de uso, CONTRIBUTING; empacotamento Linux (AppImage e deb) e Windows (msi) por tag no CI; roteiro E2E manual sobre `fixtures/workspace` cobrindo os critérios P1; atualizar `summary.md`.
**Onde**: `README.md`, `CONTRIBUTING.md`, `.github/workflows/release.yml`, `docs/e2e.md`
**Depende de**: T23
**Requirement**: OSS-01, OSS-04
**Pronto quando**:

- Tag de teste gera os artefatos nos dois sistemas
- Roteiro E2E executado inteiro e anotado no `summary.md`
- Gate final verde

**Testes**: manual
**Gate**: final

---

## Mapa de execução das fases

```
Fase 0 → Fase 1 → Fase 2 → Fase 3 → Fase 4 → Fase 5

Fase 0:  T1 → T2 → T3 → T4
Fase 1:  T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12
Fase 2:  T13 → T14 → T15
Fase 3:  T16 → T17 → T18 → T19 → T20
Fase 4:  T21 → T22
Fase 5:  T23 → T24
```
