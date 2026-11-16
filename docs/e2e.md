# Roteiro E2E da v0.1

Roteiro manual sobre o workspace sintético (`fixtures/`), cobrindo os critérios P1 da spec (`.specs/features/quire-v0-1/spec.md`). A coluna **Auto** indica o que também é coberto por testes automáticos (`cargo test --workspace`, `pnpm -r test`); o resto precisa de olho humano na janela real.

## Preparação

```bash
pnpm install
scripts/make-fixtures.sh                     # recria fixtures/workspace do zero (idempotente)
rm -f /tmp/quire-e2e.sqlite
QUIRE_CONFIG=fixtures/quire.toml QUIRE_INDEX=/tmp/quire-e2e.sqlite pnpm --filter desktop tauri dev
```

O workspace tem dois projetos fictícios: `acme_gateway` (com origin, uma worktree idêntica dentro do scan path e uma divergente em `fixtures/workspace/external/`, fora dele) e `acme_billing` (sem origin), além de pastas `.specs` de isca dentro de `node_modules`, `target` e `dist`.

Para o CLI, use o binário gerado pelo `tauri dev` (`target/debug/quire`) com as mesmas variáveis `QUIRE_CONFIG` e `QUIRE_INDEX`.

## Descoberta, projetos e índice

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

## Leitura

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 16 | VIW-01 | Abrir `ACME-142` e depois `ACME-170` | ACME-142: 7 abas e "outros 2"; ACME-170 (parcial): só spec, design e tasks | sim |
| 17 | VIW-02, VIW-04 | Ler `spec.md` e `design.md` | Tabelas, listas de tarefas, links, código realçado e o diagrama ASCII alinhado | parcial |
| 18 | VIW-03 | Abrir `design.md` | Diagrama Mermaid desenhado; um bloco inválido (página de medição do spike) mostra o código e o erro sem quebrar a página | parcial |
| 19 | VIW-05 | Rolar o `spec.md`; clicar em "Casos de borda" no outline | A seção visível fica destacada; o clique rola até ela | parcial |
| 20 | VIW-06 | Recolher "Histórias de usuário" | O conteúdo some até expandir de novo | sim |
| 21 | VIW-07 | Ver a faixa de metadados; clicar em `ACME-142 ↗` | Chips do Status e Idioma; o link abre no navegador do sistema | parcial |
| 22 | VIW-08 | Aba "outros": `notes.md` e `payload.json` | Markdown e JSON realçado; arquivo binário mostraria aviso | sim |

## Tasks, gates e lessons

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

## Ao vivo

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 32 | WAT-01 | Com a aba tasks de `ACME-142` aberta, marcar `T6` no `tasks.md` da cópia exibida num editor | Em menos de 2 s: 8/8, Fase 2 completa (medido ~0,2 s) | sim |
| 33 | WAT-01 | Criar `.specs/features/acme-300-nova/spec.md` (pasta e arquivo de uma vez) e depois apagar a pasta | A spec aparece e some da lista | sim |
| 34 | WAT-02 | Rolar até o meio do `spec.md`, recolher uma seção e editar o arquivo por fora | Aba, seção recolhida e posição de rolagem mantidas | parcial |
| 35 | WAT-03 | Em `[watch.paths]`, forçar `poll` para a pasta das fixtures, salvar uma edição | Rodapé com "polling a cada 3 s"; a mudança aparece dentro do intervalo | parcial |
| 36 | WAT-04 | Com `ACME-170` aberta na worktree externa, rodar `git -C fixtures/workspace/projects/acme_gateway worktree remove --force ../../external/acme-170/acme_gateway` | A visão passa para outra cópia com aviso | parcial |
| 37 | WAT-01 | Salvar um arquivo com rename atômico (vim, VS Code) | Tratado como mudança, não como remoção | sim |

## CLI

| # | Critério | Ação | Esperado | Auto |
| --- | --- | --- | --- | --- |
| 38 | CLI-05 | `quire --help` e `quire --version` (app aberto e fechado) | Texto no terminal, código 0, nenhuma janela | parcial |
| 39 | CLI-01, CLI-04 | `cd fixtures/workspace/external/acme-170/acme_gateway && quire .` | A janela existente vem para frente; 3 specs destacadas na sidebar; continua 1 janela | sim |
| 40 | CLI-02 | `quire open acme-200` | Seletor com as duas specs (gateway e billing) | sim |
| 41 | CLI-02 | `quire open ACME-142` | Abre a spec direto | sim |
| 42 | CLI-03 | `quire open fixtures/workspace/projects/acme_gateway/.specs/features/acme-142-service-tokens/design.md` | Abre a spec na aba design | sim |

## Plataformas ainda não cobertas

- **WSL:** WSLg (render) e `\\wsl$`/DrvFs (watcher): ver as pendências dos spikes T2 e T3 no `summary.md`.
- **Windows nativo:** saída do CLI no console (P2 da spec) e o instalador `.msi`.
