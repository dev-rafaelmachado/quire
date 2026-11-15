# QUIRE v0.1 — Leitura de specs em vários repositórios e worktrees

**Status:** Em execução · Fase 0 em andamento · 1/24 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

App desktop local (Tauri) que descobre as pastas de spec de vários repositórios e worktrees, deduplica e agrupa por projeto, e as exibe com visões por arquivo (markdown, tasks por fase, gates e lessons), atualizando ao vivo e abrindo pelo terminal.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Spike e fundação | quire | 1/4 |
| 1: Núcleo (`quire-core`) | quire | 0/8 |
| 2: Shell e base da UI | quire | 0/3 |
| 3: Visões | quire | 0/5 |
| 4: Ao vivo e CLI | quire | 0/2 |
| 5: Fechamento da v0.1 | quire | 0/2 |

**Total:** 1/24 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** Fase 0; T1 concluída, próxima é a T2 (spike do WebKitGTK)
- **Concluídas:** T1 (1d09a59)
- **Em andamento:** nada
- **Próximo passo:** T2: página de spike com markdown de ~40 KB, Mermaid e rolagem; medir render, rolagem e bundle (P1)
- **Bloqueios:** nenhum
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** `feat/monorepo-scaffold` (sem push)

## Journal

- 2026-11-15 · planejamento · ideação concluída: nome Quire, MoSCoW, wireframe v1 (três colunas, neutro moderno, escuro primeiro) e decisões fechadas em `docs/ideation/moscow.md`
- 2026-11-15 · planejamento · plataforma: Linux como alvo principal e Windows nativo em segundo, porque o time migra de Windows com WSL para Linux em cerca de um mês
- 2026-11-15 · planejamento · checkbox do `tasks.md` fica somente leitura na v0.1 (opt-in entra na v0.2); gates e lessons são sempre somente leitura
- 2026-11-15 · planejamento · fixtures sintéticas obrigatórias; nenhuma spec real entra no repositório público
- 2026-11-15 · planejamento · spec, design e tasks da v0.1 escritos (24 tarefas em 6 fases) a partir do MoSCoW; P1 e P2 da spec ficam abertas até o spike
- 2026-11-15 · T1 · scaffold do monorepo concluído (1d09a59): workspace Cargo (`quire-core` vazio e sem Tauri; `quire-desktop` com binário `quire`) e pnpm workspace com `apps/desktop`. Versões: Tauri 2.12 (crate 2.12.5, CLI/API 2.12.4), Vite 8.3, React 19.3, TypeScript 7.0 (tsc nativo), Tailwind 4.3 via `@tailwindcss/vite`, Vitest 5.1, Rust 1.100 (edition 2024)
- 2026-11-15 · T1 · a crate `tauri` mais nova no crates.io é 3.0 alpha; fixado `tauri = "2"` / `tauri-build = "2"` conforme o TD-01
- 2026-11-15 · T1 · o gate de fase passou com os comandos já escritos; o gate final também foi confirmado (o build do frontend precisa vir antes do `cargo build --release`). Notas registradas em "Comandos de gate" no `tasks.md`
- 2026-11-15 · T1 · janela vazia abre no Linux (CachyOS, webkit2gtk-4.1 2.54.2). A tela preta é o fundo `bg-neutral-950` do shell vazio; uma troca temporária de conteúdo confirmou que o WebKit renderiza e o HMR funciona, sem precisar de `WEBKIT_DISABLE_DMABUF_RENDERER`
- 2026-11-15 · T1 · extras pequenos: lints do workspace (`unsafe_code = forbid`, clippy `unwrap_used` com `allow-unwrap-in-tests`) para aplicar a regra "sem unwrap fora de testes" do AGENTS.md; ícone placeholder gerado de `icons/icon-source.svg`; CSP do Tauri continua `null` (padrão do template) e deve ser definida antes do release (T24)
