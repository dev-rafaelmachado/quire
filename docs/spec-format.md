# Formato da pasta de spec (preset `exban`)

Notas para quem implementa os parsers. Todos os exemplos são sintéticos. O núcleo do Quire não depende deste formato: ele só descreve o que o preset `exban` reconhece.

## Onde ficam

```
<repo>/.specs/features/<issue-key-em-minusculas>-<slug>/
  spec.md  design.md  tasks.md  summary.md  validation.md  gates.jsonl  lessons.jsonl  [outros .md]
<repo>/.specs/STATE.md          # opcional: decisões do projeto (AD-xxx)
```

- O nome da pasta é `<chave>-<slug>` em minúsculas (ex.: `acme-142-service-tokens`). A chave da issue (`ACME-142`) se extrai do prefixo por regex `^([a-z]+-\d+)-`.
- A mesma pasta pode existir em várias worktrees do mesmo repo (`<raiz>/.worktrees/<branch>/<repo>/.specs/...`), com conteúdo igual ou divergente.
- Nem toda spec tem todos os arquivos: uma spec em andamento pode não ter `validation.md` nem `lessons.jsonl`.

## Metadados: cabeçalho em negrito, sem frontmatter

As primeiras linhas de `spec.md` e `summary.md` carregam metadados como `**Chave:** valor`:

```markdown
# ACME-142 — Título da spec

**Status:** Concluído, aguardando revisão · 7 de 7 fases · 28/28 tarefas · verificação full PASS
**Issue:** [ACME-142](https://tracker.example/browse/ACME-142)
**Idioma:** pt-BR
```

O cabeçalho é o bloco de campos `**Chave:** valor` (também aceito `**Chave**: valor`) logo depois do título `# …`, com linhas em branco permitidas entre o título e o bloco. O bloco termina na primeira linha em branco depois dos campos, no próximo título (`#`, `##`, …) ou na primeira linha que não seja um campo; linhas `**Chave:**` mais abaixo no arquivo não fazem parte do cabeçalho. A URL base do tracker vem do link da issue.

## `tasks.md`

````markdown
### Fase 1: Nome da fase — repo

```
T1 → T2 → T3
```

- [x] T1: Título da tarefa (a1b2c3d)
- [x] T2: Outra tarefa (e4f5a6b)
- [ ] T3: Tarefa pendente
````

- Linha de tarefa: `- [ ]` ou `- [x]`, id `T<n>`, `:`, título, e opcionalmente o hash curto do commit entre parênteses no fim.
- Depois do título vêm, opcionalmente, um ou mais hashes curtos entre parênteses (`(a1b2c3d)` ou `(a1b2c3d, e4f5a6b)`) e, depois deles, uma nota entre parênteses, como `(gate passou, aguardando revisão visual)`. O parser separa os hashes e a nota do título. Um parêntese que não vem depois de um hash continua no título (`CI (Linux e Windows)`). Uma anotação entre o id e os dois-pontos (`T4 (aguardando CI):`) também é lida como nota.
- Fases: cabeçalhos `### Fase <n>: <título>` (com sufixo opcional ` — <repo>`). O bloco de código logo abaixo é o diagrama de dependência em ASCII (`T1 → T2`; fases paralelas usam várias linhas).
- Detalhamento: seções `### T<n>: <título>` com campos em negrito: `**What**`, `**Onde**`, `**Depende de**`, `**Reuses**`, `**Requirement**` (IDs separados por vírgula), `**Tools**`, `**Pronto quando**` (lista), `**Testes**`, `**Gate**` (`fase`, `final`, `e2e`).
- O progresso é contado nas linhas de checkbox. Tarefas de fases de correção podem ter ids fora de ordem (ex.: `T26` entre `T20` e `T21`).

## `summary.md`

- Cabeçalho de metadados (acima), seção `## Em uma frase`.
- Tabela de progresso gerada por script entre marcadores:

```markdown
<!-- progress:start -->
| Fase | Repositório | Tarefas |
|---|---|---|
| 1: Fundações | repo | 5/5 ✓ |
<!-- progress:end -->
```

- `## Onde parou`: lista de campos (`Fase / tarefa`, `Concluídas`, `Em andamento`, `Próximo passo`, `Bloqueios`, `Arquivos sem commit`, `Branch / worktree`).
- `## Journal`: uma entrada por linha, `- AAAA-MM-DD [HH:MM] · <ator> · <texto>`, em ordem cronológica. Atores vistos: `planejamento`, `dev`.

## `spec.md`: IDs de requisito

- Histórias `### P1: título ⭐ MVP`, com critérios numerados em estilo `QUANDO … ENTÃO o sistema DEVE …`.
- Cada critério termina com um comentário HTML com o id: `<!-- CCR-02 -->`. Padrão de id: `[A-Z]{2,5}-\d{2}`.
- Seções fixas usadas pelo outline: Fontes, Problema, Objetivos, Fora do escopo, Premissas e perguntas em aberto, Histórias de usuário, Casos de borda, Rastreabilidade de requisitos, Critérios de sucesso.

## `validation.md`

- Cabeçalho `## Validação: <nome> - PASS ✅` (ou FAIL).
- Tabela de critérios: `| ID | Resultado da spec | Evidência | Status |`, onde a evidência cita `arquivo:linha`.
- Seções `## Sensor de discriminação` (tabela de mutações: `arquivo:linha`, mutação, critério, resultado), `## Casos de borda`, `## Gates`, `## Não verificado`.

## `gates.jsonl` (somente leitura)

Um objeto JSON por linha, append-only:

```json
{"ts":"2026-11-04T13:50:51Z","repo":"gateway","level":"fase","cmd":"yarn typecheck && yarn jest","cwd":"/home/dev/projects/.worktrees/ACME-142/gateway","head":"4ebd9b01f38d","exit":0,"status":"pass","duration_s":96.6,"tail":[]}
```

- `level`: `fase`, `final` ou `e2e`. `status`: `pass` ou `fail`. `head`: sha do commit testado (liga o gate às tarefas pelo hash curto). `tail`: últimas linhas de saída, normalmente só preenchidas em falhas.

## `lessons.jsonl` (somente leitura)

```json
{"ts":"2026-11-04T16:25:37Z","feature":"acme-142-service-tokens","signal":"ac_gap","source":"spec.md:233","text":"…","scope":"integration/concurrency"}
```

- `signal`: `ac_gap`, `gate_fail`, `spec_precision_gap`, entre outros. `source` pode ser `arquivo:linha`, caminho de código ou um id de requisito (`KEY-08`).

## Referências cruzadas que o app deve resolver

- `arquivo:linha` (em validation, lessons, "Fontes"): abre no editor na linha, relativo à raiz do repo da instância.
- Ids de requisito (`CCR-02`): spec ↔ `Requirement` das tasks ↔ evidência no validation.
- Hash curto nas tasks e `head` nos gates: commit.
- `AD-xxx`: decisões em `.specs/STATE.md`.
- Chaves de issue de outras specs (`ACME-170`): links entre specs, possivelmente de outro projeto.
