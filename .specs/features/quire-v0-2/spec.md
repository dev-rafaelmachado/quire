# QUIRE v0.2 — Escrita segura, busca e command palette

**Idioma:** pt-BR

**Repositório:** `quire`: `crates/quire-core` (escrita, conflito, busca), `apps/desktop` (editor, busca, palette, shell Tauri), `fixtures/`
**Data:** 2026-11-16
**Status:** rascunho, aguardando aprovação do autor
**Complexidade:** Large. Verificação: `full` (escreve nos arquivos do usuário; edição concorrente com agentes; vários sistemas de arquivos).

## Fontes

- **Ideação e MoSCoW:** `docs/ideation/moscow.md` (fatiamento: v0.2 = "Escrita e busca"; critério de pronto "editar uma spec do começo ao fim só no app")
- **v0.1:** `.specs/features/quire-v0-1/` (spec, design, tasks e o Journal do `summary.md`, de onde saem as dívidas da Fase 0)
- **Formato das pastas de spec:** `docs/spec-format.md`
- **Design visual:** `docs/design/README.md`, `docs/design/tokens.css`, `docs/design/wireframes/*.html` (modo editar/dividir e command palette ainda não têm wireframe)
- **Convenções do projeto:** `AGENTS.md`

## Problema

A v0.1 lê specs de vários repositórios e worktrees, mas toda alteração ainda exige outro editor. Isso quebra o fluxo justamente onde o Quire mais ajuda: revisar e corrigir uma spec enquanto um agente (Claude Code, Codex) trabalha nos mesmos arquivos. Editar dentro do app traz um risco que a v0.1 não tinha: o Quire passa a escrever no disco, numa pasta onde outro processo também escreve. Um save errado apaga trabalho do agente ou do usuário. Também falta encontrar texto entre specs: com dezenas de specs em vários projetos, a lista e as abas não bastam.

## Objetivos

- Editar qualquer `.md` de uma spec com preview, do começo ao fim, sem abrir outro editor.
- Nunca perder dados: escrita atômica, formato do arquivo preservado, e nenhuma gravação por cima de uma mudança que o usuário não viu.
- Encontrar texto em todas as specs e navegar por teclado (busca full-text e command palette).
- Marcar tarefas no `tasks.md` quando o projeto optar por isso, com a mesma segurança da edição.
- Fechar as dívidas da v0.1 que atrapalham o uso diário.

## Fora do escopo

- **Edição de `gates.jsonl` e `lessons.jsonl`:** continuam somente leitura (escritos por agente; Won't do MoSCoW).
- **Edição de arquivos que não são `.md`:** `.json`, `.txt` e outros continuam somente leitura.
- **Itens de outras releases:**
  - **Should do MoSCoW fora do fatiamento da v0.2:** criar spec ou arquivo a partir de template (ver P1), `arquivo:linha` clicável, abrir no `$EDITOR`, diff entre instâncias, journal com quick-add, copiar contexto para um agente, indicadores de git.
  - **v0.3+:** lint de estrutura, backlinks, histórico git por spec, export PDF/HTML.
  - **Won't:** kanban, Jira por API, MCP, sync em nuvem, colaboração em tempo real, editor WYSIWYG.
- **Autosave e rascunhos:** autosave fica desligado e não há opção para ligá-lo (ver P2); rascunhos fora do disco não existem (ver P3).
- **Trava cooperativa com agentes** (lock de arquivo): os agentes não respeitam nenhuma; a proteção é a detecção de conflito (CNF).

## Premissas e perguntas em aberto

- **A1:** só arquivos `.md` são editáveis. JSONL é sempre somente leitura, inclusive no núcleo (o comando de escrita recusa), não só na UI.
- **A2:** o app não guarda o conteúdo de um buffer fora do disco (nada em `localStorage`, nada no índice), seguindo o `AGENTS.md`. Fechar o app com alterações não salvas exige confirmação (SAV-06).
- **A3:** a identidade do conteúdo carregado é o hash do conteúdo bruto (xxh3-128 dos bytes). `mtime` e tamanho servem só como atalho. Um `mtime` igual não prova que o conteúdo é o mesmo, e um diferente não prova que mudou.
- **A4:** quando um arquivo tem fins de linha mistos (CRLF e LF), salvar normaliza para o tipo predominante. O editor avisa ao abrir e o primeiro save pede confirmação (SAV-03).
- **A5:** a busca ignora acentos e maiúsculas, sem stemming: "ação" acha "acao", mas "tarefa" não acha "tarefas" sem o prefixo (a busca é por prefixo no último termo).
- **A6:** não existe trava compartilhada com agentes. Resta uma janela de corrida entre a última verificação e o rename (CNF-03), que é documentada e reduzida ao mínimo, mas não pode ser eliminada.
- **A7:** a edição sempre escreve na instância exibida (a do seletor da v0.1); nada se propaga para outras cópias.
- **A8:** o limite de tamanho para edição começa no mesmo da v0.1 (`LARGE_FILE_BYTES` = 2 MB) e o spike T9 confirma ou ajusta.
- **A9:** o idioma da interface é escolhido em `config.toml` (`language`). Sem valor, segue o idioma do sistema: `pt*` usa pt-BR e os demais usam inglês (ver P8).

**Perguntas em aberto (decisões do autor; alteram escopo ou comportamento visível):**

- **P1 (aberta):** "editar uma spec do começo ao fim" inclui **criar** a spec ou um arquivo novo dentro dela (`validation.md` que ainda não existe, por exemplo)? O MoSCoW põe "criar spec a partir de template" em Should, fora do fatiamento da v0.2. Esta spec assume que não: só arquivos existentes são editáveis.
- **P2 (aberta):** deve existir autosave **opcional** (opt-in por projeto, desligado por padrão), ou nenhum autosave na v0.2? Esta spec assume nenhum.
- **P3 (aberta):** recuperação do buffer após um crash do app exigiria guardar o texto fora do disco, o que conflita com o `AGENTS.md` ("o app nunca guarda estado de spec fora do disco"). Abrir exceção para rascunhos temporários (apagados ao salvar ou descartar), ou manter a regra e aceitar a perda do buffer num crash?
- **P4 (aberta):** marcar uma tarefa pelo app muda **só** o caractere do checkbox, ou também acrescenta uma anotação (por exemplo, `(marcada no Quire)`)? No fluxo de origem quem marca é o agente, com o hash do commit. Esta spec assume só o caractere.
- **P5 (aberta):** o alerta de divergência entre o progresso do `tasks.md` e a tabela de progresso do `summary.md` (Should do MoSCoW, PRG-02) entra na v0.2? O parser (`parse::summary_progress`) já existe e não é usado.
- **P6 (aberta, depende do spike T11):** se o rename atômico não for confiável em `\\wsl$`, DrvFs ou 9p, as opções são (a) edição somente leitura nesses caminhos, com aviso, ou (b) escrita não atômica com aviso explícito e confirmação. Esta spec assume (a) até a decisão.
- **P7 (aberta):** a busca indexa também o texto de `lessons.jsonl` e `gates.jsonl` (campo `text`, comando, `tail`), ou só markdown? Esta spec assume só markdown.
- **P8 (aberta):** o idioma padrão segue o sistema (A9) ou é pt-BR sempre, com inglês só por escolha?
- **P9 (aberta):** para o console do CLI no Windows (DEB-06), vale incluir no instalador um segundo binário pequeno (console) ao lado do `quire.exe`? A alternativa é uma dependência que anexa o console ao processo pai. Detalhes em TD-20.
- **P10 (aberta):** as revisões visuais pendentes da v0.1 (T14, T16, T18, T19, T20, T22, T24), o merge do PR e a tag `v0.1.0` precisam terminar antes de a v0.2 começar? Esta spec assume que sim para o merge e que não para a tag.

## Histórias de usuário

### P1: Editar com preview ⭐ MVP

**User Story**: Como dev revisando uma spec, quero editar o markdown dentro do Quire e ver o resultado renderizado, para corrigir a spec sem trocar de ferramenta.

**Critérios de aceite**:

1. QUANDO um arquivo `.md` está aberto ENTÃO o sistema DEVE oferecer os modos Ler, Editar e Dividir (editor e preview lado a lado), com Ler como padrão. <!-- EDT-01 -->
2. QUANDO o usuário entra em Editar ou Dividir pela primeira vez na sessão ENTÃO o sistema DEVE carregar o editor sob demanda; o bundle inicial NÃO DEVE crescer por causa do editor. <!-- EDT-02 -->
3. QUANDO o usuário digita no modo Dividir ENTÃO o preview DEVE refletir o texto em até 500 ms depois de uma pausa de 200 ms na digitação, com o mesmo render do modo Ler (GFM, realce, Mermaid, colapso). <!-- EDT-03 -->
4. QUANDO o usuário rola o editor ou o preview no modo Dividir ENTÃO o outro lado DEVE acompanhar, alinhando o bloco correspondente (título, parágrafo, tabela ou bloco de código). <!-- EDT-04 -->
5. QUANDO o usuário pressiona Ctrl+S (ou o botão Salvar) com alterações ENTÃO o sistema DEVE salvar pela escrita segura (SAV) e conflito (CNF); sem alterações, NÃO DEVE escrever nada nem alterar o `mtime`. <!-- EDT-05 -->
6. QUANDO há alterações não salvas ENTÃO o sistema DEVE indicá-las na aba do arquivo e no título da janela. <!-- EDT-06 -->
7. QUANDO o usuário edita ENTÃO o editor DEVE oferecer realce de markdown, quebra de linha, desfazer/refazer, localizar e substituir no arquivo, e aceitar entrada em pt-BR (acentos por tecla morta, composição e IME) sem perder caracteres. <!-- EDT-07 -->
8. QUANDO o arquivo não é `.md`, é `gates.jsonl` ou `lessons.jsonl`, não é UTF-8 válido, é binário ou passa do limite de tamanho (A8) ENTÃO os modos Editar e Dividir DEVEM ficar desabilitados, com cadeado e o motivo. <!-- EDT-08 -->
9. QUANDO o editor está aberto num arquivo de 40 KB ENTÃO a latência de tecla até a tela DEVE ficar em p95 ≤ 50 ms, e abrir o editor DEVE levar menos de 500 ms com o código já carregado (medidos no spike T9 e na T16). <!-- EDT-09 -->

### P1: Escrita segura ⭐ MVP

**User Story**: Como dev, quero ter certeza de que salvar no Quire nunca corrompe nem trunca um arquivo, e que o arquivo continua com o mesmo formato que o agente e o git esperam.

**Critérios de aceite**:

1. QUANDO o sistema salva um arquivo ENTÃO DEVE escrever o conteúdo num arquivo temporário na mesma pasta, forçar a gravação em disco (fsync), renomear por cima do original e, onde o SO permite, forçar a gravação da pasta. Em nenhum momento o original pode ficar truncado ou parcial. <!-- SAV-01 -->
2. QUANDO a escrita falha em qualquer etapa (disco cheio, permissão, arquivo em uso) ENTÃO o original DEVE continuar intacto, o temporário criado nessa ação DEVE ser removido, o buffer DEVE continuar com as alterações e o motivo DEVE aparecer para o usuário. <!-- SAV-02 -->
3. QUANDO o sistema salva ENTÃO DEVE preservar o tipo de fim de linha (LF ou CRLF), a presença de BOM UTF-8 e a presença ou ausência de newline no fim do arquivo; QUANDO o arquivo tem fins de linha mistos ENTÃO o sistema DEVE avisar ao entrar em edição e pedir confirmação no primeiro save (A4). <!-- SAV-03 -->
4. QUANDO o sistema salva ENTÃO DEVE preservar as permissões do arquivo (modo no Unix, atributo somente leitura no Windows); QUANDO o arquivo é somente leitura ou pertence a outro usuário ENTÃO o sistema NÃO DEVE salvar e DEVE oferecer "Salvar como cópia". <!-- SAV-04 -->
5. QUANDO o arquivo é um link simbólico ENTÃO o sistema DEVE escrever no arquivo de destino, mantendo o link; o temporário DEVE ficar na pasta do destino. <!-- SAV-05 -->
6. QUANDO há alterações não salvas e o usuário troca de arquivo, de aba, de spec ou de instância, ou fecha a janela, ou uma navegação chega pelo CLI ENTÃO o sistema DEVE perguntar: Salvar, Descartar ou Cancelar. NADA DEVE ser escrito sem a escolha explícita. <!-- SAV-06 -->
7. QUANDO o sistema roda ENTÃO NÃO DEVE escrever em arquivos de spec por conta própria: sem autosave, sem escrita ao trocar de tela, sem escrita ao recarregar. Só Ctrl+S, o botão Salvar, uma escolha no diálogo de conflito ou de saída, ou o clique no checkbox (CHK) escrevem. <!-- SAV-07 -->
8. QUANDO o rename falha no Windows porque outro processo está com o arquivo aberto ENTÃO o sistema DEVE tentar de novo por até 1 s e, persistindo, DEVE falhar como em SAV-02 com o motivo "arquivo em uso". <!-- SAV-08 -->
9. QUANDO um comando de escrita chega ao núcleo para um arquivo que não é `.md` dentro da pasta da instância, ou para `gates.jsonl`/`lessons.jsonl`, ou com um caminho que sai da pasta da spec ENTÃO o núcleo DEVE recusar, independentemente da UI. <!-- SAV-09 -->
10. QUANDO sobra um arquivo temporário do Quire na pasta (um crash entre a escrita e o rename) ENTÃO o sistema NÃO DEVE listá-lo como arquivo da spec nem apagá-lo sozinho; DEVE mostrar um aviso na spec com a ação explícita de remover. <!-- SAV-10 -->

### P1: Conflito com edições externas ⭐ MVP

**User Story**: Como dev que edita uma spec enquanto um agente também a edita, quero que o Quire perceba quando o arquivo mudou por fora e me deixe decidir, em vez de sobrescrever o trabalho do agente.

**Critérios de aceite**:

1. QUANDO um arquivo é carregado para edição ENTÃO o sistema DEVE guardar a impressão digital do que foi lido: tamanho, `mtime` e hash do conteúdo bruto. <!-- CNF-01 -->
2. QUANDO o usuário salva e o conteúdo no disco difere do carregado ENTÃO o sistema NÃO DEVE escrever e DEVE mostrar o diálogo de conflito com: Recarregar do disco (descarta as alterações), Sobrescrever, Salvar como cópia e Ver diferenças. <!-- CNF-02 -->
3. QUANDO o sistema vai salvar ENTÃO DEVE comparar o disco com a impressão digital antes de escrever o temporário e de novo imediatamente antes do rename. A janela de corrida que resta (A6) DEVE estar documentada no design. <!-- CNF-03 -->
4. QUANDO o arquivo muda no disco e o buffer não tem alterações ENTÃO o sistema DEVE recarregar o editor e o preview (WAT-01, WAT-02), mantendo o modo e, até onde o texto permitir, o cursor e a rolagem. <!-- CNF-04 -->
5. QUANDO o arquivo muda no disco e o buffer tem alterações ENTÃO o sistema NÃO DEVE recarregar e DEVE mostrar um aviso fixo, sem bloquear a edição, "o arquivo mudou no disco", com as mesmas opções de CNF-02. <!-- CNF-05 -->
6. QUANDO a mudança no disco foi feita pelo próprio app (save ou checkbox) ENTÃO o sistema NÃO DEVE recarregar o editor, NÃO DEVE mostrar aviso de conflito e NÃO DEVE entrar em ciclo de recarga; a lista e o índice DEVEM refletir a mudança (WAT-01). <!-- CNF-06 -->
7. QUANDO o arquivo é apagado no disco durante a edição ENTÃO o sistema DEVE manter o buffer, avisar, e só recriar o arquivo se o usuário salvar e confirmar a recriação. <!-- CNF-07 -->
8. QUANDO o usuário escolhe "Salvar como cópia" ENTÃO o sistema DEVE gravar o buffer num arquivo novo na mesma pasta (`<nome>.conflito-AAAAMMDD-HHMMSS.md`), pela escrita segura, sem nunca sobrescrever um arquivo existente. <!-- CNF-08 -->
9. QUANDO o usuário escolhe "Ver diferenças" ENTÃO o sistema DEVE mostrar as diferenças por linha entre o buffer e o disco, e voltar ao diálogo sem perder nada. <!-- CNF-09 -->

### P1: Busca full-text ⭐ MVP

**User Story**: Como dev com muitas specs, quero buscar um termo em todas elas e ir direto ao trecho, em vez de abrir spec por spec.

**Critérios de aceite**:

1. QUANDO o índice é construído ou reconciliado ENTÃO o sistema DEVE indexar o texto dos arquivos `.md` das specs no próprio cache SQLite, que continua descartável (IDX-01). <!-- SRC-01 -->
2. QUANDO um arquivo muda ENTÃO o sistema DEVE reindexar só esse arquivo, na mesma reconciliação incremental da v0.1 (IDX-02). <!-- SRC-02 -->
3. QUANDO o usuário busca ENTÃO o sistema DEVE ignorar maiúsculas e acentos ("acao" acha "ação"), casar o último termo por prefixo e exigir todos os termos (E). <!-- SRC-03 -->
4. QUANDO há resultados ENTÃO o sistema DEVE agrupá-los por spec, mostrar projeto, chave, arquivo, seção e um trecho com os termos destacados, ordenados por relevância; cópias idênticas da mesma spec DEVEM aparecer uma vez só. <!-- SRC-04 -->
5. QUANDO o usuário filtra por projeto ou por arquivo (spec, design, tasks…) ou restringe à spec aberta ENTÃO o sistema DEVE aplicar os filtros na consulta. <!-- SRC-05 -->
6. QUANDO o usuário abre um resultado ENTÃO o sistema DEVE abrir a spec na instância do resultado, na aba do arquivo, rolada até a seção, com as ocorrências destacadas no texto renderizado. <!-- SRC-06 -->
7. QUANDO a consulta contém caracteres de sintaxe do FTS (`"`, `*`, `-`, `:`, `(`, `AND`, `NEAR`) ou está vazia ENTÃO o sistema NÃO DEVE dar erro: os termos são tratados como texto. <!-- SRC-07 -->
8. QUANDO há cerca de 300 specs indexadas ENTÃO uma consulta DEVE responder em p95 ≤ 100 ms, e o índice DEVE caber no orçamento definido no spike T10. <!-- SRC-08 -->

### P1: Command palette ⭐ MVP

**User Story**: Como dev, quero fazer tudo pelo teclado: abrir uma spec, trocar de aba ou de cópia, buscar e mudar o tema.

**Critérios de aceite**:

1. QUANDO o usuário pressiona Ctrl+K (em qualquer tela, inclusive dentro do editor) ENTÃO o sistema DEVE abrir a palette com foco no campo de texto. <!-- CMD-01 -->
2. QUANDO o usuário digita ENTÃO a palette DEVE listar, com casamento aproximado (subsequência) por chave, pasta e título, as specs e os comandos: trocar de aba e de instância da spec aberta, alternar Ler/Editar/Dividir, salvar, buscar no conteúdo, alternar tema e abrir preferências. <!-- CMD-02 -->
3. QUANDO o usuário escolhe "Buscar no conteúdo" ou digita o prefixo de busca ENTÃO a palette DEVE mostrar os resultados da busca full-text (SRC) no mesmo painel. <!-- CMD-03 -->
4. QUANDO a palette está aberta ENTÃO setas, Enter e Esc DEVEM funcionar, os itens recentes DEVEM aparecer primeiro com o campo vazio, e o painel DEVE ser acessível (diálogo com campo e lista anunciados pelo leitor de tela). <!-- CMD-04 -->
5. QUANDO a ação escolhida navega para outro arquivo, spec ou instância e há alterações não salvas ENTÃO a guarda de SAV-06 DEVE ser aplicada. <!-- CMD-05 -->

### P1: Checkbox do `tasks.md` (opt-in) ⭐ MVP

**User Story**: Como dev de um projeto que optou por isso, quero marcar e desmarcar tarefas pelo app, sem mexer em mais nada do arquivo.

**Critérios de aceite**:

1. QUANDO um projeto não tem a opção ligada (padrão) ENTÃO a aba tasks DEVE continuar somente leitura, como em TSK-05 da v0.1; QUANDO o usuário liga a opção nas Preferências (por projeto) ENTÃO ela DEVE ser gravada no `config.toml` e valer sem reiniciar. <!-- CHK-01 -->
2. QUANDO a opção está ligada e o usuário clica no checkbox de uma tarefa ENTÃO o sistema DEVE alternar só os caracteres do checkbox dessa linha (`[ ]` ↔ `[x]`); o resto do arquivo DEVE continuar idêntico byte a byte, inclusive fins de linha, BOM e newline final. <!-- CHK-02 -->
3. QUANDO o checkbox é alternado ENTÃO a escrita DEVE seguir SAV-01, SAV-02 e SAV-04; QUANDO o `tasks.md` mudou desde que a aba foi carregada, ou a linha não é mais a da tarefa com aquele id e aquele estado ENTÃO o sistema NÃO DEVE escrever: DEVE recarregar a aba e avisar. <!-- CHK-03 -->
4. QUANDO o `tasks.md` está aberto no editor com alterações não salvas ENTÃO os checkboxes DEVEM ficar desabilitados, com o motivo. <!-- CHK-04 -->
5. QUANDO o checkbox é alternado ENTÃO o progresso da fase, da aba, da lista e do cabeçalho DEVE se atualizar em até 1 s, sem aviso de conflito (CNF-06). <!-- CHK-05 -->

### P2: Progresso x/y

**User Story**: Como dev, quero confiar no progresso que o app mostra enquanto edito e marco tarefas.

A v0.1 já entrega o progresso x/y: barra e contagem por spec na sidebar (T15), contagem na aba (`tasks 7/8`, T17), faixa x/y com fases e cartões por fase na aba tasks (T18) e chips do cabeçalho do `summary.md` (T17). Os critérios abaixo cobrem só o que é novo.

**Critérios de aceite**:

1. QUANDO o próprio app altera um `tasks.md` (save no editor ou checkbox) ENTÃO todos os pontos que mostram progresso DEVEM se atualizar a partir do conteúdo salvo, sem depender do evento do watcher (que pode demorar o intervalo de polling em WSL). <!-- PRG-01 -->
2. **(depende de P5)** QUANDO a tabela de progresso do `summary.md` diverge das contagens do `tasks.md` da mesma instância ENTÃO a aba tasks e a faixa de metadados DEVEM mostrar um aviso com os dois valores. <!-- PRG-02 -->

### P1: Edição com várias instâncias ⭐ MVP

**User Story**: Como dev com a mesma spec em várias worktrees, quero saber exatamente em qual cópia estou escrevendo.

**Critérios de aceite**:

1. QUANDO o usuário salva ENTÃO o sistema DEVE escrever só na instância exibida; nos modos Editar e Dividir, o cabeçalho DEVE mostrar "Edições vão para `<branch>` · `<worktree>`". <!-- INS-01 -->
2. QUANDO a spec tem cópias divergentes (badge "N versões") e o usuário entra em edição ENTÃO o sistema DEVE mostrar um aviso visível de que existem outras versões e que a edição não se propaga. <!-- INS-02 -->
3. QUANDO o save torna divergentes cópias que eram idênticas ENTÃO o badge e o seletor DEVEM refletir isso (DUP-03) em até 2 s. <!-- INS-03 -->
4. QUANDO a instância editada é removida (WAT-04) e há alterações não salvas ENTÃO o sistema DEVE manter o buffer, NÃO DEVE trocar de cópia em silêncio, e DEVE oferecer "Salvar como cópia em outra instância", que abre o diálogo de conflito contra o conteúdo daquela instância. <!-- INS-04 -->

### P1: Dívidas da v0.1

**User Story**: Como usuário da v0.1, quero que as pendências conhecidas sejam fechadas antes de a v0.2 crescer em cima delas.

**Critérios de aceite**:

1. QUANDO o idioma configurado (ou do sistema, A9) é inglês ENTÃO toda a interface DEVE aparecer em inglês, com o catálogo `en` cobrindo todas as chaves do `pt-BR` (verificado em teste). <!-- DEB-01 -->
2. QUANDO o backend devolve um erro ENTÃO a UI DEVE mostrar a mensagem no idioma da interface, a partir de um código de erro, e não do texto em inglês do backend (limitação registrada na T23 da v0.1). <!-- DEB-02 -->
3. QUANDO um projeto usa o preset `generic` e a estrutura do `exban` é reconhecida (PST-01) ENTÃO a UI DEVE mostrar a sugestão, com uma ação explícita "Usar exban" que grava o preset no `config.toml`. <!-- DEB-03 -->
4. QUANDO a janela é estreita ENTÃO os chips de requisito das linhas de tarefa NÃO DEVEM passar da borda da linha: cedem espaço e mostram "+N" com a lista completa no painel da tarefa. <!-- DEB-04 -->
5. QUANDO `QUIRE_CONFIG` ou `QUIRE_INDEX` é um caminho relativo ENTÃO o sistema DEVE recusá-lo com uma mensagem clara e NÃO DEVE criar nem gravar arquivo em lugar nenhum por causa dele. <!-- DEB-05 -->
6. **(P2)** QUANDO o CLI roda no Windows a partir de um terminal ENTÃO `--help`, `--version` e os erros DEVEM aparecer nesse terminal (P2 da spec da v0.1). <!-- DEB-06 -->
7. QUANDO uma spec de 40 KB está aberta (modo Ler e modo Dividir) ENTÃO o consumo de memória DEVE ser medido no build de produção e DEVE ficar dentro do orçamento definido na T6. <!-- DEB-07 -->
8. QUANDO existe versão estável de `tauri-specta`/`specta` ENTÃO o projeto DEVE usá-la; senão, a verificação e a data DEVEM ficar registradas no Journal. <!-- DEB-08 -->
9. QUANDO o autor roda as rodadas pendentes no WSL (spikes T2 e T3 da v0.1) ENTÃO os resultados DEVEM ser registrados e a regra do TD-05 confirmada ou ajustada; a pasta `apps/desktop/src/spike/` DEVE ser removida depois disso. <!-- DEB-09 -->

### P2: Projeto open source

**User Story**: Como mantenedor, quero que a documentação e o CI acompanhem a nova capacidade de escrita.

**Critérios de aceite**:

1. O README DEVE documentar edição, conflito, busca, palette e checkbox opt-in, com as garantias e os limites de escrita (inclusive os de A6 e P6). <!-- OSS-05 -->
2. QUANDO há push ou PR ENTÃO o CI DEVE rodar os testes de escrita segura em Linux e Windows, inclusive os de fim de linha, BOM, permissões e arquivo em uso. <!-- OSS-06 -->

## Casos de borda

- **Arquivo sem newline final, com CRLF, com BOM, ou as três coisas juntas:** salvar sem alterar nada além do texto editado (SAV-03). Testes com todas as combinações.
- **Fins de linha mistos:** aviso ao editar e confirmação no primeiro save; os fins de linha são normalizados para o predominante (A4). Em caso de empate, vence o do primeiro fim de linha do arquivo.
- **Link simbólico para o arquivo ou para a pasta da spec:** escreve no destino e mantém o link (SAV-05). Link quebrado: edição desabilitada com motivo.
- **Arquivo somente leitura ou de outro dono:** não salva; oferece cópia (SAV-04).
- **Pasta sem permissão de escrita** (não dá para criar o temporário): falha como SAV-02. Nunca tenta escrever direto no original.
- **Disco cheio no meio da escrita:** o original fica intacto e o temporário é removido (SAV-02).
- **Crash entre o temporário e o rename:** sobra um temporário oculto. O original está intacto, e o temporário não aparece em "outros" e não é apagado sozinho (SAV-10). Um `git add -A` do agente pode pegar o temporário; o aviso da SAV-10 diz isso.
- **Agente escreve no mesmo arquivo durante o save:** detectado pelas duas verificações (CNF-03), salvo a janela residual (A6).
- **Agente escreve no `tasks.md` entre o carregamento da aba e o clique no checkbox:** recusa e recarrega (CHK-03).
- **Checkbox numa linha dentro de bloco de código** ou numa linha que se parece com tarefa mas não foi reconhecida pelo parser: não é clicável.
- **Tarefa marcada com `[X]` maiúsculo:** desmarcar grava `[ ]`; marcar sempre grava `[x]` minúsculo.
- **Save sem alterações:** não escreve e não mexe no `mtime` (EDT-05).
- **Ctrl+S no modo Ler:** não faz nada.
- **Arquivo apagado durante a edição:** CNF-07. **Instância removida durante a edição:** INS-04.
- **Mesmo arquivo aberto em outro editor que guarda o próprio buffer:** o Quire não sabe disso; ver risco "edição concorrente".
- **Arquivo grande (> limite) ou não UTF-8:** edição desabilitada (EDT-08); a leitura continua como na v0.1.
- **Caminho com acentos, espaços, em `\\wsl$` ou em unidade do Windows:** escrita testada nos sistemas do spike T11; `\\wsl$` depende de P6.
- **Windows: arquivo aberto por outro processo sem compartilhamento de exclusão:** nova tentativa por até 1 s, depois erro (SAV-08).
- **Consulta com operadores do FTS, aspas desbalanceadas, só espaços ou um termo muito comum:** sem erro (SRC-07); resultados limitados (os primeiros 200) com "refine a busca".
- **Spec com cópias idênticas em 3 worktrees:** um resultado de busca só (SRC-04); o save numa delas a torna divergente (INS-03).
- **Ctrl+K dentro do editor:** abre a palette (CMD-01); o atalho tem prioridade sobre o keymap do editor.
- **Navegação pelo CLI (`quire open …`) com buffer sujo:** passa pela guarda (SAV-06); cancelar mantém a tela atual e descarta a navegação.
- **Índice (`QUIRE_INDEX`) apontado para dentro de um `.specs` observado:** recusado na inicialização, para que a escrita do índice não gere eventos no watcher.

## Rastreabilidade de requisitos

| Requisito | Tarefas |
| --- | --- |
| DEB-01, DEB-02 | T1 |
| DEB-03 | T2 |
| DEB-04 | T3 |
| DEB-05 | T4 |
| DEB-06 | T5 |
| DEB-07 | T6, T26 |
| DEB-08 | T7 |
| DEB-09 | T8 |
| EDT-01, EDT-02, EDT-05 a EDT-08 | T9, T16 |
| EDT-03, EDT-04 | T17 |
| EDT-09 | T9, T16 |
| SAV-01 a SAV-05, SAV-08 | T11, T12 |
| SAV-06 | T18 |
| SAV-07 | T12, T15, T18 |
| SAV-09 | T12, T15 |
| SAV-10 | T12, T15, T19 |
| CNF-01, CNF-03 | T13 |
| CNF-02, CNF-05, CNF-07 a CNF-09 | T19 |
| CNF-04, CNF-06 | T13, T19 |
| SRC-01 a SRC-03, SRC-07, SRC-08 | T10, T21 |
| SRC-04 a SRC-06 | T22 |
| CMD-01 a CMD-05 | T23 |
| CHK-01 a CHK-05 | T14, T24 |
| PRG-01 | T25 |
| PRG-02 | T25 (se P5 for aprovada) |
| INS-01 a INS-04 | T20 |
| OSS-05 | T26 |
| OSS-06 | T12, T26 |

## Critérios de sucesso

- Uma spec inteira é revisada e corrigida no Quire, com preview, sem abrir outro editor.
- Em nenhum teste nem no roteiro E2E um save do Quire apaga uma alteração feita por fora sem o usuário ter escolhido "Sobrescrever".
- `git diff` depois de um save mostra só as linhas que o usuário mudou: nenhuma mudança de fim de linha, BOM ou newline final.
- Marcar uma tarefa pelo app (num projeto com a opção ligada) produz um diff de exatamente um caractere.
- Uma busca em ~300 specs responde em menos de 100 ms e leva ao trecho certo.
- Tudo o que a v0.1 fazia continua funcionando, com o CI verde em Linux e Windows.
