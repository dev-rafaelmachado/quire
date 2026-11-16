# QUIRE v0.2 — Escrita segura, busca e command palette

**Status:** Planejamento · 0 de 7 fases completas · aguardando aprovação do autor e respostas às perguntas P1–P10 · 0/26 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

Editar qualquer `.md` de uma spec dentro do Quire, com preview e escrita que nunca perde dados nem atropela o agente, mais busca full-text, command palette e checkbox do `tasks.md` opt-in, depois de fechar as dívidas da v0.1.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Dívidas da v0.1 | quire | 0/8 |
| 1: Spikes | quire | 0/3 |
| 2: Escrita segura no núcleo | quire | 0/4 |
| 3: Editor | quire | 0/5 |
| 4: Busca e command palette | quire | 0/3 |
| 5: Tasks e progresso | quire | 0/2 |
| 6: Fechamento da v0.2 | quire | 0/1 |

**Total:** 0/26 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** planejamento concluído; nenhuma tarefa iniciada
- **Concluídas:** nenhuma
- **Em andamento:** nenhuma
- **Próximo passo:** o autor revisa spec, design e tarefas e responde P1–P10; depois, T1
- **Pré-requisitos vindos da v0.1:** revisões visuais de T14, T16, T18, T19, T20, T22 e T24; merge do PR da v0.1 em `main`; tag `v0.1.0` (o `release.yml` nunca rodou); ver P10
- **Bloqueios:** perguntas abertas P1–P10 da spec
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** `docs/v0-2-spec`, criada a partir de `feat/core-and-ipc` (sem push)

## Pendências da v0.1 e destino na v0.2

Levantadas no Journal da v0.1 (`.specs/features/quire-v0-1/summary.md`):

| Pendência | Origem | Destino |
| --- | --- | --- |
| Catálogo de strings em inglês | A1 da spec v0.1, AGENTS.md | T1 (DEB-01) |
| Erros do backend em inglês na UI | T23 da v0.1, "limitação conhecida" | T1 (DEB-02) |
| Sugestão do `exban` só no backend | T9 da v0.1 (PST-01) | T2 (DEB-03) |
| Chips de requisito transbordando em janela estreita; filtros de gates quebrando em 1064 px | correção pós-T24, T19 da v0.1 | T3 (DEB-04) |
| `QUIRE_CONFIG` relativo grava em lugar errado | correção pós-T24 (4eed549) | T4 (DEB-05) |
| Console do CLI no Windows (P2) | T22 da v0.1 | T5 (DEB-06) |
| ~390–418 MB de PSS com uma spec aberta | T2 e T16 da v0.1 | T6 (DEB-07) |
| `tauri-specta` em rc | T13 da v0.1 (TD-07) | T7 (DEB-08) |
| WSLg (render) e watcher em WSL/DrvFs/`\\wsl$` não medidos; polling no Windows sem teste em compartilhamento real | T2, T3, T21 da v0.1 | T8 (DEB-09) |
| Pasta `apps/desktop/src/spike/` ainda no repositório | T2 da v0.1 | T8 |
| Revisões visuais T14, T16, T18, T19, T20, T22, T24; tag `v0.1.0`; `.msi` não testado | relatório da manhã da v0.1 | pré-requisito (P10) |
| Passos E2E 19, 21, 34, 35 da v0.1 que pedem olho humano | T24 da v0.1 | pré-requisito (revisão da v0.1) |
| Fontes Inter e JetBrains Mono não embutidas (fallback do sistema) | T14 da v0.1 | **fora da v0.2**: não atrapalha o uso; reavaliar com o tema na v0.3 |
| Detecção de unidade de rede mapeada no Windows (exige FFI) | T21 da v0.1 | **fora da v0.2**: o override `[watch.paths]` cobre; reavaliar junto com a decisão de FFI do TD-20 |
| Seção ativa do outline só depois de rolar até um título | T16 da v0.1 | **fora da v0.2**: cosmético |
| "Comparar selecionadas", "N commits à frente" no seletor | T20 da v0.1 | **fora da v0.2**: diff entre instâncias é Should de outra release |
| `arquivo:linha` e IDs clicáveis, `$EDITOR` | T18, T19 da v0.1 | **fora da v0.2**: Should fora do fatiamento |

## Journal

- 2026-11-16 · planejamento · spec, design e tarefas da v0.2 escritos na branch `docs/v0-2-spec`. Escopo do pedido do autor: edição com preview (CodeMirror 6), escrita segura, conflito com agentes, busca full-text (FTS5), command palette, checkbox opt-in, progresso x/y, edição com várias instâncias, Fase 0 de dívidas e três spikes antes das features
- 2026-11-16 · planejamento · **progresso x/y (item 7 do pedido):** a v0.1 já entrega a barra e a contagem na sidebar (T15), a contagem na aba (T17), a faixa e as fases na aba tasks (T18) e os chips do cabeçalho (T17). O que é novo: atualizar o progresso a partir da própria escrita sem esperar o watcher (PRG-01, porque em polling isso levaria até 3 s) e, se o autor aprovar (P5), o alerta de divergência com a tabela do `summary.md` (PRG-02), Should do MoSCoW cujo parser já existe sem uso
- 2026-11-16 · planejamento · **decisões de design que o autor deve conferir:** a escrita fica inteira no núcleo (TD-12), com duas verificações de conflito por hash do conteúdo bruto (TD-13), sem confiar em `mtime`; a supressão da própria escrita compara impressões digitais, não janelas de tempo (TD-14), para não esconder uma escrita do agente logo depois do save; o FTS5 fica no mesmo cache SQLite (TD-15), já compilado no `libsqlite3-sys` embutido; `QUIRE_CONFIG` e `QUIRE_INDEX` relativos passam a ser recusados (TD-21). Nenhum TD da v0.1 foi reescrito; os novos vão de TD-11 a TD-21
- 2026-11-16 · planejamento · **perguntas que mudam escopo e ficaram para o autor** (spec, P1–P10): criar arquivo ou spec nova; autosave opcional; rascunhos após crash × regra do AGENTS.md; anotação ao marcar tarefa; alerta summary × tasks; o que fazer se o rename não for atômico em `\\wsl$`/DrvFs/9p; indexar JSONL na busca; idioma padrão; binário de console extra no Windows; o que da v0.1 precisa fechar antes
- 2026-11-16 · planejamento · **NÃO VERIFICADO no planejamento** (cabe aos spikes): CodeMirror no WebKitGTK/Wayland com teclas mortas e IME; tamanho e latência do FTS5 com 300 specs; atomicidade do rename em DrvFs, 9p, `\\wsl$` e NTFS com o arquivo aberto; preservação de atributos e ACLs no Windows depois do rename
