# QUIRE v0.2 — Escrita segura, busca e command palette

**Status:** Execução · 0 de 7 fases completas · branch `feat/v0-2` · 2/28 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

Editar qualquer `.md` de uma spec dentro do Quire, com preview e escrita que nunca perde dados nem atropela o agente, mais busca full-text, command palette e checkbox do `tasks.md` opt-in, depois de fechar as dívidas da v0.1.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Dívidas da v0.1 | quire | 2/9 |
| 1: Spikes | quire | 0/3 |
| 2: Escrita segura no núcleo | quire | 0/5 |
| 3: Editor | quire | 0/5 |
| 4: Busca e command palette | quire | 0/3 |
| 5: Tasks e progresso | quire | 0/2 |
| 6: Fechamento da v0.2 | quire | 0/1 |

**Total:** 2/28 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** Fase 0, T2 (sugestão do preset exban)
- **Concluídas:** T1, T27
- **Em andamento:** T2
- **Próximo passo:** T2, depois T3
- **Pré-requisitos vindos da v0.1:** merge do PR da v0.1 feito (PR #1, `f256af3`). A tag `v0.1.0` e as revisões visuais de T14, T16, T18, T19, T20, T22 e T24 continuam pendentes na v0.1 e não bloqueiam a v0.2 (P10)
- **Bloqueios:** nenhum de especificação. Dependem de máquina: T8 e partes de T9, T11 e T27 (autor com WSL ou Windows); T5 depende só do CI do Windows
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
