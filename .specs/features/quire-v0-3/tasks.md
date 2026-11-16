# QUIRE v0.3 — Tarefas

## Execução

- Repositório: quire
- Base: `main` com a v0.2 mergeada (PR #2, `5efaa19`); release de teste `v0.2.0-rc.1` em rascunho. As revisões pendentes da v0.2 não bloqueiam (Q9, padrão)
- Verificação: full
- Paralelo: não
- E2E: manual (roteiro em T23)
- Idioma: pt-BR

## Execution Protocol (MANDATORY -- do not skip)

Siga o `AGENTS.md`. Implemente **uma tarefa por vez, na ordem das fases**. Antes de começar uma tarefa, releia os critérios de aceite dos IDs em `Requirement`. Rode o gate indicado; só depois marque `- [x]` e acrescente o hash curto do commit ao fim da linha. Se algo na spec ou no design não funcionar, **pare e registre no `summary.md` (Journal)**; não ajuste o teste ao resultado. Commits em Conventional Commits, sem `Co-Authored-By`. Nunca faça push, abra PR ou crie tag sem o autor pedir.

---

**Spec**: `.specs/features/quire-v0-3/spec.md`
**Design**: `.specs/features/quire-v0-3/design.md`
**Status**: rascunho, aguardando as decisões do autor sobre Q1–Q10

**Condições obrigatórias da execução:**

1. As perguntas Q1–Q10 da spec têm um padrão conservador. Sem resposta do autor, a execução usa o padrão. Uma tarefa que precise de algo fora do padrão e das decisões registradas para e registra no Journal.
2. A Fase 0 vem antes das features. A T1 depende do autor numa máquina com WSL e, pelo padrão da Q7, **não bloqueia** as fases seguintes: sem a rodada, `\\wsl$`, DrvFs e 9p continuam somente leitura. As T2, T3 e T4 bloqueiam.
3. Os spikes (Fase 1) vêm antes das features (Fases 2 a 6). Cada spike registra a decisão no `summary.md`; um resultado que contradiga o design **para a execução** até o autor decidir.
4. Nenhuma tarefa escreve no repositório git do usuário nem em arquivo de spec fora do pipeline do TD-12. Os testes de histórico criam repositórios em tempdir; os de criação de arquivo e de export usam tempdir.
5. Fixtures são sintéticas. Nenhum dado real de empresa entra no repositório, nem em spike: os repositórios grandes dos spikes são gerados por script numa pasta temporária.
6. `quire-core` e `quire-cli` não dependem de Tauri.

## Comandos de gate

Os mesmos da v0.2:

- **fase:** `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --no-fail-fast && pnpm -r typecheck && pnpm -r test`
- **final:** o gate de fase + `pnpm --filter desktop build` + `cargo build --workspace --release` + `pnpm --filter desktop tauri build --bundles deb`
- **e2e:** roteiro manual de T23 sobre `fixtures/workspace`, com o binário de produção

## Plano de execução

### Fase 0: Dívidas da v0.2

```
T1 (autor, não bloqueia)
T2 → T3 → T4
```

- [ ] T1: Rodadas no WSL e decisão de escrita em `\\wsl$`, DrvFs e 9p
- [ ] T2: Assinatura do MSI: avaliação e decisão
- [ ] T3: Fontes Inter e JetBrains Mono embutidas
- [ ] T4: Seção ativa do outline desde o primeiro render

### Fase 1: Spikes

```
T5 → T6 → T7 → T8
```

- [ ] T5: Spike do índice de referências com ~300 specs
- [ ] T6: Spike do `git log`/`show`/`cat-file` por pasta em repositório grande
- [ ] T7: Spike do export: PDF no WebKitGTK e no WebView2 e Mermaid em SVG
- [ ] T8: Spike do custo do lint incremental

### Fase 2: Referências e navegação

```
T22 → T9 → T10 → T11 → T12
```

- [ ] T22: Fixtures de referências, `STATE.md`, defeitos de lint e histórico git
- [ ] T9: Gramática de referências no preset, extração e resolução no núcleo
- [ ] T10: Referências no índice (esquema v3) e comandos
- [ ] T11: Links no modo Ler, "Referenciado em" e voltar/avançar
- [ ] T12: Cadeia até a evidência e `arquivo:linha` clicável

### Fase 3: Lint

```
T13 → T14 → T15
```

- [ ] T13: Motor de lint, regras e configuração
- [ ] T14: Painel de problemas, selos e lint incremental
- [ ] T15: `quire lint` no CLI

### Fase 4: Histórico e diff via git

```
T16 → T17 → T18
```

- [ ] T16: `git` somente leitura no núcleo e comandos de histórico
- [ ] T17: Painel de histórico, arquivo no commit, diff e commit da tarefa
- [ ] T18: Comparar instâncias

### Fase 5: Export

```
T19 → T20
```

- [ ] T19: Export HTML estático com verificação de vazamento
- [ ] T20: Export PDF

### Fase 6: Criar arquivo ausente

```
T21
```

- [ ] T21: Criar arquivo primário ausente a partir do modelo do preset

### Fase 7: Fechamento da v0.3

```
T23
```

- [ ] T23: Roteiro E2E, documentação, CI e gate final

## Detalhamento das tarefas

### T1: Rodadas no WSL e decisão de escrita em `\\wsl$`, DrvFs e 9p

**What**: A T8 da v0.2, que não rodou por falta de WSL, mais o `rename_spike` nas três classes. Rodar o spike T2 da v0.1 (render no WSLg) e o T3 (watcher em WSL2 ext4, DrvFs e `\\wsl$`) e o `cargo run --release -p quire-core --example rename_spike -- <pasta>` em `/mnt/c/...` (DrvFs), num caminho 9p e em `\\wsl$\...` a partir do Windows. Registrar as linhas na tabela da T11 da v0.2 (copiada para "Medições" da v0.3), decidir cada classe pelo dado (TD-22) e confirmar ou ajustar o TD-05. Remover `apps/desktop/src/spike/`.
**Onde**: máquina do autor com WSL; `crates/quire-core/src/fs_kind.rs` (`VERIFIED_ATOMIC`); `summary.md`
**Depende de**: nenhuma (é do autor; não bloqueia, Q7)
**Requirement**: DEB-11
**Pronto quando**:

- Uma linha por classe na tabela, com leituras concorrentes e leituras ruins, modo, somente leitura, link e arquivo aberto, e a linha observada do `mountinfo`
- `VERIFIED_ATOMIC` só ganha uma classe com 0 leituras ruins em 400 trocas e o resto do roteiro conferido; o teste do `fs_kind` cobre a mudança
- TD-05 confirmado ou ajustado no Journal, com os números do watcher
- `apps/desktop/src/spike/` removida, com o gate verde

**Testes**: manual (autor), unit (`fs_kind`)
**Gate**: fase

---

### T2: Assinatura do MSI: avaliação e decisão

**What**: Levantar as opções para assinar o `.msi` (não assinar; certificado de assinatura de código comprado; serviço de assinatura para projetos open source), com custo, processo de obtenção, guarda do segredo no GitHub Actions (só em tag do repositório principal, nunca em PR de fork) e efeito no SmartScreen. Recomendar uma e registrar a decisão do autor como TD-37 (Q6). Sem decisão, aplicar o padrão: não assinar e documentar no README o aviso do SmartScreen ("Mais informações" → "Executar assim mesmo"). Se a decisão for assinar, a implementação no `release.yml` entra nesta tarefa, com a verificação da assinatura (`signtool verify` ou equivalente) no job do MSI.
**Onde**: `design.md` (TD-37), `README.md`, `.github/workflows/release.yml` e `ci.yml` (se assinar)
**Depende de**: nenhuma
**Requirement**: DEB-12
**Pronto quando**:

- Tabela de opções no `summary.md` ("Medições"), com as fontes consultadas e o que ficou NÃO VERIFICADO (preços mudam)
- TD-37 registrado com a decisão e o motivo
- Se não assinar: README com o passo a passo do SmartScreen. Se assinar: assinatura verificada no CI numa tag de teste, e nenhum segredo exposto a PR de fork (`actionlint` limpo)

**Testes**: manual, CI (se assinar)
**Gate**: fase

---

### T3: Fontes Inter e JetBrains Mono embutidas

**What**: Embutir as duas fontes em woff2 (só os pesos usados, subconjunto Latin Extended) com a licença OFL, declarar `@font-face` locais nos tokens e usar nas duas telas e na impressão (TD-35). Medir o crescimento do bundle.
**Onde**: `apps/desktop/src/assets/fonts/`, `docs/design/tokens.css` ou o CSS do app, `LICENSE`/`NOTICE` das fontes
**Depende de**: T2
**Requirement**: DEB-13
**Pronto quando**:

- O app de produção não faz nenhuma requisição de rede para fontes (CSP inalterada, conferida no binário)
- Crescimento do bundle ≤ 400 KB, medido e registrado
- Capturas nos dois temas mostram as fontes embutidas mesmo com a máquina sem Inter instalada (conferido em ambiente sem a fonte do sistema)

**Testes**: unit (o CSS referencia só arquivos locais), manual (revisão visual)
**Gate**: fase

---

### T4: Seção ativa do outline desde o primeiro render

**What**: O outline destaca a seção visível ao abrir a spec e ao restaurar uma posição de rolagem, sem esperar o usuário rolar até um título (pendência da T16 da v0.1). Base para a restauração do voltar/avançar (T11).
**Onde**: `apps/desktop/src/lib/markdown/Outline.tsx`, `features/spec-view/`
**Depende de**: T3
**Requirement**: DEB-14
**Pronto quando**:

- Teste de componente: com a rolagem inicial no meio do documento, a seção correspondente vem destacada no primeiro render
- Com a rolagem no topo, a primeira seção vem destacada

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T5: Spike do índice de referências com ~300 specs

**What**: Exemplo `cargo run --release -p quire-core --example refs_spike`, no molde do `fts_spike` da v0.2: corpus sintético de 300 specs × 7 arquivos com IDs definidos e usados (`**Requirement**`, `validation.md`, texto corrido), chaves de issue entre specs, `AD-xxx` e `arquivo:linha`, umas 50 referências por arquivo. Medir indexação completa junto com o FTS, tamanho das tabelas novas em relação ao texto, reindexar um arquivo, e "Referenciado em" (por ID na spec, por chave no workspace, por `AD-xxx` no projeto), p50 e p95. Decidir se o vínculo arquivo → documento reaproveita o `file_docs` do FTS.
**Onde**: `crates/quire-core/examples/refs_spike.rs`, `summary.md`
**Depende de**: T4
**Requirement**: REF-10
**Pronto quando**:

- Tabela em "Medições" com as condições e os números
- Decisão registrada: o orçamento de REF-10 confirmado ou ajustado na spec, e a forma de `file_docs`
- Um resultado fora do orçamento por mais de 2× para a execução e vai ao autor (condição 3)

**Testes**: manual (spike)
**Gate**: fase

---

### T6: Spike do `git log`/`show`/`cat-file` por pasta em repositório grande

**What**: Script que gera, numa pasta temporária, um repositório sintético grande (por exemplo, 50 mil commits, 20 mil arquivos, a pasta da spec tocada em 1 de cada 200 commits). Medir `git log --max-count=50 -- <pasta>` (primeira e quinta página), `show <c>:<arquivo>`, `cat-file --batch` com 7 arquivos, `diff-tree` de um commit e `rev-parse` de hash curto, p50 e p95, com o endurecimento do TD-31 ligado. Medir também com o `git` frio (cache do SO limpo, se possível) e num clone raso. Conferir se `.git` fica byte a byte igual depois de tudo.
**Onde**: `scripts/` ou `crates/quire-core/examples/history_spike.rs`, `summary.md`
**Depende de**: T5
**Requirement**: HIS-08
**Pronto quando**:

- Tabela em "Medições" com a versão do `git`, o tamanho do repositório e os números
- Tempo limite por chamada decidido (confirmar os 5 s da v0.1 ou ajustar) e orçamento de HIS-08 confirmado ou ajustado
- Repositório intacto depois das medições, conferido por hash de todo o `.git`

**Testes**: manual (spike)
**Gate**: fase

---

### T7: Spike do export: PDF no WebKitGTK e no WebView2 e Mermaid em SVG

**What**: Página isolada (como o `spike-editor` da v0.2) que gera o HTML do export da spec de 40 KB das fixtures dos spikes (6 diagramas), com `hast-util-to-html`, Shiki em estilos embutidos e Mermaid com `htmlLabels: false` e tema claro. Medir tempo e tamanho (EXP-07). Imprimir para PDF por `WebviewWindow::print` numa janela auxiliar e no próprio webview com contêiner de impressão, no Linux (diálogo GTK) e no Windows (pelo autor ou no CI, se um runner conseguir; senão NÃO VERIFICADO). Avaliar `break-inside: avoid` em tabelas, código e diagramas, títulos órfãos, números de página e tamanho do PDF. Conferir que o HTML com as fontes do sistema e o PDF com as embutidas (T3) mostram o mesmo conteúdo.
**Onde**: `apps/desktop/src/spike-export/`, `summary.md`
**Depende de**: T6
**Requirement**: EXP-05, EXP-06, EXP-07, DEB-13
**Pronto quando**:

- Tabela em "Medições" por plataforma: tempo do HTML, tamanho do HTML, PDF gerado ou não, páginas, quebras ruins observadas, tamanho do PDF
- Decisão registrada: janela auxiliar ou contêiner no webview; PDF disponível ou não em cada plataforma (TD-34)
- Os 6 diagramas das fixtures com `htmlLabels: false` comparados com o modo Ler (capturas); diferenças registradas
- A pasta `spike-export` sai no fim da Fase 5 (T20)

**Testes**: manual (spike)
**Gate**: fase

---

### T8: Spike do custo do lint incremental

**What**: Com um protótipo das regras de LNT-02 sobre o corpus do T5, medir o lint de uma spec a partir do disco (frio e quente), de 300 specs em sequência e em paralelo (threads), e a memória. Decidir se o lint da abertura roda inteiro em segundo plano ou só sob demanda por spec.
**Onde**: `crates/quire-core/examples/lint_spike.rs`, `summary.md`
**Depende de**: T7
**Requirement**: LNT-08
**Pronto quando**:

- Tabela em "Medições"; orçamento de LNT-08 confirmado ou ajustado
- Decisão sobre o lint da abertura registrada

**Testes**: manual (spike)
**Gate**: fase

---

### T22: Fixtures de referências, `STATE.md`, defeitos de lint e histórico git

**What**: Ampliar `fixtures/templates` e `scripts/make-fixtures.sh`: `STATE.md` com `AD-001` a `AD-003` em `acme_gateway`; o mesmo ID (`TOK-01`) em duas specs do mesmo projeto; a mesma chave de issue em dois projetos (já existe para o `quire open`; reaproveitar); referências cruzadas por chave; `arquivo:linha` para código das fixtures; uma spec "limpa" sem nenhum problema e uma spec com um defeito plantado por regra de LNT-02 (lista esperada num arquivo `expected-lint.json` ao lado); tabela de progresso divergente no `summary.md` de uma spec; e uma sequência de commits no `make-fixtures.sh` que toca a pasta de uma spec, com os hashes curtos escritos no `tasks.md` e no `gates.jsonl` gerados pelo script.
**Onde**: `fixtures/templates/`, `scripts/make-fixtures.sh`, `fixtures/README.md`
**Depende de**: T8
**Requirement**: REF-01, REF-02, LNT-06
**Pronto quando**:

- `scripts/make-fixtures.sh` gera o workspace de forma idempotente, e os testes da v0.1 e da v0.2 continuam verdes
- `expected-lint.json` lista o defeito plantado de cada regra, com arquivo e linha
- Os hashes do `tasks.md` e do `gates.jsonl` gerados apontam para commits reais do repositório das fixtures
- Nenhum dado real; nomes `acme` e `tracker.example`, como no resto

**Testes**: unit (scripts conferidos pelos testes existentes)
**Gate**: fase

---

### T9: Gramática de referências no preset, extração e resolução no núcleo

**What**: `quire-core::refs` (TD-25, TD-26): `ReferenceGrammar` no `Preset` (`requirement`, `issue_key`, `decision`, `file_line`), leitura de `decision_pattern` e `file_line` no `config.toml`, `extract` com papéis de definição e uso fora de código, e `resolve` por escopo (REF-03) com os motivos de quebra (REF-08). `evidence_chain` fica na T12.
**Onde**: `crates/quire-core/src/refs.rs`, `preset.rs`, `config.rs`
**Depende de**: T22
**Requirement**: REF-01, REF-02, REF-03
**Pronto quando**:

- Testes por tabela da extração: ID em comentário (definição), em `**Requirement**`, na tabela do `validation.md` e em texto (usos); nada dentro de bloco cercado nem de código inline; `AD-001` em título do `STATE.md` (definição) e em texto (uso); `src/main.ts:12` e `src\main.ts:12` normalizados
- Testes de resolução com as fixtures: `TOK-01` de cada spec resolve na própria spec; a chave em dois projetos devolve dois destinos; `AD-009` sem definição é quebrada com `missing_definition`; `../../etc/passwd:1` e caminho absoluto são `outside_repo`; linha além do fim é `line_out_of_range`
- O preset `generic` só extrai `arquivo:linha` e links; um `decision_pattern` inválido no `config.toml` dá erro no campo, como o `id_pattern`

**Testes**: unit
**Gate**: fase

---

### T10: Referências no índice (esquema v3) e comandos

**What**: Esquema v3 com `ref_docs` e `refs` (TD-27), reindexação na mesma transação do FTS (REF-02), `STATE.md` do repositório indexado como arquivo do projeto, e os comandos `references_at` e `referenced_by` com bindings regenerados.
**Onde**: `crates/quire-core/src/index.rs`, `apps/desktop/src-tauri/src/` (comando novo), `apps/desktop/src/lib/ipc/bindings.ts`
**Depende de**: T9
**Requirement**: REF-03, REF-08, REF-10
**Pronto quando**:

- Banco v2 existente é reconstruído como v3 sem erro (teste com um banco v2 de fixture)
- Mudar um arquivo reindexa só as ocorrências dele; cópias idênticas em três worktrees não duplicam linhas
- Teste do shell sobre as fixtures: `referenced_by` de `TOK-01` em cada spec só traz usos da própria spec; da chave da ACME-142 traz as specs que a citam, com o projeto
- Orçamento de REF-10 conferido num teste com o corpus do T5 (ignorado por padrão, como os de desempenho da v0.2) e no Journal

**Testes**: unit, integração (shell)
**Gate**: fase

---

### T11: Links no modo Ler, "Referenciado em" e voltar/avançar

**What**: No modo Ler, IDs, chaves, `AD-xxx` e `arquivo:linha` resolvidos viram links acessíveis (botões ou âncoras reais, com rótulo); quebrados ganham estilo próprio e motivo (REF-08). Chips de requisito da aba tasks, tabela do `validation.md` e `source` das lessons também navegam (REF-04). Painel "Referenciado em" (REF-05). Store de navegação com voltar/avançar (TD-28) por Alt+←/Alt+→, botões laterais do mouse e botões na barra, passando pela guarda. Strings no catálogo `pt-BR` e `en`.
**Onde**: `apps/desktop/src/features/references/`, `features/navigation/`, `lib/markdown/`, `features/tasks-view/`, `features/logs-view/`, `lib/i18n/`
**Depende de**: T10
**Requirement**: REF-04, REF-05, REF-07, REF-08
**Pronto quando**:

- Testes de componente: clicar em `TOK-01` no `tasks.md` abre o `spec.md` da mesma spec na linha do critério; uma referência quebrada não navega e anuncia o motivo
- Teste do store: voltar e avançar restauram spec, instância, aba e rolagem; com alterações não salvas, voltar mostra a guarda, e Cancelar mantém tudo
- Contraste dos estilos de link e de quebrada ≥ 4.5:1 nos dois temas (conferido com os tokens)
- Capturas nos dois temas em `/tmp/quire-shots/v0-3/` (painel aberto, link quebrado)

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T12: Cadeia até a evidência e `arquivo:linha` clicável

**What**: `evidence_chain` no núcleo e comando (REF-06): tarefas que citam o ID, com estado e hashes; linha do `validation.md`; gates cujo `head` começa com um dos hashes. Na UI, a cadeia aparece no "Referenciado em" de um critério, com cada elo clicável. `arquivo:linha` dentro da pasta da spec abre na aba e na linha; fora dela, "Abrir no aplicativo padrão" (pelo `opener`, recusando caminho fora do repositório) e "Copiar `caminho:linha`" (REF-09).
**Onde**: `crates/quire-core/src/refs.rs`, `apps/desktop/src-tauri/src/`, `apps/desktop/src/features/references/`
**Depende de**: T11
**Requirement**: REF-06, REF-09
**Pronto quando**:

- Teste do núcleo com as fixtures (T22): o critério com tarefa concluída chega à tarefa, à linha do `validation.md` e ao gate do hash dela; um critério sem tarefa devolve a cadeia vazia com o motivo
- Teste do shell: o `opener` não é chamado para caminho fora do repositório
- E2E parcial: do critério ao gate só com cliques, e de volta com Alt+← (vai para o roteiro da T23)

**Testes**: unit, integração (shell)
**Gate**: fase

---

### T13: Motor de lint, regras e configuração

**What**: `quire-core::lint` (TD-29): `SpecInput` lido do disco, as dez regras de LNT-02 com pré-condições de estrutura (LNT-06), `progress-mismatch` sobre `parse::summary_progress` (PRG-02), problemas com código e parâmetros, catálogo de mensagens em inglês no núcleo, e a configuração preset → projeto → spec com níveis e avisos para entradas desconhecidas (LNT-03). O `ProjectConfig` ganha `lint`.
**Onde**: `crates/quire-core/src/lint/`, `config.rs`, `preset.rs`
**Depende de**: T12
**Requirement**: LNT-01, LNT-02, LNT-03, LNT-06, PRG-02
**Pronto quando**:

- A spec limpa das fixtures dá zero problemas, e a spec com defeitos dá exatamente o `expected-lint.json` (regra, arquivo, linha)
- Teste por regra com o caso positivo e o caso de pré-condição ausente (sem `tasks.md`, sem `validation.md`, preset `generic`)
- Teste de que o lint não escreve nada: hash de todos os arquivos da pasta antes e depois
- Config com regra desconhecida e com nível desconhecido: avisos, nenhuma falha de config; spec com `off` não reporta a regra

**Testes**: unit
**Gate**: fase

---

### T14: Painel de problemas, selos e lint incremental

**What**: Comandos `lint_spec` e `lint_summary` e evento `LintChanged` depois da reconciliação de uma spec (LNT-05), com o lint da abertura decidido no spike T8. Painel de problemas por spec com severidade, regra, mensagem traduzida e `arquivo:linha` clicável (pela navegação da T11); contagem de erros e avisos no cabeçalho da spec e na lista lateral (LNT-04). Strings no catálogo.
**Onde**: `apps/desktop/src-tauri/src/`, `apps/desktop/src/features/problems/`, `features/workspace/`, `features/spec-view/`, `lib/i18n/`
**Depende de**: T13
**Requirement**: LNT-04, LNT-05, LNT-08
**Pronto quando**:

- Teste do shell: mudar um arquivo da spec com defeito emite `LintChanged` só para ela, e o resultado novo reflete a mudança em até 1 s depois da reconciliação
- Teste de componente: o selo conta só erro e aviso; o clique num problema abre o arquivo na linha
- A primeira tela não espera o lint (medido como na v0.1, registrado no Journal)
- Capturas nos dois temas

**Testes**: unit, integração (shell), manual (revisão visual)
**Gate**: fase

---

### T15: `quire lint` no CLI

**What**: Subcomando `lint` no `plugins.cli` do `tauri.conf.json`; parser compartilhado como biblioteca do `quire-cli` (TD-30); no app, o `main` trata o `lint` antes de montar o Tauri; no `quire-cli`, o `lint` roda no processo, com o `quire-core`. Saída em texto e JSON, `--fail-on`, `--preset`, `--config` e os códigos 0, 1, 2 e 3 (LNT-07). CI: teste de fumaça do `quire lint` em Linux e Windows, e passo informativo que roda o `quire lint` nas `.specs` do próprio Quire (OSS-08). Se a Q3 for aprovada, o `release.yml` publica o `quire-cli` para Linux.
**Onde**: `crates/quire-cli/`, `apps/desktop/src-tauri/src/main.rs` e `cli.rs`, `tauri.conf.json`, `.github/workflows/ci.yml` (e `release.yml` se Q3)
**Depende de**: T14
**Requirement**: LNT-07, OSS-08
**Pronto quando**:

- Testes de integração do binário sobre as fixtures: spec limpa → 0 e sem saída de problema; spec com erro → 1; só avisos → 0 com `--fail-on error` e 1 com `--fail-on warning`; argumento inválido → 2; caminho inexistente → 3; JSON válido e estável (snapshot)
- `quire lint` com o app aberto não encaminha ao app e não abre janela (teste do shell com single-instance)
- Teste de paridade: a saída do app e a do `quire-cli` para os mesmos argumentos é idêntica
- CI do Windows: `quire lint` pelo `quire.cmd` em `cmd` e PowerShell com os mesmos códigos; `actionlint` limpo
- Nenhum arquivo criado ou alterado na pasta analisada nem fora dela (sem índice, sem config gravada)

**Testes**: unit, integração, CI (Linux e Windows)
**Gate**: fase

---

### T16: `git` somente leitura no núcleo e comandos de histórico

**What**: `quire-core::history` (TD-31): lista fechada de subcomandos, validação de hash e caminho, endurecimento por `-c` e variáveis de ambiente, saída `-z` com teto de bytes, tempo limite do spike T6, cancelamento por id. Funções de log paginado, arquivo no commit, arquivos alterados entre commits, commit por hash curto (ambíguo e inexistente). Comandos `history_log`, `history_file`, `history_changes`, `history_commit` e `history_cancel` com bindings.
**Onde**: `crates/quire-core/src/history.rs`, `apps/desktop/src-tauri/src/`
**Depende de**: T15
**Requirement**: HIS-01, HIS-02, HIS-03, HIS-04, HIS-05, HIS-06, HIS-08
**Pronto quando**:

- Testes em repositório criado em tempdir: log da pasta da spec, paginação, arquivo num commit, alterações entre commits, root commit, hash curto ambíguo e inexistente, clone raso
- **Teste de repositório intacto:** hash de todos os arquivos de `.git` (inclusive `index`) igual antes e depois de todas as operações
- **Teste de configuração hostil:** `.git/config` com `core.fsmonitor`, `diff.external` e `diff.x.textconv` apontando para um script que cria um arquivo-sentinela; depois de todas as operações, o sentinela não existe (no Windows, o mesmo com um `.cmd`)
- Teste com `git` inexistente e com um "git" que dorme além do tempo limite: erro tipado, processo encerrado, nenhum processo filho sobrando
- Hash ou caminho com `--`, espaço, `;` ou `..` é recusado antes de chamar o `git`

**Testes**: unit, integração (shell), CI (Linux e Windows)
**Gate**: fase

---

### T17: Painel de histórico, arquivo no commit, diff e commit da tarefa

**What**: `lib/diff/` com o diff da v0.2 extraído de `features/conflict` (TD-32), sem mudar o comportamento do conflito. Painel de histórico da spec (HIS-01) com paginação; arquivo no commit renderizado como no modo Ler, somente leitura, com faixa e "voltar à versão atual" (HIS-02); seleção de dois commits, ou commit e versão atual, com o diff por arquivo (HIS-03); hashes da aba tasks e `head` da aba gates clicáveis, abrindo o commit (HIS-04). Cancelamento ao sair do painel; estados de tempo limite, `git` ausente e clone raso.
**Onde**: `apps/desktop/src/lib/diff/`, `features/history/`, `features/tasks-view/`, `features/logs-view/`, `features/conflict/`, `lib/i18n/`
**Depende de**: T16
**Requirement**: HIS-01, HIS-02, HIS-03, HIS-04
**Pronto quando**:

- Os testes do conflito da v0.2 continuam verdes depois da extração do diff
- Testes de componente com IPC simulado: paginação, arquivo no commit sem modo Editar, diff de dois commits, commit não encontrado, tempo limite com "tentar de novo"
- Nas fixtures (T22), o hash de uma tarefa abre o commit certo com o diff da pasta da spec
- Capturas nos dois temas

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T18: Comparar instâncias

**What**: "Comparar" no seletor de instâncias (wireframe 4): escolher duas instâncias, ver a lista de arquivos (iguais, diferentes, só numa) e o diff por arquivo pelo `lib/diff`, com o conteúdo lido do disco pelo `read_file` (HIS-07, TD-32). Com alterações não salvas, a comparação usa o disco e avisa que o buffer não entra.
**Onde**: `apps/desktop/src/features/spec-view/` (seletor), `features/history/`
**Depende de**: T17
**Requirement**: HIS-07
**Pronto quando**:

- Teste com as três cópias da ACME-142 das fixtures: a lista de arquivos e os diffs batem com o conteúdo das pastas
- Funciona com o `git` ausente (teste com o resolvedor sem `git`)
- Capturas nos dois temas

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T19: Export HTML estático com verificação de vazamento

**What**: `features/export/` gera o HTML da árvore do modo Ler com a decisão do spike T7 (TD-33): escolha de arquivos (só `.md`, EXP-08), CSS do tema claro embutido, Shiki em estilos, Mermaid em SVG, links reescritos (EXP-03), cabeçalho com título, chave, projeto e data. Backend: `export_check` (EXP-02) e `export_save` com o `tauri-plugin-dialog` só no Rust e `write::save_export` (EXP-04). Dependências novas: `hast-util-to-html` e `tauri-plugin-dialog`, registradas no Journal com o tamanho.
**Onde**: `apps/desktop/src/features/export/`, `apps/desktop/src-tauri/src/`, `crates/quire-core/src/write.rs`, `capabilities/default.json` (sem permissão de diálogo para o webview)
**Depende de**: T18
**Requirement**: EXP-01, EXP-02, EXP-03, EXP-04, EXP-07, EXP-08
**Pronto quando**:

- **Teste de vazamento:** com as fixtures numa pasta temporária de caminho conhecido, o HTML gerado não contém a pasta pessoal, o usuário, a raiz do repositório, a pasta da instância nem a pasta da config; com um bloco de código contendo a pasta pessoal, `export_check` lista a ocorrência
- O HTML não tem `<script>`, atributo `on*`, `src`/`href` remoto além dos links `http(s)` do conteúdo, nem `@import`/`url()` externo (teste sobre o arquivo)
- Teste do `save_export`: escrita atômica num tempdir; falha injetada deixa o destino anterior intacto; destino numa classe bloqueada é recusado
- A capability do webview não ganha `dialog:*` nem `fs:*` (teste sobre o `default.json`)
- Tempo e tamanho de EXP-07 conferidos com a spec de 40 KB e registrados
- O HTML abre num navegador sem rede com diagramas, tabelas e código (manual, registrado)

**Testes**: unit, integração (shell), manual
**Gate**: fase

---

### T20: Export PDF

**What**: Impressão do export pelo mecanismo decidido no spike T7 (TD-34), com folha `@media print` (tema claro, `break-inside: avoid` em tabelas, código e diagramas, títulos sem ficar órfãos). Opção indisponível com motivo onde o spike disse que não funciona (EXP-05). Remover `apps/desktop/src/spike-export/`.
**Onde**: `apps/desktop/src/features/export/`, `apps/desktop/src-tauri/src/`
**Depende de**: T19
**Requirement**: EXP-05, EXP-06
**Pronto quando**:

- PDF da spec de 40 KB gerado no Linux pelo diálogo do sistema, com as fontes embutidas e sem tabela ou diagrama partido quando cabe numa página (manual, PDF em `/tmp/quire-shots/v0-3/`)
- Windows: o mesmo pelo autor ou no CI, se possível; senão, NÃO VERIFICADO no Journal
- Diferenças de paginação entre os motores registradas (EXP-06)
- `spike-export/` removida, com o gate verde

**Testes**: manual
**Gate**: fase

---

### T21: Criar arquivo primário ausente a partir do modelo do preset

**What**: `quire-core::template` com os modelos do `exban` (título e títulos das seções fixas do `spec-format.md`, sem conteúdo inventado) e sobrescrita em `[presets.<nome>.templates]`; comando `create_spec_file` com validação do nome (NEW-03), criação exclusiva por `write::save_new` e `fs_kind` (TD-36). Na UI, a aba do arquivo ausente com "Criar `<arquivo>`" (NEW-01) e a abertura no modo Editar (NEW-04).
**Onde**: `crates/quire-core/src/template.rs`, `config.rs`, `write.rs`, `apps/desktop/src-tauri/src/`, `apps/desktop/src/features/spec-view/`, `features/editor/`
**Depende de**: T20
**Requirement**: NEW-01, NEW-02, NEW-03, NEW-04
**Pronto quando**:

- Testes do núcleo em tempdir: cria `validation.md` com `{key}` e `{title}` preenchidos; com o arquivo criado por fora entre a checagem e a criação, não sobrescreve (o conteúdo de fora fica intacto byte a byte); `gates.jsonl`, `notes.md` (não primário), `../x.md` e uma pasta de spec nova são recusados; pasta sem permissão de escrita falha sem criar nada
- Teste do shell: depois de criar, a aba existe, o lint e o índice refletem em até 2 s, e nenhum aviso de conflito aparece
- Classe bloqueada (TD-22): ação desabilitada com o motivo
- Capturas nos dois temas

**Testes**: unit, integração (shell), manual (revisão visual)
**Gate**: fase

---

### T23: Roteiro E2E, documentação, CI e gate final

**What**: Roteiro E2E da v0.3 em `docs/e2e.md` (critério de pronto da release: do critério ao gate e ao diff por cliques e de volta; lint com defeito plantado e corrigido no editor; histórico; comparar instâncias; export HTML aberto sem rede; PDF; criar `validation.md`). README e `CHANGELOG.md` (OSS-07). CI com os testes novos em Linux e Windows e o passo informativo do `quire lint` (OSS-08). Conferir `tauri-specta` estável (como a DEB-08 da v0.2). Gate final.
**Onde**: `docs/e2e.md`, `README.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `summary.md`
**Depende de**: T21
**Requirement**: OSS-07, OSS-08, LNT-06
**Pronto quando**:

- Roteiro E2E escrito sobre `fixtures/workspace`, com o binário de produção, e executado no que dá para executar sem olho humano; o resto fica para o autor
- README cobre referências, lint (com exemplo de CI e códigos de saída), histórico, export e criação de arquivo; `CHANGELOG.md` com a v0.3.0
- CI verde em Linux e Windows depois do push que o autor pedir; `actionlint` limpo
- Gate final verde; Relatório da manhã no topo do `summary.md`

**Testes**: e2e (manual), CI
**Gate**: final
