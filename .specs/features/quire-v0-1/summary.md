# QUIRE v0.1 — Leitura de specs em vários repositórios e worktrees

**Status:** Planejada, aguardando aprovação do autor · 0 de 6 fases · 0/24 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

App desktop local (Tauri) que descobre as pastas de spec de vários repositórios e worktrees, deduplica e agrupa por projeto, e as exibe com visões por arquivo (markdown, tasks por fase, gates e lessons), atualizando ao vivo e abrindo pelo terminal.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Spike e fundação | quire | 0/4 |
| 1: Núcleo (`quire-core`) | quire | 0/8 |
| 2: Shell e base da UI | quire | 0/3 |
| 3: Visões | quire | 0/5 |
| 4: Ao vivo e CLI | quire | 0/2 |
| 5: Fechamento da v0.1 | quire | 0/2 |

**Total:** 0/24 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** nenhuma iniciada; próxima é a T1
- **Concluídas:** nenhuma
- **Em andamento:** nada
- **Próximo passo:** aprovar a spec e o design; depois executar a Fase 0 (spike) antes de qualquer feature
- **Bloqueios:** nenhum
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** main

## Journal

- 2026-11-15 · planejamento · ideação concluída: nome Quire, MoSCoW, wireframe v1 (três colunas, neutro moderno, escuro primeiro) e decisões fechadas em `docs/ideation/moscow.md`
- 2026-11-15 · planejamento · plataforma: Linux como alvo principal e Windows nativo em segundo, porque o time migra de Windows com WSL para Linux em cerca de um mês
- 2026-11-15 · planejamento · checkbox do `tasks.md` fica somente leitura na v0.1 (opt-in entra na v0.2); gates e lessons são sempre somente leitura
- 2026-11-15 · planejamento · fixtures sintéticas obrigatórias; nenhuma spec real entra no repositório público
- 2026-11-15 · planejamento · spec, design e tasks da v0.1 escritos (24 tarefas em 6 fases) a partir do MoSCoW; P1 e P2 da spec ficam abertas até o spike
