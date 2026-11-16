# QUIRE v0.3 — Referências, lint, histórico e export

**Status:** Especificação escrita, aguardando as decisões do autor (Q1–Q10) e aprovação · 0 de 8 fases · branch `docs/v0-3-spec` · 0/23 tarefas
**Spec:** `spec.md` · **Design:** `design.md` · **Tarefas:** `tasks.md` · **Idioma:** pt-BR

## Em uma frase

Seguir um critério da spec até a tarefa, a evidência, o gate e o diff do commit sem sair do Quire, com backlinks pela gramática do preset, lint de estrutura no app e no CI (`quire lint`), histórico git somente leitura, export HTML e PDF sem vazamento e criação de arquivo ausente, depois de fechar as dívidas de WSL, assinatura do MSI e fontes.

## Fases

<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 0: Dívidas da v0.2 | quire | 0/4 |
| 1: Spikes | quire | 0/4 |
| 2: Referências e navegação | quire | 0/5 |
| 3: Lint | quire | 0/3 |
| 4: Histórico e diff via git | quire | 0/3 |
| 5: Export | quire | 0/2 |
| 6: Criar arquivo ausente | quire | 0/1 |
| 7: Fechamento da v0.3 | quire | 0/1 |

**Total:** 0/23 tarefas
<!-- progress:end -->

## Onde parou

- **Fase / tarefa:** especificação (spec, design, tasks e summary) escrita; nenhuma tarefa iniciada
- **Concluídas (`[x]`):** nenhuma
- **Em andamento:** nenhuma
- **Aguardando decisão do autor:** Q1–Q10 da spec. Cada uma tem padrão conservador, então nenhuma bloqueia o início, exceto se o autor quiser mudar o escopo (Q3, Q4, Q5 e Q6 mudam o que é entregue)
- **Próximo passo:** o autor revisa a spec e responde às perguntas; depois, aprovação e execução a partir da Fase 0 (T2, T3, T4; a T1 é do autor com WSL)
- **Pré-requisitos vindos da v0.2:** nenhum bloqueante pelo padrão da Q9. Ficam na v0.2: revisões visuais, roteiro E2E, roteiro de teclas mortas e IME, `quire .` no Windows, orçamento de memória da T6 e a decisão de publicar a `v0.2.0-rc.1` ou fechar a `v0.2.0`
- **Bloqueios:** nenhum para começar. A T1 depende do autor numa máquina com WSL e não bloqueia as outras fases (Q7)
- **Arquivos sem commit:** nenhum
- **Branch / worktree:** `docs/v0-3-spec`, criada de `main` em `13523e6`. Esse commit, com o registro da `v0.2.0-rc.1` no summary da v0.2, ainda não foi enviado ao `origin`. Sem push, sem PR, sem tag

## Pendências da v0.2 e destino na v0.3

Levantadas no Relatório da manhã, na tabela de pendências, no "Onde parou" e no Journal da v0.2 (`.specs/features/quire-v0-2/summary.md`):

| Pendência | Origem | Destino |
| --- | --- | --- |
| Rodadas no WSL (spikes T2 e T3 da v0.1) e remoção de `apps/desktop/src/spike/` | T8 da v0.2 (bloqueada sem WSL) | **T1** (DEB-11), Fase 0, do autor; não bloqueia (Q7) |
| Escrita em `\\wsl$`, DrvFs e 9p (P6 da v0.2; `VERIFIED_ATOMIC` vazia) | T11 da v0.2 | **T1**: a decisão sai do dado; sem dado, as classes seguem somente leitura (TD-22) |
| MSI sem assinatura (aviso do SmartScreen) | release `v0.2.0-rc.1` | **T2** (DEB-12): avaliar e decidir; padrão: não assinar e documentar (Q6) |
| Fontes Inter e JetBrains Mono não embutidas | T14 da v0.1, adiada na v0.2 | **T3** (DEB-13): agora é necessária, porque o PDF precisa sair igual no WebKitGTK e no WebView2 (EXP-06) |
| Seção ativa do outline só depois de rolar até um título | T16 da v0.1, adiada na v0.2 (cosmético) | **T4** (DEB-14): o voltar/avançar restaura a rolagem e precisa do destaque correto |
| `arquivo:linha` e IDs clicáveis | T18 e T19 da v0.1 | **T11, T12** (REF-04, REF-09) |
| Abrir no `$EDITOR` na linha | T18 e T19 da v0.1 | **Backlog** (Should fora do fatiamento); a v0.3 abre no aplicativo padrão e copia `caminho:linha` |
| "Comparar selecionadas" no seletor de instâncias | T20 da v0.1, wireframe 4 | **T18** (HIS-07) |
| "N commits à frente" no seletor | T20 da v0.1 | **Backlog**: não serve ao critério de pronto da v0.3 |
| Detecção de unidade de rede mapeada no Windows | T21 da v0.1, adiada na v0.2 | **Descartada da v0.3**: exige FFI ou dependência com `unsafe` encapsulado (o motivo que descartou a opção (b) do TD-20); `[watch.paths]` cobre. Reavaliar só com feedback de uso no Windows |
| Criar spec ou arquivo novo (P1 da v0.2) | backlog decidido na v0.2 | **T21** (NEW) para arquivo primário ausente; spec nova depende da Q4 (padrão: backlog da v0.4) |
| Alerta `tasks.md` × tabela do `summary.md` (P5 da v0.2, PRG-02 reservado) | backlog decidido na v0.2 | **T13**: regra `progress-mismatch`, com o ID PRG-02 reservado |
| `parse::summary_progress` sem uso | design da v0.2 | **T13** |
| Revisões visuais T3, T16–T20, T22–T24 e E2E T26 da v0.2 | Relatório da manhã da v0.2 | **Continuam na v0.2**, sem bloquear (Q9) |
| Teclas mortas e IME no Wayland (T9); `quire .` sem prender o terminal no Windows (T5) | Onde parou da v0.2 | **Continuam na v0.2** (manual, do autor) |
| Orçamento de memória do modo Dividir (≤ 456 MB) | T6 da v0.2 | **Continua na v0.2**, aguardando o autor (Q9) |
| Publicar a `v0.2.0-rc.1` ou fechar a `v0.2.0` | release de teste de 2026-11-16 | **Autor** (Q9) |
| `tauri-specta` e `specta` em rc | DEB-08 da v0.2 | **T23**: nova verificação, como na T7 da v0.2 |
| ACLs do Windows no rename (TD-12) | "Não verificado" do design da v0.2 | **Continua risco**, sem tarefa: nenhum critério da v0.3 depende disso, e o CI não mostra problema |
| Revisões visuais e tag `v0.1.0` da v0.1 | v0.1 | **Continuam na v0.1** |

## Backlog da v0.4 decidido na v0.3

Valem até o autor decidir o contrário nas perguntas da spec:

- Criar spec nova, com pasta, chave, slug, repositório e worktree (Q4)
- Export de `gates.jsonl` e `lessons.jsonl` com caminhos mascarados (Q5)
- Forma qualificada de ID entre specs (Q1) e supressão do lint dentro do arquivo (Q2), se o autor quiser mudar o formato
- Abrir `arquivo:linha` no `$EDITOR` na linha; "N commits à frente/atrás"; detecção de unidade mapeada no Windows
- Correção automática pelo lint
- Should restantes do MoSCoW: journal com quick-add, copiar contexto para um agente, indicadores de git por projeto

## Medições

Ainda nenhuma. Cada spike e a T1/T2 preenchem a sua seção.

### T1: rodadas no WSL (DEB-11)

Mesma tabela da T11 da v0.2. As linhas de btrfs, tmpfs, ext4 e NTFS já estão medidas lá.

| Sistema de arquivos | Onde | Leituras concorrentes / ruins | Modo depois do rename | Alvo somente leitura | Link simbólico | Arquivo aberto | Eventos |
| --- | --- | --- | --- | --- | --- | --- | --- |
| DrvFs (`/mnt/c` no WSL) | máquina do autor | **NÃO VERIFICADO** | | | | | |
| 9p (WSL ↔ Windows) | máquina do autor | **NÃO VERIFICADO** | | | | | |
| `\\wsl$` / `\\wsl.localhost` (Windows → WSL) | Windows com WSL | **NÃO VERIFICADO** | | | | | |

### T2: assinatura do MSI (DEB-12)

A preencher: opções, custo, processo, segredo no CI, efeito no SmartScreen, fontes consultadas e data.

### T5: índice de referências (REF-10)

A preencher.

### T6: `git` em repositório grande (HIS-08)

A preencher.

### T7: export, PDF e Mermaid em SVG (EXP-05, EXP-06, EXP-07)

A preencher, por plataforma.

### T8: lint incremental (LNT-08)

A preencher.

## Journal

- 2026-11-16 · planejamento · branch `docs/v0-3-spec` criada de `main` (`13523e6`, depois do merge da v0.2 em `5efaa19` e do registro da release de teste `v0.2.0-rc.1`). Fontes lidas: `AGENTS.md`, `docs/ideation/moscow.md` (fatiamento v0.3+ e critério de pronto), `docs/spec-format.md`, spec, design, tasks e summary da v0.1 e da v0.2 (Relatório da manhã, pendências, "Onde parou", medições T6, T9, T10 e T11), e o código atual (`parse/ids.rs`, `parse/tasks.rs`, `parse/summary.rs`, `preset.rs`, `config.rs`, `git.rs`, `index.rs`, `write.rs`, `lib/markdown/`, `features/conflict/diff.ts`, `src-tauri/Cargo.toml`, `quire-cli`)
- 2026-11-16 · planejamento · **a spec nasce sem feedback de uso da v0.2.** O campo de feedback do pedido veio vazio, e a v0.2 virou release de teste no mesmo dia. Registrado como risco no design e como Q10
- 2026-11-16 · planejamento · achado que muda o desenho dos backlinks: **IDs de requisito são únicos só dentro da spec**: o `spec-format.md` define o padrão do ID, mas nada impede duas specs do mesmo projeto de usarem o mesmo (A1). Hoje não há colisão nas specs do Quire (a v0.2 continuou a numeração da v0.1) nem nas fixtures (`TOK`, `RTL`, `INV`), conferido por comparação dos IDs; a T22 planta uma colisão para testar. Por isso um ID nu resolve só na própria spec (TD-26), e a ligação entre specs é pela chave de issue. A forma qualificada (`ACME-142/CCR-03`) muda o formato e ficou como Q1. Os IDs novos desta spec evitam colisão com os da v0.2: DEB e OSS continuam a numeração (DEB-11 a DEB-14, OSS-07, OSS-08), e o alerta `tasks.md` × `summary.md` usa o PRG-02 reservado na v0.2
- 2026-11-16 · planejamento · decisões de desenho registradas como TD novos sem reescrever os antigos: TD-25 a TD-36, mais o TD-37 reservado para a assinatura do MSI. O TD-30 complementa o TD-20: o `quire-cli` deixa de ser só um repassador do Windows e passa a depender do `quire-core` para executar o `quire lint` no console, porque o binário GUI do Linux depende do WebKitGTK e não serve a um CI. Publicar o `quire-cli` para Linux ficou como Q3
- 2026-11-16 · planejamento · confirmado no código: o modo Ler não renderiza HTML cru (sem `rehype-raw`), o Mermaid roda com `securityLevel: "strict"`, links não `http(s)` não navegam, não há `tauri-plugin-dialog`, e o `git.rs` já tem tempo limite (5 s) e trata o `git` ausente. Com isso, o export reaproveita a árvore do modo Ler sem precisar sanitizar HTML do usuário, e o histórico parte do mesmo mecanismo de processo do `git` da v0.1
- 2026-11-16 · planejamento · riscos tratados de frente no design: IDs ambíguos (TD-26), falsos positivos do lint (pré-condições por regra, fixtures limpas com zero problemas, severidade conservadora), `git` lento ou ausente (tempo limite, cancelamento, paginação), `git` executando código do repositório por `core.fsmonitor`/`diff.external`/`textconv` (TD-31, com teste de arquivo-sentinela), export vazando caminho (teste de vazamento, verificação antes de gravar, JSONL fora), PDF diferente entre WebKitGTK e WebView2 (fontes embutidas, spike T7) e desempenho do índice de referências (spike T5)
- 2026-11-16 · planejamento · Fase 0 decidida: T1 (WSL, do autor, não bloqueia pelo padrão da Q7), T2 (assinatura do MSI), T3 (fontes, agora exigidas pelo PDF) e T4 (outline, exigido pelo voltar/avançar). Detecção de unidade mapeada no Windows descartada da v0.3, com justificativa na tabela de pendências
