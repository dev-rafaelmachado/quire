# QUIRE v0.1 — Leitura de specs em vários repositórios e worktrees

**Status:** Em execução · 0 de 6 fases completas · Fase 0 em andamento · 6/24 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

App desktop local (Tauri) que descobre as pastas de spec de vários repositórios e worktrees, deduplica e agrupa por projeto, e as exibe com visões por arquivo (markdown, tasks por fase, gates e lessons), atualizando ao vivo e abrindo pelo terminal.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Spike e fundação | quire | 3/4 |
| 1: Núcleo (`quire-core`) | quire | 3/8 |
| 2: Shell e base da UI | quire | 0/3 |
| 3: Visões | quire | 0/5 |
| 4: Ao vivo e CLI | quire | 0/2 |
| 5: Fechamento da v0.1 | quire | 0/2 |

**Total:** 6/24 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** Fase 1; T7 concluída, próxima é a T8 (modelo de spec e instâncias)
- **Concluídas:** T1 (1d09a59), T2 (cbb0913), T3 (242c3d8), T5 (a3e2c35), T6 (199f344), T7 (a5a0b9b); T4 escrita, aguardando CI
- **Em andamento:** nada
- **Próximo passo:** T8: modelo de spec e instâncias com deduplicação
- **Pendências do spike T2:** rodada no WSLg não feita (máquina de desenvolvimento é Linux nativo); o spike continua isolado em `apps/desktop/src/spike/` para ser rodado lá (ver README da pasta) e deve ser apagado quando a T16 entrar
- **Pendências do spike T3 (WSL, não verificado):** rodar `scripts/watch-spike.sh` em WSL2 ext4, em DrvFs (`/mnt/c`) e o `cargo run` nativo do Windows sobre `\\wsl$` e NTFS, mais o modo `--observe` com escrita do outro lado (roteiro em `crates/quire-core/examples/README.md`). Colar as tabelas em "Resultados dos spikes › T3" e confirmar ou ajustar a regra do TD-05 antes da T21
- **Bloqueios:** nenhum
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** `feat/core-and-ipc` (sem push)
- **Pendências da T4:** o workflow de CI (cfd7d06) não rodou no GitHub. Para fechar a T4 é preciso fazer o push, abrir um PR de teste e ver o job verde em Linux e Windows; também conferir se uma falha de `clippy` ou de `typecheck` derruba o job. O Windows nunca foi testado localmente

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

### T3: watcher e custo de scan (2026-11-15)

**Ambiente:** Linux nativo (CachyOS, kernel 7.3.3), i5-1235U, NVMe. **O filesystem desta máquina é btrfs** (`/home`, zstd), não ext4: não há partição ext4 e montar uma imagem precisaria de root. Medido em btrfs (onde ficam os repos) e em tmpfs (`/tmp`) como comparação. O inotify fica na camada VFS, então o ext4 deve se comportar igual, mas isso não foi medido. `inotify max_user_watches` = 524288. Spike em `crates/quire-core/examples/watch_spike.rs`, rodado com `scripts/watch-spike.sh` (build release).

**Eventos nativos** (`notify` 8.2.2, backend inotify, 20 repetições por operação). Os resultados de btrfs e tmpfs foram iguais, com diferenças menores que 0,1 ms:

| FS | Operação | Chegou | Latência p50 | Latência máx | Eventos no alvo |
| --- | --- | ---: | ---: | ---: | --- |
| btrfs | criar arquivo | 20/20 | 0.15 ms | 0.28 ms | access, close-write, create, modify:data |
| btrfs | editar in-place (truncate + write) | 20/20 | 0.15 ms | 0.23 ms | access, close-write, modify:data |
| btrfs | rename atômico (tmp + rename) | 20/20 | 0.20 ms | 0.25 ms | rename:both, rename:to |
| btrfs | remover arquivo | 20/20 | 0.14 ms | 0.30 ms | remove |
| btrfs | criar pasta | 20/20 | 0.16 ms | 0.32 ms | access, create |
| btrfs | arquivo em pasta recém-criada (mesmo instante) | 0/20 | — | — | |
| btrfs | arquivo em pasta criada há 50 ms | 20/20 | 50.45 ms* | 50.58 ms* | access, close-write, create, modify:data |
| btrfs | remover pasta | 20/20 | 0.17 ms | 0.23 ms | remove |
| tmpfs | (as 8 operações) | igual ao btrfs, inclusive 0/20 no arquivo em pasta recém-criada | 0.11–0.19 ms | ≤ 0.33 ms | iguais |

\* A latência inclui os 50 ms de espera entre criar a pasta e o arquivo.

**Polling** (2000 arquivos em 50 repos sintéticos, só `.specs`):

| FS | Modo | Intervalo | Tempo por ciclo (wall) | CPU por ciclo | CPU média | Latência de detecção |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| btrfs | stat-walk síncrono (mtime + tamanho) | 1 / 2 / 5 s | 3.37 ms (máx 4.45) | 3.27 ms | 0.33% / 0.16% / 0.07% | ≤ intervalo |
| btrfs | notify `PollWatcher` (scan inicial 4–5 ms) | 1 s | — | — | 0.48% | 693 ms |
| btrfs | notify `PollWatcher` | 2 s | — | — | 0.26% | 680 ms |
| btrfs | notify `PollWatcher` | 5 s | — | — | 0.09% | 2172 ms |
| tmpfs | stat-walk síncrono | 1 / 2 / 5 s | 2.68 ms (máx 3.47) | 2.68 ms | 0.27% / 0.13% / 0.05% | ≤ intervalo |
| tmpfs | notify `PollWatcher` | 1 / 2 / 5 s | — | — | 0.42% / 0.24% / 0.11% | 694 / 684 / 2179 ms |

A latência do `PollWatcher` é de uma amostra por intervalo; o limite teórico é o próprio intervalo.

**Varredura com `ignore`** (50 repos, 20.295 arquivos, 4.601 diretórios; cada repo tem `.git`, `node_modules`, `target` e `dist` com uma pasta `.specs` de isca em cada; 1 repo em cada 10 sem `.gitignore`). "Frio" é 1 execução depois de `drop_caches`; "quente" é a mediana de 5:

| FS | Varredura | Cache | Entradas visitadas | `.specs` achadas | Iscas reportadas | Tempo |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| btrfs | `ignore` sequencial | frio | 3096 | 50 | 0 | 57.3 ms |
| btrfs | `ignore` paralelo | frio | 3096 | 50 | 0 | 8.7 ms |
| btrfs | sem regras de ignore (baseline) | frio | 24895 | 200 | 150 | 119.7 ms |
| btrfs | `ignore` sequencial | quente | 3096 | 50 | 0 | 3.5–3.8 ms |
| btrfs | `ignore` paralelo | quente | 3096 | 50 | 0 | 4.0–4.2 ms |
| btrfs | sem regras de ignore (baseline) | quente | 24895 | 200 | 150 | 12.9–25.2 ms |
| tmpfs | `ignore` sequencial / paralelo / baseline | quente | 3096 / 3096 / 24895 | 50 / 50 / 200 | 0 / 0 / 150 | 4.0 / 3.0 / 10.5 ms |

**Custo de setup do watch nativo:**

| FS | Alvo do watch nativo | Diretórios observados | Tempo de setup |
| --- | --- | ---: | ---: |
| btrfs | raiz dos repos, recursivo | 4601 | 72.6–90.8 ms |
| btrfs | só as pastas `.specs`, recursivo | 250 | 4.2–8.1 ms |
| tmpfs | raiz / só `.specs` | 4601 / 250 | 41.7 / 4.5 ms |

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
- 2026-11-15 · T3 · spike do watcher e do scan concluído **só na parte medida** (242c3d8). As tabelas estão em "Resultados dos spikes › T3". O FS daqui é btrfs, não ext4 como a T3 previa; medi em btrfs e em tmpfs
- 2026-11-15 · T3 · o backend nativo entregou 100% dos eventos com latência abaixo de 0,35 ms em criar, editar in-place, rename atômico, remover e criar/remover pasta. O rename atômico chega como `rename:to`/`rename:both` no arquivo final, sem `remove`, então "rename atômico não vira removido" (T21) é viável tratando o rename como mudança do destino e ignorando o tmp
- 2026-11-15 · T3 · **achado para a T21:** um arquivo criado logo depois da pasta que o contém **perde o evento sempre** (0/20, nos dois FS): o inotify só passa a observar a pasta nova depois do evento de criação dela. Com 50 ms de intervalo, chega sempre. Como agentes criam pastas de spec com arquivos de uma vez, ao receber `create` de uma pasta o watcher precisa **varrer o conteúdo dela** em vez de confiar nos eventos dos filhos. Não contradiz o design; é detalhe de implementação do `watch`
- 2026-11-15 · T3 · **achado para a T21:** observar o scan path inteiro de forma recursiva registra um watch por diretório, incluindo `node_modules`, `target` e `.git` (o `notify` não respeita `.gitignore`): 4.601 watches e 73–91 ms nos repos sintéticos, contra 250 watches e 4–8 ms observando só as pastas `.specs`. Repos reais com `node_modules` têm dezenas de milhares de diretórios e podem esgotar `max_user_watches`. Recomendação: watch recursivo só nas pastas `.specs` achadas pelo scan, watch não recursivo na raiz de cada repo (para pegar a criação de `.specs`) e reescanear os scan paths sob demanda ou na inicialização para achar repos novos
- 2026-11-15 · T3 · o polling é barato nesta escala: 2000 arquivos custam ~3,4 ms de CPU por ciclo (stat de mtime + tamanho), ou seja ~0,1% de CPU no intervalo padrão de 3 s. O `PollWatcher` do `notify` custa ~1,5× isso (0,48% em 1 s) e detectou dentro do intervalo. O padrão de 3 s do WAT-03 se mantém em Linux nativo; o custo no 9p/DrvFs não foi medido e deve ser maior
- 2026-11-15 · T3 · a varredura com `ignore` + `filter_entry` (pula `node_modules`, `target`, `dist`, `.git` por nome, além do `.gitignore`, com `hidden(false)` para enxergar `.specs`) achou as 50 specs e nenhuma das 150 iscas, inclusive nos repos sem `.gitignore`. Leva 3,5 ms com cache quente e 57 ms (sequencial) ou 8,7 ms (paralelo) com cache frio, contra 120 ms sem as regras. Para a T6: usar o walker paralelo, que ganha 6,6× no cache frio e empata no quente
- 2026-11-15 · T3 · **regra proposta para o TD-05** (detecção de "sem eventos"), decidindo por scan path na inicialização: (1) no Linux, ler o tipo do mount em `/proc/self/mountinfo`, como o spike já faz, e usar polling se for `9p`, `v9fs`, `drvfs`, `cifs`, `smb3`, `nfs`, `nfs4` ou `fuse.*` (sshfs e similares); `ext4`, `btrfs`, `xfs`, `tmpfs` e afins usam o backend nativo; (2) no Windows, usar polling para caminhos `\\wsl$\` e `\\wsl.localhost\`, qualquer UNC `\\servidor\share` e unidades mapeadas de rede; (3) override por config para cada scan path (`watch = "auto" | "native" | "poll"`); (4) fallback em tempo de execução: se o `watch()` nativo falhar (ex.: `ENOSPC` por limite de watches), cair para polling naquele path, com aviso no rodapé. **Sem sonda de escrita**: criar arquivo de teste violaria "escrita só por ação explícita", e o spike mostra que a escrita do mesmo lado sempre gera evento, então a sonda não detectaria o caso que importa (escrita do outro lado da fronteira WSL/Windows)
- 2026-11-15 · T3 · **NÃO VERIFICADO por falta de WSL** (nenhum resultado foi inferido): entrega de eventos do `notify` em WSL2 ext4, em DrvFs/9p (`/mnt/c`) e em `\\wsl$` a partir do Windows nativo; entrega quando quem escreve está do outro lado (editor do Windows em arquivo do WSL, agente do WSL em `C:\`); custo do polling e da varredura em 9p/DrvFs e em `\\wsl$`; qual `fstype` o `/proc/self/mountinfo` mostra para `/mnt/c` nas versões atuais do WSL (a regra cobre `9p` e `drvfs`); detecção de UNC e de unidade de rede no Windows; comportamento em NTFS nativo. Também não medido: ext4 em si (medi btrfs e tmpfs). O roteiro está em `crates/quire-core/examples/README.md`
- 2026-11-15 · planejamento · achados do spike T3 copiados para o "Pronto quando" da T21 (varrer o conteúdo de pasta recém-criada; watch nativo só nas pastas `.specs` e na raiz de cada repo, sem recursão)
- 2026-11-15 · dev · `feat/monorepo-scaffold` já estava mergeada na `main` (fast-forward); T4–T13 seguem na branch nova `feat/core-and-ipc`
- 2026-11-15 · T4 · workflow `.github/workflows/ci.yml` escrito (cfd7d06): matriz `ubuntu-latest` e `windows-latest`, dependências do Tauri no Linux, caches de Cargo e pnpm, e os mesmos comandos do gate de fase. Validado com `actionlint` 1.7.12 (binário oficial, checksum conferido) e com os comandos rodados localmente no Linux. Actions: checkout v7, setup-node v7 (Node 26), pnpm/action-setup v6, dtolnay/rust-toolchain stable, Swatinem/rust-cache v2. Também adicionado `.gitattributes` com `eol=lf` para o `cargo fmt --check` não variar no Windows. **Não marcada**: depende de push e do workflow verde nos dois sistemas
- 2026-11-15 · T5 · fixtures sintéticas concluídas (a3e2c35): `fixtures/templates` (`acme_gateway` com origin, `acme_billing` sem origin) e `scripts/make-fixtures.sh [saída]`, que monta repos, worktrees e iscas e é idempotente e determinístico (hashes `390c2e4`/`da2aa85`/`e305867` iguais entre gerações). Cobre spec idêntica em 3 instâncias, spec divergente com cópia mais nova numa worktree fora do scan path, specs parciais, JSONL com linha inválida e truncada, repo sem origin e a mesma chave em dois projetos. O script isola o git da config do usuário e só apaga diretório com o marcador `.quire-fixtures`
- 2026-11-15 · T6 · scanner concluído (199f344): `scan::scan(paths, filtro, callback)` com o walker paralelo do `ignore`, eventos `Found`/`PathError` em streaming e deduplicação pelo caminho canônico (já cobre scan paths sobrepostos). Como os presets só chegam na T9, o scanner recebe o filtro de arquivos primários (padrão: regra do `generic`) e devolve a lista de arquivos de cada pasta. Os testes de integração geram as fixtures com `scripts/make-fixtures.sh` (no Windows, preferem o Git Bash, porque o `bash` do PATH pode ser o do WSL). Os testes de integração usam `#![allow(clippy::unwrap_used)]` no topo, porque o `allow-unwrap-in-tests` não cobre funções auxiliares nesses crates
- 2026-11-15 · T7 · identidade do projeto via git concluída (a5a0b9b): `git::GitResolver` com timeout de 5 s por chamada, `GIT_OPTIONAL_LOCKS=0` e cache por repo. A chave é o common dir canônico (`rev-parse --path-format=absolute`, git ≥ 2.31). O nome vem do `origin` com fallback para a pasta do repo principal; branch por `symbolic-ref` (HEAD destacada vira `None`); worktree ligada quando git dir ≠ common dir. `scan::scan_with_worktrees` varre as worktrees fora dos scan paths (DSC-03): nas fixtures, as 3 specs de `external/` aparecem e o total vai a 12 instâncias. Sem git, o erro é tipado (`GitError::NotInstalled`) e o nome cai para a pasta. O alias configurável do DUP-01 fica para a T9, junto com o `config.toml`
