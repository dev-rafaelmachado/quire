# QUIRE v0.2 — Escrita segura, busca e command palette

**Status:** Execução · 0 de 7 fases completas · branch `feat/v0-2` · 5/28 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

Editar qualquer `.md` de uma spec dentro do Quire, com preview e escrita que nunca perde dados nem atropela o agente, mais busca full-text, command palette e checkbox do `tasks.md` opt-in, depois de fechar as dívidas da v0.1.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Dívidas da v0.1 | quire | 5/9 |
| 1: Spikes | quire | 0/3 |
| 2: Escrita segura no núcleo | quire | 0/5 |
| 3: Editor | quire | 0/5 |
| 4: Busca e command palette | quire | 0/3 |
| 5: Tasks e progresso | quire | 0/2 |
| 6: Fechamento da v0.2 | quire | 0/1 |

**Total:** 5/28 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** Fase 1, T9 (spike do CodeMirror 6)
- **Concluídas:** T1, T27, T2, T4, T7
- **Em andamento:** T9
- **Aguardando revisão visual:** T3 (`/tmp/quire-shots/v0-2/T3-*.png`)
- **Aguardando decisão do autor:** T6 (orçamento de memória do modo Dividir: ≤ 456 MB, ver "Medições")
- **Aguardando CI do Windows (só roda depois do push):** T5 (passo "console helper (Windows)" e job "msi install" do `ci.yml`)
- **NÃO VERIFICADO (depende de ambiente que não há aqui):** idioma pelo sistema no WebView2 (T27); T5 inteira no Windows: `bin\quire.cmd` em `cmd` e PowerShell, códigos de saída, fragmento WiX (`INSTALLDIR`, `$(sys.SOURCEFILEDIR)`), `PATH` depois do MSI, `quire .` abrindo o app sem prender o terminal
- **Próximo passo:** T9, depois T10
- **Pré-requisitos vindos da v0.1:** merge do PR da v0.1 feito (PR #1, `f256af3`). A tag `v0.1.0` e as revisões visuais de T14, T16, T18, T19, T20, T22 e T24 continuam pendentes na v0.1 e não bloqueiam a v0.2 (P10)
- **Bloqueios:** T8 (precisa do autor numa máquina com WSL: spike T2 da v0.1 no WSLg e T3 em WSL2 ext4, DrvFs e `\\wsl$`; a pasta `apps/desktop/src/spike/` só sai depois disso). Também dependem de máquina partes de T9 (teclas mortas/IME com o autor; WebView2) e T11 (WSL, DrvFs, 9p, `\\wsl$`)
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** `feat/v0-2`, criada de `docs/v0-2-spec` (que ainda não foi mergeada em `main`); sem push

## Pendências da v0.1 e destino na v0.2

Levantadas no Journal da v0.1 (`.specs/features/quire-v0-1/summary.md`):

| Pendência | Origem | Destino |
| --- | --- | --- |
| Catálogo de strings em inglês | A1 da spec v0.1, AGENTS.md | T1 (DEB-01); idioma padrão do sistema e seletor na T27 (DEB-10, P8) |
| Erros do backend em inglês na UI | T23 da v0.1, "limitação conhecida" | T1 (DEB-02) |
| Sugestão do `exban` só no backend | T9 da v0.1 (PST-01) | T2 (DEB-03) |
| Chips de requisito transbordando em janela estreita; filtros de gates quebrando em 1064 px | correção pós-T24, T19 da v0.1 | T3 (DEB-04) |
| `QUIRE_CONFIG` relativo grava em lugar errado | correção pós-T24 (4eed549) | T4 (DEB-05) |
| Console do CLI no Windows (P2) | T22 da v0.1 | T5 (DEB-06): binário auxiliar `quire-cli.exe` + `bin\quire.cmd` (P9) |
| ~390–418 MB de PSS com uma spec aberta | T2 e T16 da v0.1 | T6 (DEB-07) |
| `tauri-specta` em rc | T13 da v0.1 (TD-07) | T7 (DEB-08) |
| WSLg (render) e watcher em WSL/DrvFs/`\\wsl$` não medidos; polling no Windows sem teste em compartilhamento real | T2, T3, T21 da v0.1 | T8 (DEB-09) |
| Pasta `apps/desktop/src/spike/` ainda no repositório | T2 da v0.1 | T8 |
| Merge do PR da v0.1 | relatório da manhã da v0.1 | **feito** antes da v0.2 (PR #1, `f256af3`; P10) |
| Revisões visuais T14, T16, T18, T19, T20, T22, T24; tag `v0.1.0` | relatório da manhã da v0.1 | **continuam na v0.1**, sem bloquear a v0.2 (P10) |
| `.msi` não testado | relatório da manhã da v0.1 | T5: o CI do Windows passa a gerar e instalar o MSI |
| Passos E2E 19, 21, 34, 35 da v0.1 que pedem olho humano | T24 da v0.1 | continuam na revisão da v0.1, sem bloquear a v0.2 (P10) |
| Fontes Inter e JetBrains Mono não embutidas (fallback do sistema) | T14 da v0.1 | **fora da v0.2**: não atrapalha o uso; reavaliar com o tema na v0.3 |
| Detecção de unidade de rede mapeada no Windows (exige FFI) | T21 da v0.1 | **fora da v0.2**: o override `[watch.paths]` cobre; reavaliar junto com a decisão de FFI do TD-20 |
| Seção ativa do outline só depois de rolar até um título | T16 da v0.1 | **fora da v0.2**: cosmético |
| "Comparar selecionadas", "N commits à frente" no seletor | T20 da v0.1 | **fora da v0.2**: diff entre instâncias é Should de outra release |
| `arquivo:linha` e IDs clicáveis, `$EDITOR` | T18, T19 da v0.1 | **fora da v0.2**: Should fora do fatiamento |

## Backlog da v0.3 decidido na v0.2

- Criar spec ou arquivo novo, inclusive a partir de template (P1)
- Alerta de divergência entre o progresso do `tasks.md` e a tabela do `summary.md` (P5, ID PRG-02 reservado), junto do lint de estrutura

## Medições

### T6: memória com uma spec aberta (DEB-07)

**Como:** binário de produção (`tauri build`, 2026-11-16, commit `a69fae3`), Linux nativo (CachyOS, Hyprland 0.57.2, webkit2gtk-4.1 2.54.2, i5-1235U, Iris Xe), janela de 1440×900 numa saída headless do Hyprland, PSS somado do `quire` e dos processos WebKit filhos (`scripts/pss-tree.py`). Documento: `generateMarkdown(1)` do spike da v0.1 (42,8 KB, 6 diagramas Mermaid, blocos de código), numa pasta temporária fora do repositório, aberto com `quire open <spec.md>` no modo Ler. Amostra 15 s depois de abrir; 3 rodadas por cenário, cada uma com o app recém-aberto.

| Cenário | PSS total (MB) | só WebKitWebProcess (MB) |
| --- | --- | --- |
| App sem spec aberta | 257, 258, 258 | 136, 137, 137 |
| Spec de 40 KB, modo Ler | 395, 396, 447 | 276, 277, 328 |

**Comparação com a v0.1:** o spike da T2 (build de desenvolvimento, `vite preview`) mediu 239 MB vazio e ~390 MB com a mesma spec; a T16 da v0.1 mediu 400–418 MB. A base de produção da v0.2 fica no mesmo patamar (mediana de 396 MB), com uma rodada fora da curva em 447 MB.

**Orçamento proposto para o modo Dividir (editor + preview, T16 e T26):** base de 396 MB + 60 MB = **≤ 456 MB** de PSS total na mediana de 3 rodadas, mesmas condições. **Aguardando aprovação do autor**; até lá a T16 e a T26 usam esse número.

## Journal

- 2026-11-16 · planejamento · spec, design e tarefas da v0.2 escritos na branch `docs/v0-2-spec`. Escopo do pedido do autor: edição com preview (CodeMirror 6), escrita segura, conflito com agentes, busca full-text (FTS5), command palette, checkbox opt-in, progresso x/y, edição com várias instâncias, Fase 0 de dívidas e três spikes antes das features
- 2026-11-16 · planejamento · **progresso x/y (item 7 do pedido):** a v0.1 já entrega a barra e a contagem na sidebar (T15), a contagem na aba (T17), a faixa e as fases na aba tasks (T18) e os chips do cabeçalho (T17). O que é novo: atualizar o progresso a partir da própria escrita sem esperar o watcher (PRG-01, porque em polling isso levaria até 3 s) e, se o autor aprovar (P5), o alerta de divergência com a tabela do `summary.md` (PRG-02), Should do MoSCoW cujo parser já existe sem uso
- 2026-11-16 · planejamento · **decisões de design que o autor deve conferir:** a escrita fica inteira no núcleo (TD-12), com duas verificações de conflito por hash do conteúdo bruto (TD-13), sem confiar em `mtime`; a supressão da própria escrita compara impressões digitais, não janelas de tempo (TD-14), para não esconder uma escrita do agente logo depois do save; o FTS5 fica no mesmo cache SQLite (TD-15), já compilado no `libsqlite3-sys` embutido; `QUIRE_CONFIG` e `QUIRE_INDEX` relativos passam a ser recusados (TD-21). Nenhum TD da v0.1 foi reescrito; os novos vão de TD-11 a TD-21
- 2026-11-16 · planejamento · **perguntas que mudam escopo e ficaram para o autor** (spec, P1–P10): criar arquivo ou spec nova; autosave opcional; rascunhos após crash × regra do AGENTS.md; anotação ao marcar tarefa; alerta summary × tasks; o que fazer se o rename não for atômico em `\\wsl$`/DrvFs/9p; indexar JSONL na busca; idioma padrão; binário de console extra no Windows; o que da v0.1 precisa fechar antes
- 2026-11-16 · planejamento · **NÃO VERIFICADO no planejamento** (cabe aos spikes): CodeMirror no WebKitGTK/Wayland com teclas mortas e IME; tamanho e latência do FTS5 com 300 specs; atomicidade do rename em DrvFs, 9p, `\\wsl$` e NTFS com o arquivo aberto; preservação de atributos e ACLs no Windows depois do rename
- 2026-11-16 · planejamento · **decisões do autor sobre P1–P10 aplicadas** (todas as recomendações aceitas). Fora da v0.2 e no backlog da v0.3: criar arquivo/spec (P1) e o alerta summary × tasks (P5, PRG-02 removido com o ID reservado). Sem autosave nem rascunho (P2, P3; TD-24). Checkbox só muda o caractere (P4; TD-18). Busca só em markdown (P7; TD-15). Idioma do sistema com fallback em inglês e seletor (P8; TD-19, TD-23, DEB-10, T27 nova). Binário auxiliar de console no Windows (P9; TD-20, T5 reescrita). Merge antes, tag não bloqueia (P10; merge já feito em `f256af3`)
- 2026-11-16 · planejamento · **P6 virou regra condicionada ao dado** (SAV-11, TD-22, T28 nova): `\\wsl$`/`\\wsl.localhost`, DrvFs e 9p só aceitam escrita se a T11 registrar rename atômico verificado na classe; **classe sem dado fica somente leitura**. Consequência prática: como o autor não tem Windows, `\\wsl$` visto do Windows deve ficar somente leitura na v0.2, a menos que alguém rode o roteiro da T11 numa máquina Windows
- 2026-11-16 · planejamento · **IDs:** T27 (idioma) e T28 (escrita por classe de FS) foram criados sem renumerar as demais tarefas; entram em `T1 → T27 → T2` e `T13 → T28 → T14`. Critérios novos: SAV-11 e DEB-10. Removido: PRG-02. Rastreabilidade conferida: 64 critérios, todos com tarefa; 28 tarefas, todas com critério
- 2026-11-16 · planejamento · **NÃO VERIFICADO** (novo com as decisões): se `navigator.languages` segue o idioma do sistema no WebKitGTK e no WebView2 (T27); como DrvFs e 9p aparecem no `mountinfo` do WSL2 atual (T11, T28); paridade da ajuda do `clap` com o `tauri-plugin-cli`, fragmento WiX com `PATH` no Tauri 2.12 e código de saída do `.cmd` no PowerShell (T5)
- 2026-11-16 · T1 · catálogo `en` tipado pela forma do `pt-BR` (`Catalog<T>`: faltar ou sobrar chave não compila; teste confere também os `{parâmetros}`), `t()` lendo o idioma de um store Zustand que a raiz do app assina (troca sem reiniciar; o `SectionView` memoizado do markdown também assina), plurais/números/meses pelo idioma em uso. Erros com código estável: `ScanError::code`, `ConfigError::code`, `ScanPathError.code`, `ScanPathStatus.error` e `ConfigView.error` viraram `CmdError`; códigos de `set_config` ficaram específicos (`poll_interval_range`, `unknown_theme`, `config_*`). A UI mostra a tradução e o texto do backend só num `<details>` (`ErrorText`/`ErrorDetail`). Teste renderiza as Preferências em inglês e confere que nenhum trecho do `pt-BR` aparece. Gate de fase verde (269a782). **Decisão fora do design:** códigos de `set_config` renomeados de `invalid` para códigos específicos (o design só pedia `CmdError.code`)
- 2026-11-16 · T27 · idioma do sistema com fallback em inglês e seletor (bb0a0d0). `resolveLocale` testado por tabela (14 casos); `language` no `config.toml` lido como texto (valor desconhecido avisa nas Preferências e é regravado igual se o usuário salvar sem mexer). **Bug achado pelo teste e corrigido:** remover uma chave do `config.toml` (`set_or_remove`, também usado por `theme`) apagava o comentário escrito acima dela, que no `toml_edit` pertence ao prefixo da chave; agora o comentário passa para a chave seguinte (ou para o fim da tabela). Testes de vitest fixam `navigator.languages = ["pt-BR"]` num `setupFiles` para não depender do `LANG` da máquina. **Verificado no binário de produção (Linux):** `LANG=en_US.UTF-8` abre em inglês, `pt_BR` em português e `es_ES` em inglês (`/tmp/quire-shots/v0-2/T27-LANG-*.png`); Preferências com as linhas Tema e Idioma em `T27-prefs-dark.png` e `T27-prefs-light.png`. A primeira captura mostrou o seletor esticado e sem rótulo visível; corrigido antes do commit. **Decisão fora do design:** o idioma escolhido vale ao clicar em Salvar (como o tema), e não ao clicar no seletor. **NÃO VERIFICADO:** WebView2 (Windows). **Nota de captura:** o Hyprland daqui passou a usar configuração Lua (0.57.2) e uma janela num workspace especial oculto não pinta (a captura sai vazia); as capturas da v0.2 usam uma saída headless criada com `hyprctl output create headless QUIRESHOT` (sem tela física) e `grim -T` da janela
- 2026-11-16 · T2 · sugestão do preset visível (d6f2502): faixa na sidebar, abaixo do cabeçalho do projeto, com "Usar exban" (grava só o preset daquele projeto pelo `set_config`, desabilitado se o `config.toml` não for editável) e "Dispensar sugestão" (vale na sessão); nas Preferências, o mesmo atalho só preenche o rascunho, gravado no Salvar. Testes: nada é escrito sem o clique, dispensar não escreve, só o preset do projeto muda (`settingsWithPreset`). As fixtures versionadas têm preset explícito nos dois projetos, então a captura usou uma cópia temporária do config no scratchpad (`/tmp/quire-shots/v0-2/T2-suggestion-dark.png` e `-light.png`); o clique em si não foi feito na janela real (sem automação de clique aqui), só no teste de componente
- 2026-11-16 · T3 · chips em janela estreita (bb83a95); gate verde, **aguardando revisão visual**. `ChipList` mede a largura natural dos chips (régua invisível) e mostra os que cabem mais "+N", com a lista completa em `sr-only` e no `title`; o cálculo (`chipsThatFit`) tem teste próprio. A primeira captura em 1064 px mostrou um estouro que não era dos chips: a soma dos mínimos da linha (ícone, id, título, nota, hash, gate) passava da largura do card, e o chip "fase" saía da borda. Corrigido com container query no card da fase: abaixo de 520 px a coluna de gate some (o nível continua no painel da tarefa) e os espaços apertam; acima de 640 px o título ganha mínimo maior. Filtros da aba gates agrupados (rótulo + controle; contagem + somente leitura), então quebram em blocos. Capturas (caso real: `tasks.md` da v0.1 com o projeto `quire` em `exban`, por config temporário): `/tmp/quire-shots/v0-2/T3-tasks-1064-dark.png`, `T3-tasks-1064-light.png`, `T3-tasks-1440-dark.png`, `T3-tasks-1440-light.png`, `T3-gates-1064-dark.png`, `T3-gates-1064-light.png`. **Diferenças a revisar:** em 1064 px quase todos os chips viram só "+N", porque o card central fica com ~430 px ao lado do painel da tarefa; os filtros de gates ocupam três linhas nessa largura
- 2026-11-16 · T4 · `QUIRE_CONFIG`/`QUIRE_INDEX` relativos recusados (33af038). `AppState::from_env` (o `load` só lê as variáveis): config relativo vira erro `config_env_relative` traduzido, sem `path` e não editável (o app abre com os padrões); índice relativo ou dentro de `.specs` encerra a inicialização com a mensagem no stderr, como qualquer falha de abertura do índice. Teste do shell com nomes relativos únicos: nada é criado no diretório de trabalho, `set_config` recusa com o mesmo código, e caminhos absolutos continuam funcionando. **Decisão fora do design:** `set_config` com config inválida passa a devolver o código do motivo (antes era sempre `config_invalid`), para a UI dizer o que corrigir
- 2026-11-16 · T5 · binário auxiliar de console do Windows (a69fae3); gate verde, **aguardando CI do Windows**. Crate `quire-cli` (sem Tauri, sem `unsafe`): monta o `clap` a partir do `plugins.cli` do `tauri.conf.json` embutido, do mesmo jeito que o `tauri-plugin-cli` 2.5. **Dois detalhes achados pelo teste de paridade:** (1) o `Config::long_description()` do plugin devolve a própria descrição, o que liga o layout longo do `--help`; o auxiliar faz o mesmo de propósito; (2) o `clap` usa o `argv[0]` como nome, e o auxiliar se apresentava como `quire-cli`; agora usa sempre `quire`. Teste de paridade no shell (10 linhas de comando: ajuda, ajuda do subcomando, `help open`, flag inválida, `open` sem alvo, alvo a mais e as chamadas válidas) e diff dos binários reais no Linux: saída e código iguais. Chamada válida: inicia o `quire.exe` vizinho com `DETACHED_PROCESS` (API segura de `CommandExt`) e sai com 0; sem o app ao lado, mensagem e código 1. Empacotamento só no Windows: `tauri.windows.conf.json` com `externalBin` e o fragmento WiX `wix/quire-cli.wxs` (`bin\\quire.cmd` + `PATH` do sistema só com `bin`); `scripts/stage-quire-cli.sh` prepara o binário antes do `tauri build` (CI, release e CONTRIBUTING). `.gitattributes` força CRLF em `*.cmd`. `ci.yml`: passo que testa o `quire.cmd` em PowerShell e `cmd` (ajuda, versão, códigos 2 e 1) e job `msi` que gera, instala com `msiexec /qn` e roda `quire --version` pelo `PATH`; `actionlint` 1.7.12 limpo. `.deb` com a mesma lista de arquivos de antes. **NÃO VERIFICADO (sem Windows):** tudo que só roda no Windows, listado em Onde parou
- 2026-11-16 · T6 · memória medida no binário de produção (f945633, `scripts/pss-tree.py`): app vazio 257–258 MB, spec de 40 KB no modo Ler 395–447 MB (mediana 396), tabela em "Medições". Orçamento proposto para o modo Dividir: mediana ≤ 456 MB (base + 60 MB), **aguardando aprovação do autor**; a tarefa fica sem `[x]` só por isso. A medição usa uma saída headless do Hyprland em vez de janela visível; o efeito disso no PSS não foi medido separadamente. Captura de conferência: `/tmp/quire-shots/v0-2/T6-40kb-read.png`
- 2026-11-16 · T7 · verificado em 2026-11-16 na API do crates.io: não existe versão estável. Mais recentes: `tauri-specta` 2.0.0-rc.29 (2026-11-13), `specta` 2.0.0-rc.28 (2026-11-05), `specta-typescript` 0.0.15 (2026-11-05), exatamente as versões já fixadas. **Decisão:** manter `=2.0.0-rc.29` / `=2.0.0-rc.28` / `0.0.15` fixadas (TD-07 da v0.1 continua); o teste `bindings_are_up_to_date` segue protegendo contra mudança silenciosa. Reverificar antes da tag da v0.2
- 2026-11-16 · T8 · **bloqueada por ambiente:** não há WSL nesta máquina, e as rodadas pendentes (spike T2 da v0.1 no WSLg; T3 em WSL2 ext4, DrvFs `/mnt/c` e `\\\\wsl$` a partir do Windows) precisam do autor. A remoção de `apps/desktop/src/spike/` espera essas rodadas (DEB-09). Nada foi inferido. Sigo para a Fase 1, como o `tasks.md` permite para tarefas que dependem de máquina específica
