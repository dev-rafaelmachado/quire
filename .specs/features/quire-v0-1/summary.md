# QUIRE v0.1 — Leitura de specs em vários repositórios e worktrees

**Status:** Em execução · Fase 0 em andamento · 2/24 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

App desktop local (Tauri) que descobre as pastas de spec de vários repositórios e worktrees, deduplica e agrupa por projeto, e as exibe com visões por arquivo (markdown, tasks por fase, gates e lessons), atualizando ao vivo e abrindo pelo terminal.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Spike e fundação | quire | 2/4 |
| 1: Núcleo (`quire-core`) | quire | 0/8 |
| 2: Shell e base da UI | quire | 0/3 |
| 3: Visões | quire | 0/5 |
| 4: Ao vivo e CLI | quire | 0/2 |
| 5: Fechamento da v0.1 | quire | 0/2 |

**Total:** 2/24 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** Fase 0; T1 e T2 concluídas, próxima é a T3 (spike do watcher e do custo de scan)
- **Concluídas:** T1 (1d09a59), T2 (cbb0913)
- **Em andamento:** nada
- **Próximo passo:** T3: binário `crates/quire-core/examples/watch_spike.rs` medindo `notify` e polling; tabela FS × eventos × latência
- **Pendências do spike T2:** rodada no WSLg não feita (máquina de desenvolvimento é Linux nativo); o spike continua isolado em `apps/desktop/src/spike/` para ser rodado lá (ver README da pasta) e deve ser apagado quando a T16 entrar
- **Bloqueios:** nenhum
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** `feat/monorepo-scaffold` (sem push)

## Resultados dos spikes

### T2: render no WebKitGTK (2026-11-15)

**Ambiente:** Linux nativo (CachyOS, Hyprland/Wayland), i5-1235U, Iris Xe, 60 Hz, webkit2gtk-4.1 2.54.2. Build de produção do frontend servido pelo `vite preview` dentro do binário Tauri de debug (o render é todo do WebKit, então o modo do Rust não muda os números de render). WSLg: **não medido**, porque não há WSL nesta máquina.

**Documento:** markdown sintético de 42,8 KB (10 seções, 7 tabelas, 20 blocos com realce, 7 Mermaid sendo 1 inválido de propósito, listas de tarefas, diagrama ASCII), cerca de 3,3 mil nós DOM e 24 mil px de altura. Teste de estresse com 3× (127 KB, 21 Mermaid, 9,7 mil nós, 72 mil px).

Tempos em ms desde o início da navegação, com o app recém-aberto (cold):

| Medida | 40 KB | 127 KB |
| --- | ---: | ---: |
| parse (unified + remark-gfm + rehype) | 70–110 | 165–175 |
| primeiro paint, com código ainda sem realce (FCP) | 215–300 | 325–415 |
| realce completo (Shiki, engine JS) | ~1 800 | ~2 000 |
| realce completo (Shiki, engine Oniguruma WASM) | ~1 150 | — |
| Mermaid completo (layout ELK padrão / dagre) | 2 680 / 2 320 | 5 230 / 3 980 |
| primeiro diagrama, incluindo carregar os chunks | 1 500–1 800 | 1 650–1 900 |
| diagramas seguintes (p50) | 95–125 | 95–190 |
| recolher todas as seções / expandir todas | 16 / 35 | 25 / 75–95 |

Com cache quente (recargas no mesmo processo), o realce termina em ~0,4 s e o Mermaid em ~1,0 s para 40 KB.

**Rolagem:** foi programada, de 120 px e de 40 px por frame, do topo ao fim do documento. Em todas as 14 rodadas o frame time médio ficou em 16,5–16,7 ms (60 Hz), com p95 de 17–18 ms, p99 ≤ 22 ms e no máximo 1 frame acima de 34 ms por rodada (máximo de 55 ms), igual nos dois tamanhos. A checagem manual do autor (trackpad, rolagem rápida e lenta, recolher e expandir) no documento de 127 KB foi **fluida**. O WebKitGTK não suporta `longtask` no `PerformanceObserver`, então só temos o frame time por `requestAnimationFrame`.

**Bundle (build de produção, P1):**

| Parte | Bruto | gzip |
| --- | ---: | ---: |
| app vazio da T1 (React) | 220 KB | 69 KB |
| inicial com o pipeline de markdown (React + unified/remark/rehype) | 393 KB | 121 KB |
| Shiki sob demanda: core, engine JS, tema e 6 linguagens | 425 KB | 84 KB |
| alternativa: engine Oniguruma (WASM inline) em vez do JS | +212 / −58 KB | +159 / −20 KB |
| Mermaid carregado para este documento, layout dagre | 1 120 KB | 312 KB |
| Mermaid carregado para este documento, layout ELK (padrão do Mermaid 12) | 2 543 KB | 746 KB |
| `dist` inteiro (todos os tipos de diagrama do Mermaid) | 5 868 KB | 1 654 KB |

**Memória** (PSS somado do `quire` e dos processos WebKit filhos, 5 s depois de tudo renderizado; aos 35 s cai só ~10 MB):

| Cenário (40 KB, salvo indicação) | PSS total | só WebKitWebProcess |
| --- | ---: | ---: |
| página vazia (baseline) | 239 MB | 137 MB |
| markdown sem realce nem Mermaid | 294 MB | 193 MB |
| idem, 127 KB | 333 MB | 233 MB |
| + Shiki (engine JS) | 461 MB | 367 MB |
| + Shiki (JS) + Mermaid | 554–579 MB | 458–480 MB |
| + Shiki (Oniguruma) + Mermaid dagre | 389 MB | ~290 MB |

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
- 2026-11-15 · T2 · spike do WebKitGTK concluído (cbb0913). Números completos em "Resultados dos spikes › T2". Medido só em Linux nativo; WSLg pendente (o spike fica isolado em `apps/desktop/src/spike/`, fora do build do app e da varredura do Tailwind, para ser rodado lá)
- 2026-11-15 · T2 · **decisão: render direto, sem virtualização por seção.** A rolagem fica em 60 Hz com p95 ≤ 18 ms até em 127 KB e 9,7 mil nós DOM, recolher/expandir custa < 100 ms, e o autor confirmou fluidez na rolagem manual. O risco "WebKitGTK lento" do design não se confirmou. Reavaliar só se aparecer spec real bem acima de 130 KB
- 2026-11-15 · T2 · **P1 respondida:** o bundle inicial com o pipeline de markdown é de 393 KB (121 KB gzip), +173 KB sobre o app vazio. Shiki (~425 KB) e Mermaid (~1,1 MB com dagre, ~2,5 MB com ELK) só carregam quando o documento tem código ou diagrama. O `dist` total é de 5,9 MB, e como os assets são locais no binário Tauri, o tamanho pesa no tempo de parse/compilação do JS, não em rede. Aceitável
- 2026-11-15 · T2 · meta para a T16 ("40 KB rola sem travar"): na rolagem programada do documento de 40 KB, p95 de frame ≤ 20 ms e no máximo 1 frame > 34 ms; primeiro paint < 400 ms; realce e diagramas completos < 2,5 s no cold start
- 2026-11-15 · T2 · recomendações para a T16, sem contradizer o design (o TD-04 não fixa engine nem layout): (1) Mermaid com `layout: "dagre"` por padrão, porque o Mermaid 12 usa ELK por padrão e isso puxa 1,46 MB a mais e deixa os diagramas 15–25% mais lentos; diagramas que pedirem ELK explicitamente continuam funcionando; (2) Shiki com engine Oniguruma (WASM) em vez do engine JS: −190 MB de PSS e realce ~35% mais rápido neste documento, ao custo de +150 KB de bundle sob demanda. Ao definir a CSP (T24), o WASM exige `'wasm-unsafe-eval'`
- 2026-11-15 · T2 · ponto de atenção (não bloqueia): mesmo na configuração recomendada, uma spec de 40 KB aberta leva o app a ~390 MB de PSS, contra 239 MB vazio. O maior custo é do Shiki e do Mermaid já carregados, não do tamanho do documento. Vale medir de novo na T16 com o app real e com várias specs abertas em sequência
- 2026-11-15 · T2 · o Mermaid 12.3 detecta o diagrama inválido e o mostra com a mensagem de erro sem quebrar a página (VIW-03 viável com `suppressErrorRendering: true`)
