# Roteiros E2E

A v0.1 (leitura) vem primeiro; a v0.2 (escrita, conflito, busca e palette) está em "v0.2", no fim.

## v0.1

Roteiro manual sobre o workspace sintético (`fixtures/`), cobrindo os critérios P1 da spec (`.specs/features/quire-v0-1/spec.md`). A coluna **Auto** indica o que também é coberto por testes automáticos (`cargo test --workspace`, `pnpm -r test`); o resto precisa de olho humano na janela real.

### Preparação

```bash
pnpm install
scripts/make-fixtures.sh                     # recria fixtures/workspace do zero (idempotente)
rm -f /tmp/quire-e2e.sqlite
QUIRE_CONFIG="$PWD/fixtures/quire.toml" QUIRE_INDEX=/tmp/quire-e2e.sqlite pnpm --filter desktop tauri dev
```

O workspace tem dois projetos fictícios: `acme_gateway` (com origin, uma worktree idêntica dentro do scan path e uma divergente em `fixtures/workspace/external/`, fora dele) e `acme_billing` (sem origin), além de pastas `.specs` de isca dentro de `node_modules`, `target` e `dist`.

Para o CLI, use o binário gerado pelo `tauri dev` (`target/debug/quire`) com as mesmas variáveis `QUIRE_CONFIG` e `QUIRE_INDEX`.

### Descoberta, projetos e índice

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 1 | DSC-01, DSC-02 | Abrir o app com o índice vazio | Sidebar com `acme_billing` (3 specs) e `acme_gateway` (3 specs); nenhuma spec `decoy-*` | sim |
| 2 | DSC-03 | Ver as instâncias de `acme-170-rate-limit` | 3 cópias, inclusive a da worktree em `external/`, fora do scan path | sim |
| 3 | DSC-04 | Observar o rodapé da sidebar durante a primeira varredura | "Varrendo…" / "Indexando x de y" e depois "Varredura concluída"; a lista aparece sem travar a janela | sim |
| 4 | DSC-05 | Em Preferências, adicionar `/caminho/inexistente` e salvar | A pasta aparece com o motivo; o rodapé também mostra o erro; as specs das outras pastas continuam lá | sim |
| 5 | DUP-01 | Ver os nomes dos projetos | `acme_gateway` (do origin) e `acme_billing` (da pasta); as três cópias do gateway num só projeto | sim |
| 6 | DUP-02, DUP-03 | Ver `ACME-142` e `ACME-170` na sidebar | ACME-142 sem badge (cópias idênticas); ACME-170 com o badge de 3 versões | sim |
| 7 | DUP-04 | Abrir `ACME-170` | Abre a cópia mais recente (worktree externa, branch `feat/acme-170-rate-limit`) | sim |
| 8 | DUP-05 | Clicar no selo de instância no topo | Popover com branch, tipo, caminho, "modificada há…" e estado (aberta, diverge, idêntica) | sim |
| 9 | DUP-04 | Escolher outra cópia e "Abrir esta"; trocar de spec e voltar | A aba atual se mantém; ao voltar, a cópia escolhida continua | sim |
| 10 | DUP-06 | Ver o topo com uma spec aberta | Branch e worktree da cópia exibida | sim |
| 11 | IDX-01 | Fechar o app, apagar `/tmp/quire-e2e.sqlite*`, abrir de novo | Mesma lista depois da varredura | sim |
| 12 | IDX-02, IDX-03 | Abrir o app de novo sem mudar nada | A lista aparece imediatamente, a partir do cache (medido: 300 specs em ~2 ms) | sim |
| 13 | PST-01 | Abrir Preferências com o `config.toml` sem preset para um projeto | O projeto usa `generic`; o índice registra a sugestão do `exban` | sim |
| 14 | PST-02 | Trocar o preset de `acme_billing` para `generic` e salvar | Sem reiniciar, as abas e o progresso passam a seguir o `generic` | sim |
| 15 | PST-03 | Abrir `design.md` (não é tasks) e `payload.json` | Nenhum erro; markdown e JSON exibidos normalmente | sim |

### Leitura

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 16 | VIW-01 | Abrir `ACME-142` e depois `ACME-170` | ACME-142: 7 abas e "outros 2"; ACME-170 (parcial): só spec, design e tasks | sim |
| 17 | VIW-02, VIW-04 | Ler `spec.md` e `design.md` | Tabelas, listas de tarefas, links, código realçado e o diagrama ASCII alinhado | parcial |
| 18 | VIW-03 | Abrir `design.md` | Diagrama Mermaid desenhado; um bloco inválido (página de medição do spike) mostra o código e o erro sem quebrar a página | parcial |
| 19 | VIW-05 | Rolar o `spec.md`; clicar em "Casos de borda" no outline | A seção visível fica destacada; o clique rola até ela | parcial |
| 20 | VIW-06 | Recolher "Histórias de usuário" | O conteúdo some até expandir de novo | sim |
| 21 | VIW-07 | Ver a faixa de metadados; clicar em `ACME-142 ↗` | Chips do Status e Idioma; o link abre no navegador do sistema | parcial |
| 22 | VIW-08 | Aba "outros": `notes.md` e `payload.json` | Markdown e JSON realçado; arquivo binário mostraria aviso | sim |

### Tasks, gates e lessons

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 23 | TSK-01 | Aba tasks de `ACME-142` | 3 fases com x/y, barra e estado; a Fase 2 aberta | sim |
| 24 | TSK-02 | Ver a Fase 2 | Cadeias `T4 › T5` e `T4 › T6` em chips coloridos | sim |
| 25 | TSK-03 | Ver as linhas | Estado, id, título, IDs de requisito e hash curto | sim |
| 26 | TSK-04 | Clicar em T6 | Painel com onde, depende de, requirement, pronto quando, testes e gate | sim |
| 27 | TSK-05 | Tentar marcar um checkbox | Não marca; aviso "Quem marca é o agente" | sim |
| 28 | LOG-01 | Aba gates de `BILL-31` | Tabela da mais recente para a mais antiga; filtros de nível e status; "3 execuções · 2 pass · 1 fail" | sim |
| 29 | LOG-02 | Ver a execução com falha | `tail`, exit 101 e cwd | sim |
| 30 | LOG-03 | Aba lessons de `ACME-142` | Sinal, escopo, texto e fonte | sim |
| 31 | LOG-04 | Aba gates de `BILL-31` | Aviso de 2 linhas ignoradas (linhas 2 e 5) | sim |

### Ao vivo

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 32 | WAT-01 | Com a aba tasks de `ACME-142` aberta, marcar `T6` no `tasks.md` da cópia exibida num editor | Em menos de 2 s: 8/8, Fase 2 completa (medido ~0,2 s) | sim |
| 33 | WAT-01 | Criar `.specs/features/acme-300-nova/spec.md` (pasta e arquivo de uma vez) e depois apagar a pasta | A spec aparece e some da lista | sim |
| 34 | WAT-02 | Rolar até o meio do `spec.md`, recolher uma seção e editar o arquivo por fora | Aba, seção recolhida e posição de rolagem mantidas | parcial |
| 35 | WAT-03 | Em `[watch.paths]`, forçar `poll` para a pasta das fixtures, salvar uma edição | Rodapé com "polling a cada 3 s"; a mudança aparece dentro do intervalo | parcial |
| 36 | WAT-04 | Com `ACME-170` aberta na worktree externa, rodar `git -C fixtures/workspace/projects/acme_gateway worktree remove --force ../../external/acme-170/acme_gateway` | A visão passa para outra cópia com aviso | parcial |
| 37 | WAT-01 | Salvar um arquivo com rename atômico (vim, VS Code) | Tratado como mudança, não como remoção | sim |

### CLI

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 38 | CLI-05 | `quire --help` e `quire --version` (app aberto e fechado) | Texto no terminal, código 0, nenhuma janela | parcial |
| 39 | CLI-01, CLI-04 | `cd fixtures/workspace/external/acme-170/acme_gateway && quire .` | A janela existente vem para frente; 3 specs destacadas na sidebar; continua 1 janela | sim |
| 40 | CLI-02 | `quire open acme-200` | Seletor com as duas specs (gateway e billing) | sim |
| 41 | CLI-02 | `quire open ACME-142` | Abre a spec direto | sim |
| 42 | CLI-03 | `quire open fixtures/workspace/projects/acme_gateway/.specs/features/acme-142-service-tokens/design.md` | Abre a spec na aba design | sim |

### Plataformas ainda não cobertas (v0.1)

- **WSL:** WSLg (render) e `\\wsl$`/DrvFs (watcher): ver as pendências dos spikes T2 e T3 no `summary.md`.
- **Windows nativo:** saída do CLI no console (P2 da spec) e o instalador `.msi`.

## v0.2

Roteiro manual da v0.2 (`.specs/features/quire-v0-2/spec.md`), com os cenários de perda de dados. **Sempre sobre um workspace gerado** (`scripts/make-fixtures.sh` recria `fixtures/workspace`, que é ignorado pelo git), nunca sobre specs reais. Use o binário de produção (`pnpm --filter desktop tauri build --bundles deb`, depois `target/release/quire`), com `QUIRE_CONFIG="$PWD/fixtures/quire.toml"` e um `QUIRE_INDEX` absoluto descartável. Para os passos do checkbox, ligue "Marcar tarefas" de `acme_gateway` nas Preferências.

Os arquivos de formato ficam em `acme_billing/.specs/features/bill-40-refunds/` (aba "outros"): `formato-crlf-bom.md`, `formato-sem-newline.md`, `formato-misto.md` e, fora do Windows, `formato-link.md` (link para `acme_billing/docs/link-alvo.md`) e `formato-somente-leitura.md`.

**Auto:** "sim" quando um teste automático cobre o passo inteiro (o nome do teste vem entre parênteses), "parcial" quando cobre a lógica mas não a janela.

### Editor e escrita

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 43 | EDT-01, EDT-02 | Abrir `ACME-142`, aba spec, e alternar Ler / Editar / Dividir | Ler por padrão; o editor aparece na primeira vez em até ~0,5 s; Dividir mostra editor e preview lado a lado | parcial (`EditBar.test`) |
| 44 | EDT-07 | Em Editar, digitar `á é ã ç ü` com teclas mortas (ABNT2 e US-intl), colar um parágrafo, Ctrl+Z / Ctrl+Shift+Z, Ctrl+F e substituir | Nada perdido ou duplicado; desfazer volta através de um save | parcial (`codemirror.test`) |
| 45 | EDT-03, EDT-04 | Em Dividir, digitar e parar; rolar o editor até a tabela e o preview até um título | O preview acompanha em < 0,5 s depois da pausa; os dois lados se alinham nos dois sentidos, sem tremer | parcial (`syncScroll.test`) |
| 46 | EDT-05, EDT-06 | Ctrl+S sem alteração; depois alterar e salvar | Sem alteração nada é gravado (o `mtime` não muda); com alteração, o ponto na aba e o "•" no título somem depois de salvar | sim (`store.test`, `write_commands_over_ipc`) |
| 47 | SAV-03 | Em `formato-crlf-bom.md` e `formato-sem-newline.md`, acrescentar uma linha e salvar; rodar `git -C fixtures/workspace/projects/acme_billing diff` | O diff mostra só a linha nova; nada de `^M`, BOM ou "No newline" mudando | sim (`text_formats_survive_a_save_byte_for_byte`) |
| 48 | SAV-03 | Abrir `formato-misto.md` em Editar, alterar e salvar | Aviso "Fins de linha mistos"; o save pede "Normalizar e salvar"; depois, todas as linhas em CRLF | sim (idem) |
| 49 | SAV-05 | Editar `formato-link.md` e salvar (fora do Windows) | `ls -l` mostra que continua link; `docs/link-alvo.md` recebeu o texto | sim (idem) |
| 50 | SAV-04 | Editar `formato-somente-leitura.md` e salvar | Erro "o arquivo é somente leitura" com "Salvar como cópia"; o original fica igual | sim (idem) |
| 51 | EDT-08 | Abrir `gates.jsonl`, `payload.json` e um arquivo > 2 MB | Editar e Dividir desabilitados, com cadeado e o motivo | sim (`EditBar.test`, `write_commands_over_ipc`) |
| 52 | SAV-06 | Com alteração não salva: trocar de aba, de spec, de cópia, abrir a busca, as Preferências, a palette numa spec, `quire open ACME-170` e fechar a janela | Sempre Salvar / Descartar / Cancelar; Cancelar mantém tudo; nada é gravado sem a escolha | parcial (`unsaved.test`) |

### Conflito com o agente (perda de dados)

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 53 | CNF-04 | Sem alteração no editor, `printf '\nDo agente.\n' >> .../acme-142-service-tokens/spec.md` | O editor recarrega sozinho, mantendo o modo; Ctrl+Z desfaz a recarga | parcial (`conflict.test`) |
| 54 | CNF-05 | Com alteração não salva, repetir o `printf` | Aviso fixo "O arquivo mudou no disco"; nada é recarregado nem gravado; a edição continua | parcial (`conflict.test`) |
| 55 | CNF-02, CNF-09 | Salvar nesse estado; "Ver diferenças" | O diálogo de conflito abre; o diff mostra a linha do agente "só no disco" e a sua "só no editor" | parcial (`conflict.test`) |
| 56 | CNF-02 | "Recarregar do disco" / "Sobrescrever" / "Salvar como cópia", um em cada rodada | Recarregar descarta as suas; Sobrescrever grava as suas por cima (só por escolha); a cópia `spec.conflito-AAAAMMDD-HHMMSS.md` aparece em "outros" e o original fica com o texto do agente | parcial (`conflict.test`, `write_commands_over_ipc`) |
| 57 | CNF-03 | Salvar enquanto um laço escreve no arquivo (`while :; do printf x >> spec.md; sleep 0.01; done`) | O save vira conflito; nenhuma escrita do laço some sem "Sobrescrever" | sim (`an_outside_write_between_the_temp_and_the_rename_is_caught`) |
| 58 | CNF-07 | Com alteração não salva, `rm` do arquivo | Aviso "foi apagado no disco"; o buffer continua; "Recriar o arquivo" só por escolha | parcial (`conflict.test`) |
| 59 | CNF-06 | Salvar várias vezes seguidas | Nenhum aviso de conflito, nenhuma recarga, nenhuma reconciliação em laço | sim (`a_save_settles_without_a_reload_cycle`) |
| 60 | SAV-08 | Windows: abrir o arquivo em outro programa que trava a exclusão e salvar | Tenta por ~1 s e mostra "o arquivo está em uso"; o original fica | sim, só no CI do Windows |
| 61 | SAV-01, SAV-10 | Crash: matar o app com `kill -9` durante um save grande (ou rodar `cargo test -p quire-core --test crash`) | O original fica inteiro; se sobrar `.<nome>.quire-…tmp`, a spec mostra o aviso com Remover e o `git status` o mostra como não rastreado | sim (`a_crash_before_the_rename_keeps_the_original`) |
| 62 | SAV-11 | Abrir uma spec em `/mnt/c` (WSL) ou `\\wsl$` (Windows) | Editar e Dividir desabilitados com "escrita desativada neste sistema de arquivos"; checkbox desligado | parcial (`fs_kind`), sem máquina aqui |

### Várias cópias

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 63 | INS-01, INS-02 | Editar `ACME-170` | A barra diz "Edições vão para feat/acme-170-rate-limit · acme-170/acme_gateway" e avisa que as cópias não são todas iguais | sim (`instances.test`) |
| 64 | INS-03 | Salvar uma mudança em `ACME-142` | Em até 2 s a sidebar e o selo passam a mostrar o badge de versões | sim (`saving_one_identical_copy_makes_the_versions_badge_appear`) |
| 65 | INS-04 | Com alteração não salva em `ACME-170`, `git worktree remove --force` da worktree externa | Nada troca sozinho; aviso com "Salvar como cópia em outra instância"; ao usar, o texto chega à outra cópia e abre o diálogo de conflito | parcial (`instances.test`) |

### Busca, palette, checkbox e idioma

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 66 | SRC-03, SRC-04 | Ctrl+Shift+F, buscar `integracoes token` | Trechos agrupados por spec, termos marcados; ACME-142 aparece uma vez apesar das 3 cópias | sim (`search_groups_by_spec_and_shows_identical_copies_once`) |
| 67 | SRC-05, SRC-07 | Filtrar por projeto, por `tasks.md`, por "só na spec aberta"; buscar `"`, `NEAR(`, `a:b` | Filtros aplicados; nenhuma busca dá erro | sim (`tests/search.rs`) |
| 68 | SRC-06 | Abrir um resultado | A spec abre na cópia do resultado, na aba e na seção, com os termos destacados no texto | parcial (`search.test`) |
| 69 | SRC-02 | Editar um arquivo por fora e buscar a palavra nova | Aparece em ~1 s (reindexa só aquele arquivo) | sim (`changing_one_file_reindexes_only_that_file`) |
| 70 | CMD-01..04 | Ctrl+K dentro do editor; `acm170` + Enter; Esc; `/retry` | Abre com foco no campo; casamento aproximado; setas, Enter e Esc; a busca aparece no mesmo painel, com a branch quando há cópias divergentes | sim (`Palette.test`) |
| 71 | CHK-01..05 | Com "Marcar tarefas" ligado, marcar a T3 de `ACME-170`; depois `git diff` | Só o caractere `[ ]` → `[x]` muda; o progresso da aba, da sidebar e do cabeçalho muda em < 1 s | sim (`checkbox_and_progress_without_the_watcher`, `checkbox.test`) |
| 72 | CHK-03 | Alterar o `tasks.md` por fora e marcar sem recarregar | Aviso "o tasks.md mudou no disco"; nada gravado; aba recarregada | sim (`checkbox.test`) |
| 73 | DEB-10 | `LANG=en_US.UTF-8 target/release/quire`; depois Preferências → Idioma → Português (Brasil) → Salvar | Abre em inglês; troca sem reiniciar; `language = "pt-BR"` no `config.toml` com os comentários | sim (`Preferences.test`, `config`) |
| 74 | DEB-06 | Windows: `quire --help`, `quire --version`, `quire --bogus`, `quire .` num `cmd` e num PowerShell novos | Texto no terminal, códigos 0, 0 e 2; `quire .` abre o app e devolve o terminal | sim, só no CI do Windows |
