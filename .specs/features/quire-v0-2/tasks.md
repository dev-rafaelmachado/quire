# QUIRE v0.2 — Tarefas

## Execução

- Repositório: quire
- Base: `main` depois do merge da v0.1 (ver P10 da spec)
- Verificação: full
- Paralelo: não
- E2E: manual (roteiro em T26, com os cenários de perda de dados)
- Idioma: pt-BR

## Execution Protocol (MANDATORY -- do not skip)

Siga o `AGENTS.md`. Implemente **uma tarefa por vez, na ordem das fases**. Antes de começar uma tarefa, releia os critérios de aceite dos IDs em `Requirement`. Rode o gate indicado; só depois marque `- [x]` e acrescente o hash curto do commit ao fim da linha. Se algo na spec ou no design não funcionar, **pare e registre no `summary.md` (Journal)**; não ajuste o teste ao resultado. Commits em Conventional Commits, sem `Co-Authored-By`. Nunca faça push, abra PR ou crie tag sem o autor pedir.

---

**Spec**: `.specs/features/quire-v0-2/spec.md`
**Design**: `.specs/features/quire-v0-2/design.md`
**Status**: rascunho, aguardando aprovação do autor

**Condições obrigatórias da execução:**

1. As perguntas abertas P1 a P10 da spec que afetam uma tarefa precisam estar respondidas antes de ela começar (cada tarefa diz quais).
2. Os spikes (Fase 1) vêm antes de qualquer feature da v0.2 (Fases 2 a 5). Cada spike registra a decisão no `summary.md`; um resultado que contradiga o design **para a execução** até o autor decidir.
3. Nenhuma tarefa escreve em arquivo de spec fora do pipeline do TD-12, nem em teste: os testes de escrita usam tempdir.
4. Fixtures são sintéticas. Nenhum dado real de empresa entra no repositório.
5. `quire-core` não depende de Tauri.

## Comandos de gate

Os mesmos da v0.1 (confirmados lá):

- **fase:** `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --no-fail-fast && pnpm -r typecheck && pnpm -r test`
- **final:** o gate de fase + `pnpm --filter desktop build` + `cargo build --workspace --release` + `pnpm --filter desktop tauri build --bundles deb`
- **e2e:** roteiro manual de T26 sobre `fixtures/workspace`, com o binário de produção

## Plano de execução

### Fase 0: Dívidas da v0.1

```
T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8
```

- [ ] T1: Catálogo em inglês, idioma configurável e erros do backend por código
- [ ] T2: Sugestão visível do preset `exban`
- [ ] T3: Chips de requisito em janela estreita
- [ ] T4: Recusar `QUIRE_CONFIG` e `QUIRE_INDEX` relativos
- [ ] T5: Console do CLI no Windows (P2)
- [ ] T6: Memória com uma spec aberta: medição e orçamento
- [ ] T7: `tauri-specta` estável
- [ ] T8: Rodadas pendentes no WSL e remoção do spike da v0.1

### Fase 1: Spikes

```
T9 → T10 → T11
```

- [ ] T9: Spike do CodeMirror 6 no WebKitGTK (teclas mortas, IME, bundle, arquivo grande)
- [ ] T10: Spike do FTS5 com ~300 specs
- [ ] T11: Spike da semântica de rename atômico (btrfs/ext4/tmpfs, NTFS, DrvFs, 9p, `\\wsl$`)

### Fase 2: Escrita segura no núcleo

```
T12 → T13 → T14 → T15
```

- [ ] T12: Pipeline de escrita atômica com preservação de formato
- [ ] T13: Impressão digital e detecção de conflito
- [ ] T14: Alternância de checkbox por linha
- [ ] T15: Comandos de escrita no shell e bindings

### Fase 3: Editor

```
T16 → T17 → T18 → T19 → T20
```

- [ ] T16: Editor CodeMirror e modos Ler, Editar e Dividir
- [ ] T17: Preview ao vivo e rolagem sincronizada
- [ ] T18: Guarda de alterações não salvas
- [ ] T19: Conflito: diálogo, diff, mudança no disco e supressão da própria escrita
- [ ] T20: Edição com várias instâncias

### Fase 4: Busca e command palette

```
T21 → T22 → T23
```

- [ ] T21: Índice FTS5 incremental
- [ ] T22: Busca global e na spec aberta
- [ ] T23: Command palette

### Fase 5: Tasks e progresso

```
T24 → T25
```

- [ ] T24: Checkbox opt-in do `tasks.md`
- [ ] T25: Progresso atualizado pela própria escrita (e alerta de divergência, se P5)

### Fase 6: Fechamento da v0.2

```
T26
```

- [ ] T26: Fixtures de escrita, roteiro E2E, documentação e gate final

## Detalhamento das tarefas

### T1: Catálogo em inglês, idioma configurável e erros do backend por código

**What**: Criar `lib/i18n/en.ts` com o tipo do `pt-BR` como contrato (TD-19); campo `language` no `config.toml` e nas Preferências, trocado em tempo de execução; idioma do sistema quando ausente (A9, depende de P8). Os erros do backend passam a ser mostrados pelo `CmdError.code` traduzido, com o texto do backend só como detalhe.
**Onde**: `apps/desktop/src/lib/i18n/`, `crates/quire-core/src/config.rs`, `apps/desktop/src-tauri/src/commands.rs`, `features/settings/`
**Depende de**: nenhuma (P8 respondida)
**Requirement**: DEB-01, DEB-02
**Pronto quando**:

- O teste falha se faltar uma chave em `en` ou se sobrar uma chave que não existe em `pt-BR`
- O teste de literais da v0.1 continua cobrindo os componentes
- Trocar o idioma nas Preferências muda a interface sem reiniciar e grava `language` preservando os comentários do `config.toml`
- "path does not exist" aparece traduzido nas Preferências e no rodapé

**Testes**: unit
**Gate**: fase

---

### T2: Sugestão visível do preset `exban`

**What**: Mostrar na sidebar (cabeçalho do projeto) e nas Preferências a sugestão que o índice já calcula (`suggestedPreset`), com a ação "Usar exban", que grava o preset do projeto pelo `set_config` existente. A sugestão desaparece ao aplicar ou ao dispensar; dispensar vale só na sessão.
**Onde**: `features/workspace/`, `features/settings/`
**Depende de**: T1
**Requirement**: DEB-03
**Pronto quando**:

- Nas fixtures com `acme_billing` em `generic`, a sugestão aparece; o clique grava `preset = "exban"` e as abas mudam sem reiniciar
- Nenhuma escrita acontece sem o clique

**Testes**: unit, manual
**Gate**: fase

---

### T3: Chips de requisito em janela estreita

**What**: Nas linhas de tarefa, os chips de requisito encolhem antes do título sumir: mostram os que cabem e um chip "+N"; a lista completa continua no painel da tarefa. Revisar o mesmo padrão nos filtros da aba gates (quebra de linha em 1064 px, registrada na T19 da v0.1).
**Onde**: `features/tasks-view/TasksPanel.tsx`, `features/logs-view/`
**Depende de**: T2
**Requirement**: DEB-04
**Pronto quando**:

- Capturas em 1064 px e 1440 px, nos dois temas, sem conteúdo fora da linha (T16 e T18 do `tasks.md` da própria v0.1 como caso real)
- O "+N" tem rótulo acessível com a lista

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T4: Recusar `QUIRE_CONFIG` e `QUIRE_INDEX` relativos

**What**: Na inicialização, recusar caminhos relativos nas duas variáveis e um índice dentro de uma pasta `.specs` (TD-21). A config fica marcada com erro e não editável, como já acontece com uma config inválida, e a mensagem diz o que fazer. README e CONTRIBUTING já usam `$PWD`.
**Onde**: `apps/desktop/src-tauri/src/state.rs`
**Depende de**: T3
**Requirement**: DEB-05
**Pronto quando**:

- Teste do shell: `QUIRE_CONFIG=fixtures/quire.toml` não cria nenhum arquivo e mostra o erro; com caminho absoluto, funciona
- Salvar as Preferências com a config recusada é impossível

**Testes**: unit
**Gate**: fase

---

### T5: Console do CLI no Windows (P2)

**What**: Decidir entre as opções do TD-20 conforme P9 e implementar a escolhida, para que `quire --help`, `quire --version` e os erros do CLI apareçam no terminal do Windows. Registrar a decisão no Journal.
**Onde**: `apps/desktop/src-tauri/`, `tauri.conf.json` (bundle), `.github/workflows/`
**Depende de**: T4 (P9 respondida)
**Requirement**: DEB-06
**Pronto quando**:

- No CI do Windows, um passo roda `quire --help` (ou o binário de console) e confere a saída e o código 0
- O MSI instala o necessário para o comando `quire` funcionar no PATH (validado no `release.yml` em modo rascunho, se o autor pedir a tag)
- No Linux nada muda

**Testes**: integração (CI Windows), manual
**Gate**: fase

---

### T6: Memória com uma spec aberta: medição e orçamento

**What**: Medir o PSS do app (processo e filhos do WebKit) no build de produção com a spec de 40 KB das fixtures do spike aberta no modo Ler, como base para a v0.2 (a v0.1 mediu 389–418 MB no build de desenvolvimento). Definir o orçamento da v0.2 para o modo Dividir (editor e preview), a ser conferido na T16 e na T26.
**Onde**: `scripts/` (medição), `summary.md`
**Depende de**: T5
**Requirement**: DEB-07
**Pronto quando**:

- Tabela no `summary.md` com cenário, PSS total e só WebKit, nas mesmas condições da T2 da v0.1
- Orçamento do modo Dividir registrado (proposta: a base + 60 MB) e aprovado pelo autor

**Testes**: manual
**Gate**: fase

---

### T7: `tauri-specta` estável

**What**: Verificar se há versões estáveis de `tauri-specta`, `specta` e `specta-typescript`. Se houver, trocar, regenerar os bindings e ajustar o que quebrar; se não houver, registrar a data e as versões vistas.
**Onde**: `apps/desktop/src-tauri/Cargo.toml`, `apps/desktop/src/lib/ipc/bindings.ts`
**Depende de**: T6
**Requirement**: DEB-08
**Pronto quando**:

- Entrada no Journal com as versões encontradas e a decisão
- Se houve troca: bindings regenerados e gate verde

**Testes**: unit (o teste de bindings da v0.1)
**Gate**: fase

---

### T8: Rodadas pendentes no WSL e remoção do spike da v0.1

**What**: Com o autor, rodar o spike T2 da v0.1 no WSLg e o T3 em WSL2 ext4, DrvFs (`/mnt/c`) e `\\wsl$` a partir do Windows (roteiros em `apps/desktop/src/spike/README.md` e `crates/quire-core/examples/README.md`). Registrar as tabelas e confirmar ou ajustar o TD-05. Depois, apagar `apps/desktop/src/spike/`.
**Onde**: `summary.md` da v0.1 (resultados) e da v0.2 (decisão), `apps/desktop/src/spike/`
**Depende de**: T7 (precisa do autor numa máquina com WSL)
**Requirement**: DEB-09
**Pronto quando**:

- Tabelas preenchidas, sem resultado inferido
- Regra do TD-05 confirmada, ou um TD novo se mudar
- Pasta do spike removida num commit próprio

**Testes**: manual
**Gate**: fase

---

### T9: Spike do CodeMirror 6 no WebKitGTK

**What**: Página isolada (como o spike da v0.1) com CodeMirror 6 + `lang-markdown` + busca, sobre a spec de 40 KB e arquivos de 500 KB e 2 MB. Medir: bundle do chunk do editor (bruto e gzip); tempo para abrir; latência de tecla até a tela (p50, p95) durante digitação contínua; memória. Roteiro manual do autor no Hyprland/Wayland com teclado pt-BR (ABNT2 e US-intl): ´ ` ^ ~ ¨ + vogal, ç, composição, IME (ibus e fcitx5), colar, desfazer, seleção com o mouse, localizar e substituir. Repetir o roteiro no WebView2 do Windows, se o autor tiver acesso.
**Onde**: `apps/desktop/src/spike-editor/` (descartável, fora do build)
**Depende de**: T8
**Requirement**: EDT-02, EDT-07, EDT-09 (risco)
**Pronto quando**:

- Tabelas e observações no `summary.md`, com o que foi e o que não foi verificado
- Decisão registrada: CodeMirror confirmado (TD-11), com o limite de tamanho para edição (A8) e o orçamento do chunk; ou parar e levar ao autor se as teclas mortas ou o IME falharem
- Spike removido ou isolado antes do commit final da tarefa

**Testes**: manual
**Gate**: fase

---

### T10: Spike do FTS5 com ~300 specs

**What**: Exemplo no `quire-core` que gera 300 specs sintéticas (7 arquivos, tamanhos como nas fixtures, texto pt-BR com acentos), monta o esquema do design (TD-15) num SQLite temporário e mede: tamanho do banco com e sem FTS; tempo de indexação completa; reindexação de 1 arquivo; latência de consulta (p50, p95) para termo raro, comum, prefixo e dois termos; comportamento do `remove_diacritics 2` ("acao" × "ação", "Ç"); deduplicação com 3 cópias idênticas.
**Onde**: `crates/quire-core/examples/fts_spike.rs` (descartável)
**Depende de**: T9
**Requirement**: SRC-03, SRC-08 (risco)
**Pronto quando**:

- Tabela de resultados no `summary.md`
- Orçamento de tamanho do índice e meta de latência confirmados ou ajustados na spec (SRC-08)
- Decisão: uma linha por seção ou por arquivo; com ou sem deduplicação por hash

**Testes**: manual
**Gate**: fase

---

### T11: Spike da semântica de rename atômico

**What**: Exemplo no `quire-core` que, em cada sistema de arquivos, cria um arquivo, abre um leitor em laço e faz temporário + fsync + rename por cima, verificando: o leitor nunca vê arquivo vazio nem parcial; permissões e atributos depois do rename; comportamento com o arquivo aberto por outro processo (Windows: com e sem `FILE_SHARE_DELETE`); link simbólico; eventos do watcher gerados. Rodar localmente em btrfs e tmpfs, no CI em NTFS (runner Windows), e com o autor em WSL2 ext4, DrvFs, 9p e `\\wsl$`.
**Onde**: `crates/quire-core/examples/rename_spike.rs` (descartável), job opcional no CI
**Depende de**: T10
**Requirement**: SAV-01, SAV-04, SAV-05, SAV-08 (risco)
**Pronto quando**:

- Tabela FS × atomicidade × permissões × arquivo em uso × eventos no `summary.md`, com o NÃO VERIFICADO explícito
- Decisão do autor sobre P6 registrada (se algum FS falhar)
- Lista de caminhos em que a edição fica somente leitura, se for o caso

**Testes**: manual, CI (NTFS)
**Gate**: fase

---

### T12: Pipeline de escrita atômica com preservação de formato

**What**: Módulo `write` (TD-12): `decode`/`encode` de `TextFormat` (BOM, EOL, mistos, newline final) e `save` com os passos do design, exceto as verificações de conflito, que entram na T13 (os pontos de verificação ficam como ganchos testáveis). Erros tipados (`thiserror`). Testes de perda de dados: injeção de falha em cada etapa (criar temporário, escrever, fsync, rename) por um `trait` de sistema de arquivos usado só nos testes; o original é conferido byte a byte depois de cada falha.
**Onde**: `crates/quire-core/src/write.rs`, `crates/quire-core/tests/write.rs`
**Depende de**: T11
**Requirement**: SAV-01, SAV-02, SAV-03, SAV-04, SAV-05, SAV-08, SAV-09, SAV-10, OSS-06
**Pronto quando**:

- Ida e volta `decode` → `encode` idêntica byte a byte para as 8 combinações de BOM × EOL × newline final, e para mistos com confirmação
- Falha injetada em cada etapa deixa o original intacto e não deixa temporário
- Link simbólico: o link continua link e o destino recebe o conteúdo (Unix; Windows se o runner permitir)
- Modo 0600/0644/0755 preservado (Unix); somente leitura recusado (Linux e Windows); dono diferente recusado (Unix, com teste que pula se o runner não permitir)
- Windows: arquivo segurado aberto → nova tentativa e erro "em uso" depois de 1 s (CI)
- Nomes recusados: `.jsonl`, `../x.md`, `a/b.md`, arquivo não listado

**Testes**: unit, integração
**Gate**: fase

---

### T13: Impressão digital e detecção de conflito

**What**: Módulo `fingerprint` (TD-13) e as duas verificações do `save` (passos 3 e 6). O `base = None` serve para "não deve existir" (cópia de conflito e recriação). A cópia de conflito tem nome com data e nunca sobrescreve (CNF-08).
**Onde**: `crates/quire-core/src/fingerprint.rs`, `write.rs`
**Depende de**: T12
**Requirement**: CNF-01, CNF-03, CNF-04, CNF-06, CNF-08
**Pronto quando**:

- Escrita por fora antes do save → `Conflict` e o original do "agente" intacto
- Escrita por fora entre o temporário e o rename (gancho de teste) → `Conflict`, temporário removido
- `mtime` igual com conteúdo diferente (alterado com `filetime`) → `Conflict`
- `mtime` diferente com conteúdo igual → não é conflito
- Cópia de conflito com nome livre; com o nome ocupado, sufixo incremental, nunca sobrescreve

**Testes**: unit, integração
**Gate**: fase

---

### T14: Alternância de checkbox por linha

**What**: `tasks_edit::toggle` (TD-18) sobre bytes, com a gramática do `parse::tasks`, gravando por `write::save` com a base do `tasks.md` carregado.
**Onde**: `crates/quire-core/src/tasks_edit.rs`
**Depende de**: T13 (P4 respondida)
**Requirement**: CHK-02, CHK-03
**Pronto quando**:

- Diff de exatamente um byte, com CRLF, BOM e sem newline final (testes byte a byte)
- `[X]` desmarca para `[ ]`; marcar grava `[x]`
- Linha movida, id diferente, estado diferente ou linha dentro de bloco de código → `Moved`, sem escrita
- Base diferente → `Conflict`, sem escrita
- O `tasks.md` do próprio Quire (v0.1) como caso real em tempdir

**Testes**: unit
**Gate**: fase

---

### T15: Comandos de escrita no shell e bindings

**What**: Comandos `open_for_edit`, `file_fingerprint`, `save_file`, `save_copy`, `toggle_task`, `orphan_temps` e `remove_orphan_temp` (design, "Shell Tauri"). Depois da escrita, `request_scan` da pasta. Erros por código (DEB-02). A aba "outros" deixa de listar os temporários do Quire. Bindings regenerados.
**Onde**: `apps/desktop/src-tauri/src/commands.rs`, `live.rs`, `apps/desktop/src/lib/ipc/`
**Depende de**: T14
**Requirement**: SAV-07, SAV-09, SAV-10, CNF-06 (parte do shell)
**Pronto quando**:

- Testes pelo IPC do mock runtime sobre as fixtures: salvar, conflito, cópia, checkbox, recusa de `.jsonl` e de nome fora da pasta
- Depois de um save, a lista reflete a mudança sem evento do watcher (o watcher desligado no teste)
- Nenhum comando escreve sem ser chamado (busca no código por `write::save` só nos comandos de escrita)

**Testes**: unit, integração
**Gate**: fase

---

### T16: Editor CodeMirror e modos Ler, Editar e Dividir

**What**: `features/editor`: alternância Ler/Editar/Dividir nas abas de `.md` (botões reais, segmentado), editor carregado sob demanda com tema dos tokens, Ctrl+S, indicador de alterações na aba e no título da janela, cadeado e motivo nos arquivos não editáveis. Modo Dividir com preview estático nesta tarefa (o ao vivo é a T17).
**Onde**: `apps/desktop/src/features/editor/`, `features/spec-view/`
**Depende de**: T15 (T9 com decisão registrada)
**Requirement**: EDT-01, EDT-02, EDT-05, EDT-06, EDT-07, EDT-08, EDT-09
**Pronto quando**:

- O chunk inicial não cresce (comparar o `dist` antes e depois); o chunk do editor dentro do orçamento da T9
- Ctrl+S sem alterações não chama `save_file`
- Roteiro de teclas mortas e IME da T9 repetido no app real (autor)
- Latência p95 ≤ 50 ms em 40 KB medida no app real
- Capturas dos três modos nos dois temas

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T17: Preview ao vivo e rolagem sincronizada

**What**: Preview com debounce de 200 ms usando o pipeline da v0.1, cache de Mermaid e Shiki por origem, `data-line` nos blocos e rolagem sincronizada nos dois sentidos, sem eco.
**Onde**: `features/editor/`, `lib/markdown/`
**Depende de**: T16
**Requirement**: EDT-03, EDT-04
**Pronto quando**:

- Preview atualizado em ≤ 500 ms depois da pausa em 40 KB (medido)
- Digitar fora de um bloco Mermaid não refaz o diagrama (teste)
- Rolar o editor até uma tabela leva o preview à tabela, e o inverso também (teste com o mapeamento e verificação manual)
- O modo Ler continua igual ao da v0.1 (testes da T16 da v0.1 verdes)

**Testes**: unit, manual
**Gate**: fase

---

### T18: Guarda de alterações não salvas

**What**: `features/unsaved`: a guarda (Salvar / Descartar / Cancelar) em toda navegação (abas, sidebar, seletor de instância, palette, `cli://open`, preferências) e no fechamento da janela (`onCloseRequested`).
**Onde**: `apps/desktop/src/features/unsaved/`, `App.tsx`, `features/workspace/cli.ts`
**Depende de**: T17
**Requirement**: SAV-06, SAV-07
**Pronto quando**:

- Teste por ponto de navegação: com buffer sujo, nada muda até a escolha; Cancelar mantém tudo
- Fechar a janela com buffer sujo mostra a guarda; Salvar com conflito leva ao diálogo de conflito antes de fechar
- `quire open` com buffer sujo passa pela guarda (teste do store com o evento simulado)

**Testes**: unit, manual
**Gate**: fase

---

### T19: Conflito: diálogo, diff, mudança no disco e supressão da própria escrita

**What**: `features/conflict`: diálogo com Recarregar, Sobrescrever, Salvar como cópia e Ver diferenças (`@codemirror/merge` sob demanda); aviso fixo de mudança no disco com o buffer sujo; recarga automática com o buffer limpo; regra do TD-14 para `spec://changed`; arquivo apagado durante a edição; aviso e remoção de temporário órfão.
**Onde**: `apps/desktop/src/features/conflict/`, `features/spec-view/live.ts`
**Depende de**: T18
**Requirement**: CNF-02, CNF-04, CNF-05, CNF-06, CNF-07, CNF-08, CNF-09, SAV-10
**Pronto quando**:

- Teste do shell: save → nenhum aviso nem recarga em 3 s e uma reconciliação só (sem ciclo)
- Teste: escrita por fora com o buffer limpo → recarrega mantendo o modo; com o buffer sujo → aviso, sem recarga
- "Sobrescrever" depois de "Ver diferenças" escreve o buffer e a nova base é a do disco escrito
- Capturas do diálogo e do diff nos dois temas

**Testes**: unit, integração, manual (revisão visual)
**Gate**: fase

---

### T20: Edição com várias instâncias

**What**: "Edições vão para `<branch>` · `<worktree>`" no cabeçalho em edição; aviso de cópias divergentes; atualização do badge depois do save; instância removida com o buffer sujo (INS-04), com "Salvar como cópia em outra instância" pelo diálogo de conflito.
**Onde**: `features/instances/`, `features/editor/`, `features/spec-view/`
**Depende de**: T19
**Requirement**: INS-01, INS-02, INS-03, INS-04
**Pronto quando**:

- Nas fixtures: salvar na cópia idêntica de ACME-142 faz o badge "2 versões" aparecer em ≤ 2 s
- Remover a worktree com o buffer sujo mantém o buffer e não troca de cópia em silêncio (teste do store)
- Capturas do aviso nos dois temas

**Testes**: unit, manual
**Gate**: fase

---

### T21: Índice FTS5 incremental

**What**: `search` no núcleo com o esquema decidido na T10, `user_version` 2, indexação dentro da reconciliação incremental, remoção de documentos órfãos e a consulta construída (TD-16).
**Onde**: `crates/quire-core/src/search.rs`, `index.rs`
**Depende de**: T20 (T10 com decisão registrada; P7 respondida)
**Requirement**: SRC-01, SRC-02, SRC-03, SRC-07, SRC-08
**Pronto quando**:

- Índice v1 é reconstruído para v2 sem perda (teste)
- Mudar um arquivo reindexa só ele (contador de inserções no teste)
- "acao" acha "ação"; "tare" acha "tarefas"; consultas com `"`, `*`, `-`, `:`, `(`, `AND`, `NEAR` e vazias não dão erro (teste com tabela de casos)
- Latência p95 ≤ 100 ms no gerador da T10 (teste ignorado por padrão, rodado e anotado)

**Testes**: unit, integração
**Gate**: fase

---

### T22: Busca global e na spec aberta

**What**: Comando `search` e painel de resultados agrupados por spec, com trecho destacado, filtros por projeto e arquivo, escopo "spec aberta" e navegação até a seção com as ocorrências destacadas (pela guarda da T18).
**Onde**: `features/search/`, `apps/desktop/src-tauri/src/commands.rs`
**Depende de**: T21
**Requirement**: SRC-04, SRC-05, SRC-06
**Pronto quando**:

- Cópias idênticas aparecem uma vez; cópia divergente mostra a instância certa (fixtures)
- O trecho é montado com marcadores, sem HTML vindo do banco (teste)
- Abrir um resultado leva à aba e à seção, com destaque (teste do store e captura)

**Testes**: unit, integração, manual
**Gate**: fase

---

### T23: Command palette

**What**: `features/palette`: Ctrl+K global (inclusive no editor), casamento por subsequência (TD-17), comandos da CMD-02, busca full-text dentro da palette, recentes, teclado e ARIA, e a guarda da T18 nas navegações.
**Onde**: `apps/desktop/src/features/palette/`
**Depende de**: T22
**Requirement**: CMD-01, CMD-02, CMD-03, CMD-04, CMD-05
**Pronto quando**:

- Testes do casamento (chave, título, abreviações) e da ordem
- Ctrl+K funciona com o foco no editor (teste)
- Navegação só por teclado em todos os comandos (teste com Testing Library)
- Capturas nos dois temas

**Testes**: unit, manual (revisão visual)
**Gate**: fase

---

### T24: Checkbox opt-in do `tasks.md`

**What**: `tasks_checkbox` por projeto no `config.toml` e nas Preferências; checkbox real na aba tasks quando ligado; `toggle_task` com a base da aba; desabilitado com o `tasks.md` sujo no editor; mensagens de `Moved` e `Conflict`.
**Onde**: `crates/quire-core/src/config.rs`, `features/tasks-view/`, `features/settings/`
**Depende de**: T23
**Requirement**: CHK-01, CHK-02, CHK-03, CHK-04, CHK-05
**Pronto quando**:

- Desligado por padrão: a aba é igual à da v0.1 (testes da v0.1 verdes)
- Ligado: marcar T3 da ACME-170 nas fixtures produz um `git diff` de um caractere e o progresso atualizado em ≤ 1 s
- `tasks.md` alterado por fora antes do clique → aviso e recarga, sem escrita
- Capturas nos dois temas

**Testes**: unit, integração, manual
**Gate**: fase

---

### T25: Progresso atualizado pela própria escrita (e alerta de divergência, se P5)

**What**: Depois de um save ou checkbox no `tasks.md`, atualizar a lista, a aba e o cabeçalho a partir da resposta do comando, sem esperar o watcher (PRG-01). Se o autor aprovar P5: aviso quando a tabela de progresso do `summary.md` diverge do `tasks.md` (PRG-02), usando o `parse::summary_progress`.
**Onde**: `features/tasks-view/`, `features/workspace/`, `features/spec-view/`
**Depende de**: T24 (P5 respondida)
**Requirement**: PRG-01, PRG-02 (se P5)
**Pronto quando**:

- Com o watcher em polling de 3 s (`[watch] mode = "poll"`), o progresso muda em ≤ 1 s depois do checkbox (teste)
- Se PRG-02: o aviso aparece no `summary.md` de uma fixture com tabela desatualizada, e some quando a tabela é corrigida

**Testes**: unit, integração
**Gate**: fase

---

### T26: Fixtures de escrita, roteiro E2E, documentação e gate final

**What**: Gerar no `make-fixtures.sh` arquivos com CRLF, BOM, sem newline final, fins de linha mistos, somente leitura e link simbólico (o `.gitattributes` força LF no repositório, então os bytes são produzidos pelo script). Roteiro E2E da v0.2 em `docs/e2e.md` com os cenários de perda de dados: agente escrevendo durante a edição, save com arquivo em uso, crash simulado (matar o processo entre o temporário e o rename, por um gancho de teste) e conferência por `git diff`. README (OSS-05), medição final de memória (DEB-07) e gate final.
**Onde**: `scripts/make-fixtures.sh`, `fixtures/`, `docs/e2e.md`, `README.md`
**Depende de**: T25
**Requirement**: OSS-05, OSS-06, DEB-07
**Pronto quando**:

- Roteiro E2E executado com o binário de produção e anotado no `summary.md`
- Nenhum cenário termina com uma alteração externa perdida sem "Sobrescrever"
- Memória do modo Dividir dentro do orçamento da T6
- Gate final verde; CI verde em Linux e Windows

**Testes**: manual, integração
**Gate**: final

---

## Mapa de execução das fases

```
Fase 0 → Fase 1 → Fase 2 → Fase 3 → Fase 4 → Fase 5 → Fase 6

Fase 0:  T1 → T2 → T3 → T4 → T5 → T6 → T7 → T8
Fase 1:  T9 → T10 → T11
Fase 2:  T12 → T13 → T14 → T15
Fase 3:  T16 → T17 → T18 → T19 → T20
Fase 4:  T21 → T22 → T23
Fase 5:  T24 → T25
Fase 6:  T26
```

As tarefas que dependem do autor numa máquina específica (T8 com WSL e partes de T9, T11 e T5) podem ficar bloqueadas sem impedir as seguintes da mesma fase, desde que o bloqueio esteja registrado em "Onde parou". As features (Fases 2 a 5) só começam com as decisões dos três spikes registradas.
