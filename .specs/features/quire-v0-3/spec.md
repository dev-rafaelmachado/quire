# QUIRE v0.3 — Referências, lint, histórico e export

**Idioma:** pt-BR

**Repositório:** `quire`: `crates/quire-core` (referências, lint, git somente leitura, modelos), `crates/quire-cli` (`quire lint`), `apps/desktop` (painéis, navegação, export, shell Tauri), `fixtures/`
**Data:** 2026-11-16
**Status:** rascunho, aguardando as decisões do autor sobre Q1–Q10
**Complexidade:** Large. Verificação: `full` (novo índice no cache, processo `git` sobre repositórios do usuário, escrita de arquivo exportado e criação de arquivo de spec).

## Fontes

- **Ideação e MoSCoW:** `docs/ideation/moscow.md`. Fatiamento: "v0.3+: visões ricas e Could", com o critério de pronto "seguir um critério da spec até a evidência sem sair do app". Os Could escolhidos são lint de estrutura, backlinks, histórico/diff via git e export PDF/HTML.
- **v0.1:** `.specs/features/quire-v0-1/`: spec, design e o Journal do `summary.md`, com as pendências de outline, `arquivo:linha` e seletor de instâncias.
- **v0.2:** `.specs/features/quire-v0-2/`: spec, design (TD-11 a TD-24), tasks e `summary.md`, com o Relatório da manhã, as pendências, o backlog decidido para a v0.3 (P1 e P5) e as medições T6, T9, T10 e T11.
- **Formato das pastas de spec:** `docs/spec-format.md`, seção "Referências cruzadas que o app deve resolver".
- **Design visual:** `docs/design/README.md`, `docs/design/tokens.css`, `docs/design/wireframes/*.html`. O wireframe 4 prevê "Comparar" no seletor de instâncias. Painéis de referências, problemas, histórico e export ainda não têm wireframe.
- **Convenções do projeto:** `AGENTS.md`.
- **Feedback de uso da v0.2:** **nenhum.** A v0.2 virou release de teste (`v0.2.0-rc.1`, rascunho) no mesmo dia em que esta spec foi escrita, e o autor ainda não usou o app no dia a dia. A spec nasce sem feedback de uso; ver o risco "Spec sem feedback de uso" e Q10.

## Problema

A v0.2 permite ler e editar specs, mas a pergunta mais comum de quem revisa uma spec continua sem resposta dentro do app: "este critério foi implementado e provado?". Para responder, o usuário procura o ID do critério no `tasks.md` (campo `**Requirement**`), depois no `validation.md` (coluna Evidência, que cita `arquivo:linha`), depois o hash da tarefa no `gates.jsonl`, e às vezes uma decisão `AD-xxx` no `STATE.md` do repositório. Hoje isso é busca textual manual, aba por aba. Nada avisa quando a estrutura está quebrada: um critério sem tarefa, uma tarefa que cita um ID que não existe, a tabela de progresso do `summary.md` desatualizada. Também não há como ver como a spec mudou (o git já sabe) nem como entregar a spec a alguém que não tem o Quire.

## Objetivos

- Seguir um critério da spec até a tarefa, a evidência e o gate por cliques, e voltar, sem sair do app.
- Mostrar onde cada ID, chave de issue e decisão é referenciado, com a gramática vinda do preset.
- Apontar problemas de estrutura sem ruído, no app e no CI do projeto (`quire lint`, sem rede).
- Ver o histórico da spec pelo git, abrir um arquivo num commit e comparar commits e instâncias, sem nenhuma escrita no repositório.
- Exportar uma spec para HTML estático de arquivo único e para PDF, sem vazar caminho nem dado interno.
- Criar um arquivo primário ausente a partir do modelo do preset (a P1 da v0.2, no escopo mínimo).
- Fechar antes as dívidas que são decisão de segurança ou de distribuição: WSL e escrita em `\\wsl$`/DrvFs/9p, assinatura do MSI e fontes embutidas.

## Fora de escopo da v0.3

- **Criar spec nova** (pasta inteira, com chave, slug, repositório e worktree). Só depende de decisão do autor (Q4). Padrão desta spec: fica para o backlog da v0.4; a v0.3 cria apenas arquivos primários ausentes de uma spec que já existe (NEW).
- **Verificar links `http(s)`:** sem rede (AGENTS.md). O lint só verifica links relativos e referências locais.
- **Escrita no repositório git** de qualquer forma: commit, checkout, stash, fetch, `git add`, atualização do índice do git, notas. O histórico é somente leitura (HIS-05).
- **Correção automática pelo lint** ("quick fix"). O lint só aponta; corrigir é editar no editor da v0.2.
- **Export de `gates.jsonl` e `lessons.jsonl`.** Eles carregam `cwd` e caminhos absolutos dos agentes (Q5); o export cobre só markdown.
- **Abrir `arquivo:linha` de código no `$EDITOR` na linha certa.** A v0.3 abre o arquivo no aplicativo padrão do sistema e copia `caminho:linha` (REF-09). Abrir no `$EDITOR` com linha continua Should de outra release.
- **"N commits à frente/atrás" no seletor de instâncias** (wireframe 4): vai para o backlog. A v0.3 traz o "Comparar" (HIS-07).
- **Detecção de unidade de rede mapeada no Windows:** descartada da v0.3. Exige FFI ou dependência que encapsule `unsafe` (o mesmo motivo que descartou a opção (b) do TD-20). O override `[watch.paths]` da v0.1 cobre o caso. Reavaliar só com feedback de uso de alguém no Windows.
- **Itens de outras releases:** kanban, Jira por API, MCP, sync, colaboração, WYSIWYG (Won't); journal com quick-add, copiar contexto para um agente, indicadores de git por projeto, abrir no `$EDITOR`/terminal (Should fora deste fatiamento).
- **Revisões pendentes da v0.2:** capturas, roteiro E2E, roteiro de teclas mortas e IME (T9), `quire .` no Windows (T5) e orçamento de memória (T6). Continuam na v0.2, fora do caminho crítico da v0.3, como as revisões da v0.1 ficaram na v0.1 (P10 da v0.2). A exceção é a T8 da v0.2, que vira a T1 desta spec.

## Premissas

- **A1:** um ID de requisito (`[A-Z]{2,5}-\d{2}` no `exban`) é **único dentro de uma spec**, não no projeto nem no workspace. Nada no formato impede duas specs do mesmo projeto de usarem o mesmo ID (`DEB-01`, por exemplo). As specs do Quire evitam a colisão só por convenção, continuando a numeração de uma versão para a outra, e as fixtures atuais também não têm colisão. Por isso um ID sem qualificação só é resolvido dentro da mesma spec (REF-03). A forma qualificada entre specs depende do autor (Q1).
- **A2:** a definição de um ID é o comentário `<!-- ID -->` no `spec.md` (`parse::ids`, v0.1). Os usos são: o campo `**Requirement**` das seções de tarefa, a primeira coluna da tabela de critérios do `validation.md`, o `source` do `lessons.jsonl` quando é um ID, e o ID citado em texto corrido de qualquer `.md` da spec.
- **A3:** uma chave de issue é definida pelo nome da pasta da spec (`issue_key_pattern` do preset) e usada em texto corrido de qualquer `.md`. A mesma chave pode existir em mais de um projeto (v0.1, CLI-03); a resolução nunca escolhe sozinha entre projetos (REF-03).
- **A4:** `AD-xxx` é definido no `.specs/STATE.md` da raiz do repositório da instância. O `spec-format.md` só diz "decisões do projeto (AD-xxx)". Esta spec assume que a definição é a primeira ocorrência do ID num título (`#`…) ou no começo de um item de lista ou de uma linha de tabela; as demais ocorrências são usos. Confirmar com o autor (Q8).
- **A5:** `arquivo:linha` é relativo à raiz do repositório da instância, como no `spec-format.md`. Um caminho que sai da raiz (`..`) ou é absoluto não é resolvido.
- **A6:** as referências entre arquivos vêm do texto em disco. O índice de referências mora no mesmo cache SQLite descartável (TD-06 da v0.1), e apagá-lo só custa uma reindexação.
- **A7:** o histórico usa o `git` instalado no sistema, o mesmo que a v0.1 já usa para identidade do projeto e worktrees (`git.rs`, timeout padrão de 5 s). Sem `git`, o histórico fica indisponível, e todo o resto funciona.
- **A8:** o lint não altera nada: não escreve em arquivo de spec, não escreve no repositório e não exige o índice. O `quire lint` roda numa pasta recém-clonada, sem cache, sem rede e sem janela.
- **A9:** a escrita de um arquivo exportado é uma ação explícita do usuário num caminho que ele escolhe num diálogo do sistema. O app não escolhe caminho nem grava export em segundo plano.
- **A10:** criar um arquivo segue as mesmas garantias de escrita da v0.2 (TD-12, TD-22): criação exclusiva, nunca por cima de um arquivo que já existe, e bloqueada nas classes de sistema de arquivos sem rename atômico verificado.

## Perguntas em aberto (dependem do autor)

Cada pergunta traz o padrão conservador que a execução usa se ela não for respondida. Nenhuma tarefa decide por conta própria uma pergunta que muda o escopo.

- **Q1 — Referência a um ID de outra spec.** Os IDs são únicos só dentro da spec (A1). Deve existir uma forma qualificada no formato, como `ACME-142/CCR-03`? Isso muda o `spec-format.md` e o que os agentes escrevem. **Padrão:** sem forma qualificada; ID nu resolve só na própria spec, e a ligação entre specs é pela chave de issue.
- **Q2 — Silenciar uma regra do lint dentro do arquivo.** Deve existir um comentário como `<!-- quire-lint: ignore unknown-id -->`? Também muda o formato. **Padrão:** silenciar só pela configuração (preset, projeto ou spec no `config.toml`), sem nada no arquivo.
- **Q3 — `quire lint` no CI de outro projeto, no Linux.** O binário do app no Linux depende do WebKitGTK, pesado para um runner de CI. Recomendação: publicar o `quire-cli` (hoje só o auxiliar de console do Windows, TD-20) também para Linux, como binário de linha de comando sem Tauri, anexado à release. Isso muda a distribuição e o papel do `quire-cli` (TD-29). **Padrão sem resposta:** o `quire lint` existe no binário do app (Linux) e no `quire-cli` (Windows), e o binário avulso para Linux não é publicado.
- **Q4 — Criar spec nova.** Entra na v0.3 além de criar arquivo ausente? Exige decidir repositório, worktree, branch, nome da pasta (chave + slug) e quais arquivos nascem. **Padrão:** fora da v0.3 (backlog da v0.4).
- **Q5 — Export de `gates.jsonl` e `lessons.jsonl`.** Eles têm `cwd` com caminhos absolutos e saída de comandos. Exportar com os caminhos mascarados, exportar como estão com aviso, ou não exportar? **Padrão:** não exportar (EXP-08).
- **Q6 — Assinatura do MSI.** Depois da avaliação da T2: não assinar e documentar o aviso do SmartScreen; certificado pago (custo recorrente); ou serviço de assinatura para projetos open source (exige aprovação de terceiros). **Padrão:** não assinar na v0.3 e documentar o SmartScreen no README.
- **Q7 — A Fase 0 bloqueia as features se a rodada no WSL não acontecer?** A T1 depende do autor numa máquina com WSL. Bloquear segura toda a v0.3. Não bloquear mantém `\\wsl$`, DrvFs e 9p somente leitura, que é o lado seguro. **Padrão:** a T1 não bloqueia as fases seguintes. As classes continuam somente leitura até haver dado, e a T1 fica aberta como pendência.
- **Q8 — Onde um `AD-xxx` é definido no `STATE.md`** (A4). Um exemplo real (sem dado da empresa) ou a regra basta. **Padrão:** a regra de A4.
- **Q9 — Pendências da v0.2 antes da execução da v0.3.** Fechar a `v0.2.0` (revisões, E2E, orçamento de memória da T6, tag final) antes de começar a v0.3, ou executar a v0.3 em paralelo? **Padrão:** em paralelo, como a v0.2 fez com as pendências da v0.1 (P10 da v0.2).
- **Q10 — Feedback de uso.** A spec nasce sem feedback de uso da v0.2. Se o autor usar a `v0.2.0-rc.1` antes da aprovação, o feedback pode reordenar as fases ou cortar escopo. **Padrão:** a spec segue como está, com o risco registrado.

## Histórias de usuário

### P1: Referências e backlinks ⭐ MVP

**User Story**: Como dev revisando uma spec, quero clicar num ID, numa chave de issue ou numa decisão e ver onde ela é definida e onde é citada, para seguir um critério até a evidência sem procurar texto aba por aba.

**Critérios de aceite**:

1. QUANDO um projeto usa um preset ENTÃO os tipos de referência reconhecidos DEVEM vir do preset, cada um com a gramática de definição e de uso: ID de requisito, chave de issue, decisão `AD-xxx` e `arquivo:linha`. O preset `generic` reconhece só `arquivo:linha` e links relativos. O `config.toml` pode sobrescrever a gramática de um preset, como já faz com `id_pattern`. Nada disso fica fixo no núcleo. <!-- REF-01 -->
2. QUANDO o índice é construído ou reconciliado ENTÃO o sistema DEVE extrair as definições e os usos de referência dos arquivos `.md` (e do `source` do `lessons.jsonl`), fora de blocos de código, no cache SQLite descartável, reindexando só o arquivo que mudou, na mesma reconciliação incremental da busca (SRC-02). <!-- REF-02 -->
3. QUANDO o sistema resolve uma referência ENTÃO DEVE aplicar o escopo do tipo: um ID nu resolve só dentro da mesma instância da spec (A1, Q1); um `AD-xxx` resolve no `.specs/STATE.md` do repositório da instância (A4); `arquivo:linha` resolve a partir da raiz do repositório da instância (A5); uma chave de issue resolve primeiro no mesmo projeto e depois nos outros, e QUANDO há mais de um destino ENTÃO o sistema DEVE listar todos e NÃO DEVE escolher sozinho. <!-- REF-03 -->
4. QUANDO o usuário clica numa referência resolvida (no texto do modo Ler, num chip de requisito da aba tasks, na tabela do `validation.md` ou no `source` de uma lesson) ENTÃO o sistema DEVE abrir o destino: a spec, a instância e a aba certas, rolada até a linha, com a linha destacada. <!-- REF-04 -->
5. QUANDO o usuário pede "Referenciado em" para um ID, uma decisão ou a spec aberta ENTÃO o sistema DEVE listar os usos agrupados por arquivo, na ordem spec → tasks → validation → lessons → outros, com linha e trecho; para uma spec, DEVE listar as outras specs que citam a chave dela, marcando as de outro projeto. <!-- REF-05 -->
6. QUANDO o usuário está num critério do `spec.md` ENTÃO o sistema DEVE oferecer o caminho até a evidência: as tarefas que o citam em `**Requirement**`, com o estado e os hashes delas; a linha do `validation.md` com o resultado e a evidência; e as linhas do `gates.jsonl` cujo `head` começa com o hash de uma dessas tarefas. <!-- REF-06 -->
7. QUANDO o usuário navega por uma referência ENTÃO o sistema DEVE guardar o ponto de partida num histórico em memória e oferecer voltar e avançar: Alt+← e Alt+→, os botões laterais do mouse e botões na barra. Voltar DEVE restaurar a spec, a instância, a aba e a rolagem. Toda navegação que troca de arquivo DEVE passar pela guarda de alterações não salvas (SAV-06). <!-- REF-07 -->
8. QUANDO uma referência não resolve (ID inexistente na spec, `AD-xxx` sem definição, arquivo ausente, linha além do fim, chave de issue desconhecida) ENTÃO o sistema DEVE mostrá-la com estilo de quebrada e o motivo no rótulo acessível, e o clique NÃO DEVE navegar. <!-- REF-08 -->
9. QUANDO o usuário clica num `arquivo:linha` que aponta para um arquivo da pasta da spec ENTÃO o sistema DEVE abri-lo na aba, na linha; QUANDO aponta para outro arquivo do repositório (código) ENTÃO o sistema DEVE oferecer "Abrir no aplicativo padrão" e "Copiar `caminho:linha`", sem abrir nada que esteja fora da raiz do repositório. <!-- REF-09 -->
10. QUANDO há cerca de 300 specs indexadas ENTÃO "Referenciado em" DEVE responder em p95 ≤ 50 ms, reindexar as referências de um arquivo DEVE custar ≤ 10 ms, e as tabelas de referência DEVEM ocupar no máximo 0,5× o texto markdown indexado. Os números são confirmados ou ajustados pelo spike T5. <!-- REF-10 -->

### P1: Lint de estrutura ⭐ MVP

**User Story**: Como dev, quero que o Quire aponte o que está quebrado na estrutura da spec (critério sem tarefa, ID que não existe, progresso desatualizado), no app e no CI do projeto, sem me afogar em avisos falsos.

**Critérios de aceite**:

1. QUANDO o lint roda ENTÃO NÃO DEVE escrever em nenhum arquivo de spec nem no repositório; os resultados ficam em memória ou no cache descartável. <!-- LNT-01 -->
2. QUANDO o lint analisa uma spec ENTÃO DEVE aplicar as regras abaixo, cada uma com um nome estável em inglês e uma severidade padrão: `duplicate-id` (ID definido duas vezes no `spec.md`; erro); `unknown-id` (`**Requirement**`, `validation.md` ou texto cita ID que a spec não define; erro); `unknown-dependency` (`**Depende de**` cita tarefa inexistente; erro); `criterion-without-task` (critério que nenhuma tarefa cita; aviso); `task-without-requirement` (seção de tarefa sem `**Requirement**`; aviso); `task-without-done-when` (seção de tarefa sem `**Pronto quando**`; aviso); `broken-reference` (`AD-xxx` sem definição; `arquivo:linha` com arquivo ausente ou linha além do fim; aviso); `dead-link` (link relativo para arquivo ou âncora inexistente; aviso); `progress-mismatch` (PRG-02; aviso); `criterion-without-evidence` (critério sem linha no `validation.md` quando ele existe; informação). <!-- LNT-02 -->
3. QUANDO o preset ou o `config.toml` define severidades ENTÃO o sistema DEVE usá-las, na ordem preset → projeto → spec, com os níveis `error`, `warning`, `info` e `off`; QUANDO a configuração cita uma regra ou um nível desconhecido ENTÃO o sistema DEVE ignorar a entrada e avisar nas Preferências, sem invalidar a config e sem regravar o arquivo. <!-- LNT-03 -->
4. QUANDO uma spec tem problemas ENTÃO o sistema DEVE mostrar um painel de problemas com severidade, regra, mensagem no idioma da interface e `arquivo:linha`; o clique DEVE abrir o arquivo na linha (REF-04). A contagem de erros e avisos DEVE aparecer no cabeçalho da spec e na lista lateral; informação não conta no selo. <!-- LNT-04 -->
5. QUANDO um arquivo da spec muda (no app ou por fora) ENTÃO o sistema DEVE refazer o lint só daquela spec, em segundo plano, e atualizar o painel em até 1 s depois da reconciliação, sem travar a interface. <!-- LNT-05 -->
6. QUANDO uma regra depende de uma estrutura que a spec não tem ENTÃO ela NÃO DEVE disparar: sem `tasks.md` reconhecido, nada das regras de tarefa; sem `validation.md`, nada de evidência; preset `generic`, só `dead-link` e `broken-reference` de arquivo. As fixtures limpas DEVEM produzir zero problemas, e as fixtures com defeitos plantados DEVEM produzir exatamente a lista esperada (teste de falso positivo e de falso negativo). <!-- LNT-06 -->
7. QUANDO o usuário roda `quire lint [caminho…]` ENTÃO o sistema DEVE analisar as specs encontradas sob o caminho (padrão: diretório atual) sem abrir janela, sem usar nem criar o índice, sem rede e sem escrever nada; DEVE imprimir uma linha por problema (`arquivo:linha: severidade regra mensagem`) ou JSON com `--format json`; e DEVE sair com 0 sem problemas no limite, 1 com problemas no limite (`--fail-on error` por padrão, ou `warning`), 2 para erro de uso e 3 quando não consegue ler o caminho ou a config. A saída e os códigos DEVEM ser iguais no Linux e no terminal do Windows. <!-- LNT-07 -->
8. QUANDO a contagem de tarefas por fase do `tasks.md` difere da tabela de progresso do `summary.md` (entre `<!-- progress:start -->` e `<!-- progress:end -->`) ENTÃO a regra `progress-mismatch` DEVE apontar a linha da tabela e mostrar os dois valores; sem a tabela, a regra não dispara. <!-- PRG-02 -->
9. QUANDO há cerca de 300 specs ENTÃO o lint de uma spec DEVE custar ≤ 50 ms e o de todas, em segundo plano na abertura, ≤ 2 s, sem atrasar a primeira tela. Os números são confirmados ou ajustados pelo spike T8. <!-- LNT-08 -->

### P2: Histórico e diff via git

**User Story**: Como dev, quero ver como a spec mudou, abrir uma versão antiga de um arquivo e comparar versões e cópias, sem sair do app e sem correr o risco de mexer no repositório.

**Critérios de aceite**:

1. QUANDO o usuário abre o histórico de uma spec ENTÃO o sistema DEVE listar os commits que tocaram a pasta da spec, a partir do `HEAD` da instância, com hash curto, data, autor e assunto, em páginas de 50. <!-- HIS-01 -->
2. QUANDO o usuário abre um arquivo num commit ENTÃO o sistema DEVE mostrá-lo renderizado como no modo Ler, somente leitura, com uma faixa "commit `<hash>` · `<data>`" e a ação de voltar à versão atual. <!-- HIS-02 -->
3. QUANDO o usuário escolhe dois commits, ou um commit e a versão atual em disco, ENTÃO o sistema DEVE mostrar as diferenças por linha de cada arquivo da spec que mudou entre eles. <!-- HIS-03 -->
4. QUANDO o usuário clica num hash de tarefa (aba tasks) ou num `head` de gate (aba gates) ENTÃO o sistema DEVE abrir o commit: mensagem, data, autor, a lista de arquivos alterados e o diff dos arquivos da pasta da spec; QUANDO o hash não existe no repositório (clone raso, outro repositório) ENTÃO DEVE dizer "commit não encontrado" sem erro genérico. <!-- HIS-04 -->
5. QUANDO o sistema chama o `git` ENTÃO DEVE usar só subcomandos de leitura de uma lista fechada, sem pager, sem editor, sem prompt de credencial, sem diff externo nem `textconv`, sem `fsmonitor` e sem locks opcionais; um teste DEVE provar que o repositório (inclusive `.git/index`) fica byte a byte igual depois de todas as operações de histórico. <!-- HIS-05 -->
6. QUANDO o `git` passa do tempo limite ENTÃO o sistema DEVE encerrar o processo, mostrar o que já tinha e oferecer tentar de novo; QUANDO o usuário sai do painel ENTÃO a chamada em andamento DEVE ser cancelada; QUANDO o `git` não existe ou a pasta não está num repositório ENTÃO o painel DEVE mostrar o motivo e o resto do app NÃO DEVE ser afetado. <!-- HIS-06 -->
7. QUANDO a spec tem cópias divergentes ENTÃO o seletor de instâncias DEVE oferecer "Comparar" entre duas instâncias, com a lista de arquivos (iguais, diferentes, só numa delas) e o diff por linha de cada arquivo, a partir do conteúdo em disco, sem depender do `git` (wireframe 4). <!-- HIS-07 -->
8. QUANDO o repositório é grande ENTÃO a primeira página do histórico DEVE aparecer em p95 ≤ 1 s e abrir um arquivo num commit em p95 ≤ 300 ms, nas condições do spike T6, que confirma ou ajusta os números e o tempo limite. <!-- HIS-08 -->

### P2: Export para HTML e PDF

**User Story**: Como dev, quero mandar uma spec para alguém que não tem o Quire, num arquivo que abre em qualquer navegador ou como PDF, sem vazar caminhos da minha máquina.

**Critérios de aceite**:

1. QUANDO o usuário exporta a spec aberta para HTML ENTÃO o sistema DEVE gerar um arquivo único com os arquivos markdown escolhidos (padrão: os primários, na ordem das abas), CSS embutido, Mermaid como SVG embutido, código com realce em estilo embutido, sem JavaScript e sem nenhum recurso externo. O cabeçalho do export DEVE ter título, chave, projeto e data do export. <!-- EXP-01 -->
2. QUANDO o sistema gera um export ENTÃO o que o próprio app acrescenta NÃO DEVE conter caminho absoluto, nome de usuário, pasta pessoal, raiz do repositório nem caminho da config; QUANDO o conteúdo do usuário contém a pasta pessoal ou a raiz do repositório ENTÃO o sistema DEVE listar as ocorrências antes de gravar, com "Exportar mesmo assim" e "Cancelar", e NÃO DEVE alterar o conteúdo em silêncio. <!-- EXP-02 -->
3. QUANDO o export tem links ENTÃO um link relativo para um arquivo exportado DEVE virar âncora interna, um link para arquivo não exportado DEVE virar texto com o nome do arquivo (sem caminho), `http(s)` DEVE ficar como link, e referências e `arquivo:linha` DEVEM virar texto. <!-- EXP-03 -->
4. QUANDO o usuário confirma o export ENTÃO o sistema DEVE gravar só no caminho escolhido num diálogo de salvar aberto pelo backend; a escrita DEVE ser atômica (temporário na mesma pasta e rename); sobrescrever DEVE exigir a confirmação do diálogo; cancelar NÃO DEVE gravar nada. <!-- EXP-04 -->
5. QUANDO o usuário exporta para PDF ENTÃO o sistema DEVE usar o mecanismo decidido no spike T7, com tema claro, quebras de página que evitam partir tabelas, blocos de código e diagramas quando cabem numa página, e títulos sem ficar órfãos no pé; QUANDO o mecanismo não funciona numa plataforma ENTÃO a opção PDF DEVE ficar indisponível ali, com o motivo, e o HTML continua. <!-- EXP-05 -->
6. QUANDO o mesmo export é feito no Linux (WebKitGTK) e no Windows (WebView2) ENTÃO o conteúdo DEVE ser o mesmo (texto, tabelas, diagramas e ordem) e as fontes DEVEM ser as embutidas no app (DEB-13); as diferenças de paginação observadas DEVEM ficar registradas. <!-- EXP-06 -->
7. QUANDO a spec tem 40 KB com 6 diagramas ENTÃO o export HTML DEVE terminar em ≤ 3 s e o arquivo DEVE ter ≤ 2 MB, nas condições do spike T7. <!-- EXP-07 -->
8. QUANDO o usuário escolhe os arquivos do export ENTÃO `gates.jsonl`, `lessons.jsonl` e os arquivos que não são `.md` NÃO DEVEM aparecer como opção (Q5). <!-- EXP-08 -->

### P3: Criar arquivo ausente

**User Story**: Como dev, quero criar o `validation.md` (ou outro arquivo primário que falta) de uma spec já existente a partir do modelo do preset, sem sair do app.

**Critérios de aceite**:

1. QUANDO um arquivo `.md` primário previsto pelo preset não existe na instância ENTÃO a aba dele DEVE mostrar "Este arquivo não existe nesta cópia" com a ação explícita "Criar `<arquivo>`". <!-- NEW-01 -->
2. QUANDO o usuário cria o arquivo ENTÃO o sistema DEVE gravar o modelo do preset para aquele arquivo, com `{key}` e `{title}` preenchidos a partir da spec, por criação exclusiva: QUANDO o arquivo passou a existir nesse meio-tempo ENTÃO o sistema NÃO DEVE sobrescrever e DEVE abrir o existente com um aviso. A criação DEVE respeitar SAV-04 (pasta sem permissão) e SAV-11 (classe de sistema de arquivos bloqueada). <!-- NEW-02 -->
3. QUANDO um comando de criação chega ao núcleo para um nome que não é um `.md` primário do preset, para `gates.jsonl` ou `lessons.jsonl`, para fora da pasta da instância ou para uma pasta de spec nova ENTÃO o núcleo DEVE recusar, independentemente da UI (Q4). <!-- NEW-03 -->
4. QUANDO o arquivo é criado ENTÃO o sistema DEVE abri-lo no modo Editar com o modelo, sem alterações pendentes, e a lista, o índice e o lint DEVEM refletir o arquivo novo em até 2 s, sem aviso de conflito (CNF-06). <!-- NEW-04 -->

### P1: Dívidas da v0.2 (Fase 0)

**User Story**: Como usuário da v0.2, quero que as decisões de segurança e de distribuição que ficaram abertas sejam fechadas antes de a v0.3 crescer.

**Critérios de aceite**:

1. QUANDO o autor roda as rodadas no WSL (spikes T2 e T3 da v0.1 no WSLg e no WSL2 em ext4, DrvFs e `\\wsl$`; `rename_spike` em DrvFs, 9p e `\\wsl$`) ENTÃO os resultados DEVEM entrar nas tabelas do `summary.md`, a `VERIFIED_ATOMIC` DEVE mudar só para as classes com rename atômico medido (TD-22), o TD-05 DEVE ser confirmado ou ajustado, e a pasta `apps/desktop/src/spike/` DEVE ser removida. Sem rodada, as três classes continuam somente leitura (Q7). <!-- DEB-11 -->
2. QUANDO a T2 termina ENTÃO a decisão sobre a assinatura do MSI DEVE estar registrada como TD novo, com as opções avaliadas (custo, processo, segredo no CI, reputação no SmartScreen); QUANDO a decisão for assinar ENTÃO a release DEVE assinar só em tag do repositório principal, nunca em PR de fork, e o CI DEVE verificar a assinatura; QUANDO for não assinar ENTÃO o README DEVE explicar o aviso do SmartScreen e como prosseguir (Q6). <!-- DEB-12 -->
3. QUANDO o app abre ENTÃO DEVE usar as fontes Inter e JetBrains Mono embutidas (woff2, licença OFL incluída no repositório), sem rede, nos dois temas e na impressão; o crescimento do bundle DEVE ser medido e ficar ≤ 400 KB. <!-- DEB-13 -->
4. QUANDO uma spec abre, ou o usuário volta a ela pelo histórico de navegação ENTÃO o outline DEVE destacar a seção visível desde o primeiro render, sem precisar rolar até um título (pendência da T16 da v0.1). <!-- DEB-14 -->

### P2: Projeto open source

**User Story**: Como mantenedor, quero que a documentação e o CI acompanhem as novas capacidades.

**Critérios de aceite**:

1. O README DEVE documentar referências e navegação, o lint (regras, severidades, configuração, `quire lint` com exemplo de uso em CI e os códigos de saída), o histórico (exige `git`; garantias de somente leitura), o export (o que entra, a verificação de vazamento, PDF por plataforma) e a criação de arquivo ausente; o `CHANGELOG.md` DEVE ganhar a v0.3.0. <!-- OSS-07 -->
2. QUANDO há push ou PR ENTÃO o CI DEVE rodar em Linux e Windows os testes de referências, lint (inclusive os códigos de saída do `quire lint`), histórico (inclusive o de repositório intacto) e export (inclusive o de vazamento), e DEVE rodar o `quire lint` sobre as próprias `.specs` do Quire. Esse passo fica informativo até o autor decidir torná-lo bloqueante. <!-- OSS-08 -->

## Casos de borda

- **Mesmo ID em duas specs do mesmo projeto** (`TOK-01` em duas specs, plantado nas fixtures pela T22): cada um resolve na própria spec (REF-03). "Referenciado em" nunca mistura os dois.
- **ID citado em texto corrido de outra spec** (`ver CCR-03 da ACME-170`): sem forma qualificada (Q1), o ID não resolve fora da própria spec. A chave `ACME-170` resolve e leva à spec.
- **Mesma chave de issue em dois projetos:** lista com os dois destinos e o projeto de cada um (REF-03), igual ao picker do `quire open`.
- **ID que aparece num bloco de código ou num exemplo:** ignorado pela extração (REF-02) e pelo lint.
- **ID definido duas vezes no `spec.md`:** `duplicate-id`; a navegação leva à primeira definição e lista as duas.
- **Spec com cópias divergentes:** as referências resolvem na instância exibida. Uma tarefa que só existe em outra cópia não aparece em "Referenciado em" até o usuário trocar de cópia.
- **`STATE.md` ausente:** todo `AD-xxx` vira referência quebrada (`broken-reference`), com o motivo "sem `.specs/STATE.md` neste repositório".
- **`arquivo:linha` com caminho absoluto, `..` que sai do repositório, ou caminho do Windows (`src\x.rs:10`):** absoluto e fora da raiz não resolvem. Barra invertida é normalizada antes de resolver.
- **`arquivo:linha` além do fim do arquivo:** quebrada, com o número de linhas do arquivo no motivo.
- **Link relativo com âncora (`design.md#arquitetura`):** a âncora é conferida contra os títulos do arquivo, com a mesma geração de âncoras do render.
- **Tabela de progresso do `summary.md` gerada por script mas desatualizada:** `progress-mismatch`. O lint não regera a tabela (LNT-01).
- **`tasks.md` com tarefas de fases de correção fora de ordem (`T26` entre `T20` e `T21`):** não é problema; nenhuma regra exige ordem.
- **Config com regra desconhecida (`[lint] unknwon-id = "off"`):** ignorada com aviso nas Preferências (LNT-03); o `quire lint` imprime o aviso em stderr e não muda o código de saída.
- **`quire lint` numa pasta sem `.specs`:** nenhuma spec, nenhuma saída de problema, código 0 e uma linha em stderr dizendo que nada foi encontrado.
- **`quire lint` com o app aberto:** roda no próprio processo e não encaminha para a instância aberta (o single-instance não se aplica ao subcomando).
- **Repositório sem commits, `HEAD` destacado, clone raso:** o histórico mostra o que o `git` devolve; num clone raso, o fim da lista diz "histórico truncado (clone raso)".
- **Spec renomeada (pasta movida) no histórico:** o `git log` da pasta atual não segue a pasta antiga; documentado como limite.
- **Arquivo da spec ignorado pelo `.gitignore` ou nunca commitado:** não aparece no histórico; o diff "commit × atual" mostra como "só na versão atual".
- **Repositório com `core.fsmonitor`, `diff.external` ou `textconv` configurados:** nenhum deles roda (HIS-05).
- **Repositório enorme e `git log` lento:** tempo limite, parcial e "tentar de novo" (HIS-06).
- **`git` ausente do `PATH`:** painel de histórico com o motivo; referências, lint e export funcionam.
- **Hash de tarefa ambíguo** (o prefixo casa com dois commits): "hash ambíguo" com as opções, sem escolher.
- **Export de spec com Mermaid inválido:** o bloco sai com a mensagem de erro do diagrama, como no modo Ler.
- **Export com o caminho da pasta pessoal num bloco de código do usuário:** listado antes de gravar (EXP-02); nada é mascarado sem o usuário pedir.
- **Destino do export dentro de uma pasta `.specs` observada:** permitido (é escolha explícita); o arquivo aparece em "outros" da spec pelo watcher.
- **Destino do export numa classe de sistema de arquivos bloqueada (`\\wsl$`):** a mesma regra do TD-22 vale para o export, porque a escrita é do mesmo pipeline.
- **Criar `validation.md` enquanto um agente o cria:** a criação exclusiva perde a corrida sem sobrescrever (NEW-02).
- **Criar arquivo numa instância numa classe bloqueada:** ação desabilitada com o motivo (NEW-02).
- **Voltar pelo histórico de navegação para uma spec que sumiu (worktree removida):** a entrada é pulada, com aviso.
- **Voltar com alterações não salvas:** guarda (REF-07).

## Rastreabilidade de requisitos

| Requisito | Tarefas |
| --- | --- |
| DEB-11 | T1 |
| DEB-12 | T2 |
| DEB-13 | T3, T7 |
| DEB-14 | T4 |
| REF-01, REF-02 | T9, T22 |
| REF-03 | T9, T10 |
| REF-04, REF-05 | T11 |
| REF-06 | T12 |
| REF-07 | T11 |
| REF-08 | T10, T11 |
| REF-09 | T12 |
| REF-10 | T5, T10 |
| LNT-01, LNT-02, LNT-03 | T13 |
| LNT-04, LNT-05 | T14 |
| LNT-06 | T13, T22, T23 |
| LNT-07 | T15 |
| LNT-08 | T8, T14 |
| PRG-02 | T13 |
| HIS-01 a HIS-04 | T16, T17 |
| HIS-05, HIS-06 | T16 |
| HIS-07 | T18 |
| HIS-08 | T6, T16 |
| EXP-01 a EXP-04 | T19 |
| EXP-05, EXP-06 | T7, T20 |
| EXP-07 | T7, T19 |
| EXP-08 | T19 |
| NEW-01 a NEW-04 | T21 |
| OSS-07 | T23 |
| OSS-08 | T15, T23 |

## Critérios de sucesso

- Partindo de um critério no `spec.md`, o usuário chega à tarefa que o cobre, à linha de evidência do `validation.md`, ao gate do commit da tarefa e ao diff desse commit, só com cliques, e volta ao critério com Alt+←.
- Nas fixtures limpas o lint aponta zero problemas; nas fixtures com defeitos plantados, aponta exatamente os plantados.
- `quire lint` roda no CI de um projeto de exemplo, sem rede e sem janela, e falha o job só com erro.
- Nenhuma operação de histórico muda um byte do repositório, inclusive `.git/index`.
- Um export HTML aberto noutra máquina, sem rede, mostra texto, tabelas, código e diagramas, e não contém nenhum caminho absoluto acrescentado pelo app.
- Criar um `validation.md` ausente nunca sobrescreve um arquivo existente.
- Tudo o que a v0.2 fazia continua funcionando, com o CI verde em Linux e Windows.
