# QUIRE v0.2 — Design

**Idioma:** pt-BR
**Spec:** `spec.md`
**Status:** rascunho com as decisões do autor de 2026-11-16 aplicadas, aguardando aprovação

## Decisões do autor e onde entram no design

As perguntas P1 a P10 foram decididas em 2026-11-16 (spec, "Decisões do autor"). Mapeamento:

| Decisão | Onde entra |
| --- | --- |
| P1: criar arquivo ou spec fica para a v0.3 | Nenhum comando cria arquivo, exceto `save_copy` (CNF-08) e o `save_file` com `allow_recreate` (CNF-07), que partem de um arquivo aberto |
| P2, P3: sem autosave e sem rascunho | TD-24 |
| P4: checkbox muda só o caractere | TD-18 (atualizado) |
| P5: alerta summary × tasks para a v0.3 | Removido do design; `parse::summary_progress` continua sem uso |
| P6: escrita em `\\wsl$`/DrvFs/9p condicionada ao spike | TD-22, `quire-core::fs_kind` |
| P7: busca só em markdown | TD-15 (atualizado) |
| P8: idioma do sistema, fallback em inglês, troca nas Preferências | TD-19 (atualizado), TD-23, "Catálogo e idioma" |
| P9: binário auxiliar de console no Windows | TD-20 (decidido), "CLI no Windows" |
| P10: v0.1 mergeada antes; tag não bloqueia | Base da v0.2: `main` em `f256af3` (PR #1) |

## Ponto de partida (o que a v0.1 entrega e o que muda)

A arquitetura da v0.1 continua (`design.md` da v0.1): o núcleo em Rust (`quire-core`) faz scan, git, parsers, índice e watcher; o shell Tauri expõe comandos tipados; o React só renderiza. A v0.2 acrescenta a **primeira escrita do app em arquivos de spec**, e por isso o caminho de escrita fica inteiro no núcleo, testável sem UI e sem Tauri, como o resto.

Estado herdado que importa para a v0.2 (verificado no código em 2026-11-16, ver "Confirmado no código"):

- O watcher já decide os eventos pelo **estado em disco**, não pelo tipo de evento (correção pós-v0.1, fbbd790): compara tamanho e `mtime` dos arquivos de cada pasta de spec e ignora arquivos ocultos, de backup e temporários (`.x`, `x~`, `*.tmp`, `*.swp`). O temporário da escrita atômica (oculto) é invisível para ele.
- `read_file` devolve o texto UTF-8 como está no disco (BOM e CRLF inclusos), sem impressão digital.
- A única escrita do app hoje é o `config.toml` (`config::save_settings`, com `toml_edit` e escrita por temporário + rename, mas sem fsync nem verificação de conflito).

## Abordagens

### Escrita

- **A (escolhida): pipeline de escrita no núcleo (`quire-core::write`)** com temporário na mesma pasta, fsync, rename e fsync da pasta, mais detecção de conflito por hash do conteúdo bruto. Prós: um único lugar para as garantias de SAV e CNF, testável com tempdir em Linux e Windows (CI), reaproveitado pelo editor, pelo checkbox e pela cópia de conflito. Contras: o núcleo precisa conhecer o formato de texto (EOL, BOM) para codificar o que o editor devolve.
- **B: escrita pelo frontend** (`@tauri-apps/plugin-fs`): menos código em Rust, mas espalha as garantias pela UI, não dá fsync da pasta nem controle de permissões e abre o sistema de arquivos ao webview.
- **C: crate pronta de escrita atômica** (`atomic-write-file`, `tempfile::persist`): resolvem o rename, mas não a preservação de permissões, links simbólicos, a segunda verificação de conflito nem o retry de arquivo em uso no Windows. O pipeline próprio pode usar `tempfile` por baixo só para criar o temporário com nome único.

### Editor

- **A (escolhida): CodeMirror 6** carregado por import dinâmico (TD-11). Leve, modular, acessível, com bom suporte a composição e IME (a confirmar no spike T9 no WebKitGTK).
- **B: Monaco:** pesado (vários MB), pensado para Chromium; sem vantagem para markdown.
- **C: `textarea` com preview:** sem realce, busca nem desfazer confiável; não atende EDT-07.

### Busca

- **A (escolhida): FTS5 no mesmo SQLite do índice** (TD-15). Já compilado no `libsqlite3-sys` embutido; continua cache descartável; reindexa junto com a reconciliação incremental.
- **B: `tantivy`:** mais recursos, mas é um segundo índice para manter em sincronia, com binário maior.
- **C: busca no frontend** (minisearch e afins): exige carregar todo o texto no webview, o que contradiz a meta de memória.

## Architecture Overview

```mermaid
graph TD
    subgraph UI["apps/desktop (React + TS)"]
      ED[editor: CodeMirror, modos, preview]
      GU[guarda de alterações não salvas]
      CF[diálogo de conflito e diff]
      SR[busca]
      PA[command palette]
      TV[tasks view + checkbox opt-in]
    end
    subgraph Shell["src-tauri"]
      CMD[comandos: open_for_edit, save_file, save_copy, toggle_task, search]
      LV[live: fila de scan e watcher]
    end
    subgraph Core["crates/quire-core"]
      WR[write: atômico, preserva formato]
      FK[fs_kind: escrita por classe de FS]
      FP[fingerprint + conflito]
      TT[tasks::toggle]
      IX[index + FTS5]
      WT[watch por estado em disco]
    end
    ED --> CMD
    TV --> CMD
    SR --> CMD
    PA --> SR
    CMD --> WR
    CMD --> TT
    TT --> WR
    WR --> FP
    WR --> FK
    CMD --> IX
    WT --> LV
    LV --> IX
    WR --> FS[(Arquivos .specs)]
    WT --> FS
```

## Estrutura do repositório (o que é novo)

```
crates/quire-core/src/
├─ write.rs        # escrita atômica, formato de texto, permissões, links, retry no Windows
├─ fs_kind.rs      # classe do sistema de arquivos de um caminho e política de escrita (TD-22)
├─ fingerprint.rs  # impressão digital (tamanho, mtime, xxh3-128) e verificação de conflito
├─ tasks_edit.rs   # alternância de checkbox por linha, sobre bytes
└─ search.rs       # esquema FTS5, indexação por seção, consulta segura
crates/quire-cli/        # binário auxiliar de console do Windows (TD-20); sem Tauri, sem unsafe
apps/desktop/src-tauri/
├─ binaries/             # quire-cli-<target>.exe gerado antes do bundle (externalBin)
└─ wix/quire-cli.wxs     # fragmento WiX: bin\quire.cmd e PATH
apps/desktop/src/
├─ features/editor/      # CodeMirror (lazy), modos, preview, rolagem sincronizada
├─ features/conflict/    # diálogo, diff (@codemirror/merge, lazy), aviso "mudou no disco"
├─ features/unsaved/     # guarda de navegação e de fechamento
├─ features/search/      # painel de resultados, filtros
├─ features/palette/     # Ctrl+K, casamento aproximado, comandos
├─ lib/i18n/en.ts        # catálogo em inglês (DEB-01)
└─ lib/i18n/locale.ts    # resolução do idioma: config → sistema → inglês (DEB-10, TD-23)
```

## Components

### `quire-core::fingerprint`

`Fingerprint { len: u64, mtime_ns: Option<i128>, hash: u128 }` calculada sobre os **bytes brutos** do arquivo (xxh3-128, a mesma família usada no dedup da v0.1). Igualdade é por `hash` e `len`. O `mtime` serve só de atalho para pular a leitura quando tamanho e `mtime` são iguais aos de uma impressão recente da mesma sessão (A3). `fn of(path) -> Result<Option<Fingerprint>>`, em que `None` significa que o arquivo não existe.

### `quire-core::write`

```
TextFormat { bom: bool, eol: Eol (Lf | Crlf), mixed: bool, final_newline: bool }
fn decode(bytes) -> Result<(String, TextFormat), NotEditable>    // tira o BOM, CRLF → LF, recusa não UTF-8
fn encode(text, &TextFormat) -> Vec<u8>                          // LF → eol, recoloca BOM
fn save(target: &Path, bytes: &[u8], base: Option<&Fingerprint>, opts) -> Result<Fingerprint, WriteError>
```

Passos de `save` (SAV-01 a SAV-05, SAV-08, CNF-03):

1. Resolve o alvo. Se `symlink_metadata` diz que é link, segue até o destino (`canonicalize`); link quebrado → `WriteError::BrokenLink`. A partir daqui tudo acontece na pasta do **destino**.
2. Consulta `fs_kind::policy(destino)`: classe bloqueada → `WriteError::UnreliableFs(classe)` (SAV-11, TD-22). Lê os metadados do destino: somente leitura, ou dono diferente do usuário atual no Unix (`uid`) → `WriteError::ReadOnly` / `ForeignOwner`.
3. **Primeira verificação:** `fingerprint::of(destino)` ≠ `base` → `WriteError::Conflict { disk: Option<Fingerprint> }`. `base = None` significa "o arquivo não deve existir" (cópia de conflito, recriação confirmada); se existir, é conflito.
4. Cria o temporário `.<nome>.quire-<pid>-<contador>.tmp` na mesma pasta com `create_new`. O nome oculto o mantém fora do watcher e da aba "outros" (SAV-10).
5. Escreve os bytes, aplica as permissões do original (modo no Unix, somente leitura no Windows) e faz `sync_all`.
6. **Segunda verificação:** recalcula a impressão do destino. Se mudou, apaga o temporário e devolve `Conflict`.
7. Faz o `rename(tmp, destino)`. No Windows, um erro de compartilhamento (`ERROR_SHARING_VIOLATION`, `ERROR_ACCESS_DENIED` com o arquivo aberto) tenta de novo com espera crescente até 1 s (SAV-08).
8. No Unix, abre a pasta e faz `sync_all` (durabilidade do rename). No Windows não há equivalente sem FFI; o NTFS registra o rename no journal.
9. Devolve a impressão digital nova, calculada dos bytes escritos (sem reler o disco), para a UI usar como nova base.

Qualquer erro entre os passos 4 e 7 remove o temporário criado nessa ação, que é parte da mesma ação explícita (SAV-02). O pipeline nunca escreve no original diretamente, nem como fallback.

**Janela residual (A6, CNF-03):** entre a segunda verificação (passo 6) e o rename (passo 7) há microssegundos em que um agente pode escrever, e essa escrita seria substituída. Sem uma trava respeitada pelos dois lados, não dá para fechar essa janela. O design a reduz ao mínimo, pois os dois passos são consecutivos e sem I/O de rede, e a documenta no README (OSS-05).

### `quire-core::fs_kind` (P6)

```
enum FsClass { Local, WslUnc, DrvFs, NineP }      // Local = qualquer outro
enum WritePolicy { Allowed, Blocked(FsClass) }
fn classify(path) -> FsClass
fn policy(path) -> WritePolicy                    // consulta VERIFIED_ATOMIC
const VERIFIED_ATOMIC: &[FsClass] = &[];          // preenchida com o dado do spike T11
```

- **Linux, inclusive dentro do WSL:** lê `/proc/self/mountinfo` (texto, sem `unsafe` nem dependência), escolhe a montagem com o ponto de montagem mais longo que é prefixo do caminho já canonicalizado (com os escapes `\040` e afins decodificados) e olha o tipo e as opções: `drvfs`, ou `9p` com `aname=drvfs`, → `DrvFs`; outro `9p`/`v9fs` → `NineP`. Se a leitura falhar, o caminho é tratado como `Local` e o motivo vai para o log. Num Linux comum, sem 9p, o resultado é sempre `Local`.
- **Windows:** pelo prefixo do caminho normalizado (sem `\\?\`, `\\?\UNC\` → `\\`, separadores e maiúsculas indiferentes): `\\wsl$\` e `\\wsl.localhost\` → `WslUnc`. Unidade mapeada para o WSL (`Z:` → `\\wsl$`) não é reconhecida, a mesma limitação da v0.1 (T21) registrada como risco.
- **Política:** `Local` sempre `Allowed`; as outras classes só são `Allowed` se estiverem em `VERIFIED_ATOMIC`. A lista começa vazia e só muda num commit que cite a entrada do Journal com a tabela do spike T11. Sem dado, a classe fica bloqueada (P6).
- **Quem consulta:** `write::save` (passo 2), `open_for_edit` (motivo do cadeado, EDT-08) e `toggle_task`. A UI nunca decide sozinha.

### `quire-core::tasks_edit`

`fn toggle(bytes, line: usize, task_id, expected_done: bool) -> Result<Vec<u8>, ToggleError>`:

- Localiza a linha `line` nos bytes, contando LF; o CR, se houver, fica onde está.
- Exige que a linha case com a gramática de tarefa do `parse::tasks` (o mesmo regex), com o mesmo id e o estado esperado; senão devolve `ToggleError::Moved`.
- Troca só o byte entre os colchetes: `' '` → `'x'`, ou `'x'`/`'X'` → `' '`. Todo o resto do vetor fica idêntico (CHK-02); um teste compara byte a byte com e sem CRLF, BOM e newline final. A função não recebe nem escreve hash, data ou anotação (P4): a assinatura não tem como acrescentar texto.
- A gravação usa `write::save` com `base` = impressão digital do `tasks.md` quando a aba foi carregada (CHK-03).

Linhas dentro de blocos de código nunca chegam aqui, porque o parser não as reconhece como tarefa e a UI não as torna clicáveis.

### `quire-core::search` (FTS5)

Esquema no mesmo banco do índice, com `user_version` 2. Um índice com versão 1 é apagado e reconstruído, como a v0.1 já faz com versões desconhecidas (IDX-01):

```sql
-- conteúdo único por hash: cópias idênticas em várias worktrees são indexadas uma vez (SRC-04)
CREATE TABLE fts_docs (id INTEGER PRIMARY KEY, content_hash TEXT UNIQUE NOT NULL);
CREATE TABLE file_docs (instance_id INTEGER NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
                        name TEXT NOT NULL, doc_id INTEGER NOT NULL REFERENCES fts_docs(id),
                        PRIMARY KEY (instance_id, name));
-- uma linha por seção (título + corpo até o próximo título), para levar o resultado à seção (SRC-06)
CREATE VIRTUAL TABLE fts USING fts5(heading, body, doc_id UNINDEXED, line UNINDEXED,
                                    tokenize = 'unicode61 remove_diacritics 2');
```

- **Só markdown (P7):** entram só os arquivos `.md` listados na instância. `gates.jsonl`, `lessons.jsonl` e os demais nunca chegam ao `fts`, e o filtro fica na função de indexação, com teste.
- **Indexação:** na reconciliação, um `.md` novo ou alterado (IDX-02) tem o hash calculado. Se o `content_hash` já existe em `fts_docs`, só liga `file_docs`; senão, divide o texto em seções (títulos fora de blocos de código, reaproveitando o parser de outline) e insere as linhas no `fts`. Documentos sem nenhum `file_docs` são removidos ao fim da reconciliação.
- **Consulta (TD-16):** o texto do usuário vira termos separados por espaço. Cada termo é escapado como string do FTS5 (`"…"`, com aspas internas dobradas) e o último ganha `*` (prefixo); os termos são unidos por espaço, que é E implícito. Nenhuma sintaxe do usuário chega crua ao `MATCH` (SRC-07). Filtros (projeto, nome do arquivo, spec atual) são `JOIN`s em `file_docs` → `instances` → `specs` → `projects`. A ordenação é por `bm25(fts, 5.0, 1.0)`, com peso maior para o título da seção. O trecho vem de `snippet(fts, 1, char(2), char(3), '…', 12)`; os marcadores `\u0002`/`\u0003` viram `<mark>` no React, nunca HTML vindo do banco. O limite é de 200 resultados.
- **Agrupamento:** por spec. A instância mostrada é a padrão quando o trecho está nela; senão, a primeira que contém o documento.

### Frontend

- **`features/editor`:** `EditorPane` monta o CodeMirror por import dinâmico (`@codemirror/view`, `state`, `commands`, `search`, `lang-markdown`, `language`), com tema derivado dos tokens CSS. O buffer guarda `{ text, base: Fingerprint, format: TextFormat, dirty }`. O preview reaproveita o pipeline `lib/markdown` da v0.1, com debounce de 200 ms. Para evitar reprocessar diagramas a cada tecla, os blocos Mermaid e Shiki ficam em cache pela origem.
- **Rolagem sincronizada (EDT-04):** o pipeline anota cada bloco de nível superior com `data-line` (a posição do `remark`). Os dois lados mapeiam a linha visível ↔ o bloco mais próximo, com interpolação dentro do bloco; um lado só segue o outro enquanto o usuário rola o primeiro, para não haver ecos.
- **`features/unsaved`:** um store com o arquivo sujo e um `guard(next: () => void)` usado por toda navegação: abas, sidebar, seletor de instância, palette, `cli://open` e preferências. O fechamento da janela usa o `onCloseRequested` do Tauri (`preventDefault` e o diálogo); depois de Salvar ou Descartar, a janela fecha. O buffer e o histórico do CodeMirror vivem só no `EditorState` em memória (TD-24): nenhum código grava texto de buffer em `localStorage`, `sessionStorage` ou IndexedDB. Uma recarga do disco (CNF-04) entra no histórico como uma transação, então Ctrl+Z depois dela devolve o texto anterior como alteração não salva, sem escrever nada.
- **`features/conflict`:** o diálogo com as quatro ações e o diff por linha com `@codemirror/merge` (unified, carregado sob demanda e reaproveitando o CodeMirror já baixado). O aviso fixo de CNF-05 fica acima do editor.
- **Supressão da própria escrita (TD-14):** depois de `save_file`, a UI troca a `base` pela impressão digital devolvida. Ao receber `spec://changed`, pede `file_fingerprint` e compara: se for igual à `base`, ignora (CNF-06); se diferir e o buffer estiver limpo, recarrega (CNF-04); se diferir e o buffer estiver sujo, mostra o aviso (CNF-05). Não existe janela de tempo do tipo "ignorar eventos por N ms", que esconderia a escrita de um agente feita logo depois do save.
- **`features/search` e `features/palette`:** a palette é um `dialog` com `input` (`role="combobox"`) e `listbox`, e casamento por subsequência feito em casa (TD-17). Os resultados full-text vêm de `search` (backend), com debounce de 120 ms.
- **Checkbox (CHK):** na aba tasks, quando o projeto tem `tasks_checkbox = true`, o ícone de estado vira um `input type="checkbox"` real com rótulo. O clique chama `toggle_task` com a impressão do `tasks.md` carregado. Num caminho bloqueado pela SAV-11, o checkbox fica desabilitado com o motivo devolvido pelo backend.

### Catálogo e idioma (P8)

- **Catálogos:** `pt-BR.ts` continua sendo a fonte das chaves; `en.ts` é tipado como `typeof ptBR` (TD-19), então faltar ou sobrar chave é erro de compilação, e um teste confere também os parâmetros `{nome}` de cada string. `catalogs` passa a ter `"pt-BR"` e `"en"`.
- **Resolução (TD-23):** `resolveLocale(config: string | null, system: readonly string[]): Locale`, pura e testada por tabela. Se `config` for `"pt-BR"` ou `"en"`, ela vale. Senão, a primeira entrada de `system` que comece com `pt` dá `"pt-BR"`, e a primeira que comece com `en` dá `"en"`. Sem nenhuma das duas, o resultado é `"en"`. O `system` vem de `navigator.languages`, que o WebKitGTK deriva de `LANGUAGE`/`LC_ALL`/`LC_MESSAGES`/`LANG` e o WebView2 do idioma de exibição do Windows (a confirmar na T27; ver "Não verificado").
- **Troca em tempo de execução:** um store Zustand com o `Locale` atual; `t()` lê do store, e os componentes que usam `t` re-renderizam com a troca. `Intl.PluralRules`, `Intl.DateTimeFormat` e `Intl.NumberFormat` são recriados para o novo idioma, e `document.documentElement.lang` acompanha.
- **Preferências:** um seletor com "Sistema", "Português (Brasil)" e "English". A escolha passa pelo `set_config` existente: um idioma grava `language`, e "Sistema" remove a chave, com `toml_edit` preservando os comentários. Um valor desconhecido no arquivo é tratado como ausente e mostrado como aviso, sem regravar (DEB-10).
- **Erros do backend (DEB-02):** `CmdError { code, detail }`. A UI traduz `code` pelo catálogo, e `detail`, o texto do backend em inglês, aparece só como detalhe recolhido.

### CLI no Windows (`quire-cli`, P9)

O `quire.exe` do Tauri é um binário GUI, e por isso o Windows não liga a saída dele ao terminal (P2 da v0.1). A solução segue o modelo do VS Code (`bin\code.cmd`):

- **`crates/quire-cli`:** binário de console, sem Tauri e sem `unsafe` (o `forbid` do workspace vale). Ele monta o parser com `clap` a partir da seção `plugins.cli` do `tauri.conf.json`, embutida em tempo de compilação (`include_str!` + `serde_json`), para que a gramática continue com uma fonte só (TD-10 da v0.1). Para `--help`, `--version` e erro de argumento, ele imprime no próprio console e sai com 0, 0 e 2, como o app no Linux. Para qualquer chamada válida, inicia o `quire.exe` da mesma pasta (`current_exe().parent()`) com os mesmos argumentos e o diretório atual, por `std::process::Command` com `DETACHED_PROCESS` (`CommandExt::creation_flags`, API segura), e sai com 0 sem esperar. O single-instance da v0.1 cuida do resto.
- **Empacotamento:** `bundle.externalBin` do Tauri instala o `quire-cli.exe` ao lado do `quire.exe` (o CI compila o auxiliar antes do `tauri build` e o copia para `src-tauri/binaries/quire-cli-<target>.exe`). A declaração fica em `tauri.bundle-windows.conf.json`, passada só ao build de empacotamento com `--config`; um `tauri.windows.conf.json` seria mesclado em todo build do Windows, inclusive `cargo check`, antes de o auxiliar existir (falha do CI de 2026-11-16). Um fragmento WiX (`bundle.windows.wix.fragmentPaths` e `componentRefs`) instala `bin\quire.cmd`, com `@"%~dp0..\quire-cli.exe" %*`, e acrescenta `[INSTALLDIR]bin` ao `PATH` do sistema. A pasta da instalação não entra no `PATH`, para que `quire` nunca resolva para o `quire.exe` GUI, que teria precedência sobre o `.cmd` pela ordem do `PATHEXT`.
- **Linux:** o auxiliar não é empacotado no `.deb` nem no AppImage, e nada muda no CLI de lá. O crate compila e é testado em todos os sistemas, porque a lógica é portátil.
- **Paridade:** um teste roda no Linux o `quire --help` do app real e o do auxiliar e compara as saídas, ignorando o nome do programa. Se o `tauri-plugin-cli` formatar a ajuda de um jeito que o `clap` puro não reproduz, a T5 para e registra no Journal antes de aceitar a diferença.

## Shell Tauri: comandos e eventos novos

| Comando | Entrada | Saída |
| --- | --- | --- |
| `open_for_edit` | `instance_id, name` | `EditableFile { text, format, fingerprint, editable: Result<(), Motivo> }` |
| `file_fingerprint` | `instance_id, name` | `Option<Fingerprint>` |
| `save_file` | `instance_id, name, text, format, base, confirm_mixed_eol, allow_recreate` | `Saved { fingerprint }` ou `Conflict { disk_text?, fingerprint? }`, `ReadOnly`, `InUse`, `UnreliableFs`… (códigos, DEB-02) |
| `save_copy` | `instance_id, name, text, format` | `Saved { name_da_copia, fingerprint }` |
| `toggle_task` | `instance_id, task_id, line, expected_done, base` | `Toggled { fingerprint, tasks }` ou `Conflict`/`Moved` |
| `search` | `query, filtros, limite` | `Vec<SearchHit>` agrupados |
| `orphan_temps` / `remove_orphan_temp` | `instance_id` / `instance_id, name` | lista / `()` (SAV-10, só por clique) |

Todos validam o nome como na `read_known` da v0.1 (um componente só, arquivo listado na instância) e, para escrita, a extensão `.md`, a exclusão dos JSONL (SAV-09) e a política do `fs_kind` (SAV-11); o motivo de `editable` inclui `UnreliableFs(classe)`. Nenhum comando cria arquivo novo além do `save_copy` e da recriação confirmada do `save_file` (P1). Depois de uma escrita bem-sucedida, o shell chama `live::request_scan` com a pasta da spec, para o índice, a lista e o FTS se atualizarem sem esperar o watcher (PRG-01, INS-03). O `toggle_task` já devolve o `TasksView` novo para a UI atualizar o progresso na hora.

Eventos: nenhum novo. `spec://changed` (v0.1) passa a ser tratado pela regra do TD-14.

## Data Models

### `config.toml`

```toml
language = "pt-BR"            # novo (DEB-10); "pt-BR" | "en"; ausente = idioma do sistema, fallback en (A9, P8)

[projects."acme_gateway"]
preset = "exban"
tasks_checkbox = true         # novo (CHK-01); padrão false
```

`ProjectConfig` usa `deny_unknown_fields`. Os campos novos entram no `struct`, no `save_settings` (que preserva comentários) e na tela de Preferências. Um `config.toml` da v0.2 lido por uma v0.1 dá erro de campo desconhecido, que a v0.1 já mostra sem sobrescrever o arquivo. `language` é lido como `Option<String>`, e não como enum, para que um valor desconhecido não invalide a config inteira (DEB-10). Quem decide se o valor é válido é o frontend, que mostra o aviso.

### Índice

`user_version` 2 com as tabelas de busca acima. As tabelas da v0.1 não mudam.

## Error Handling Strategy

| Cenário | Comportamento |
| --- | --- |
| Conteúdo no disco ≠ base no save | Não escreve; diálogo de conflito (CNF-02) |
| Mudança entre a 2ª verificação e o rename | Não detectável (A6); documentado |
| Arquivo somente leitura ou de outro dono | Não escreve; oferece cópia (SAV-04) |
| Caminho de classe sem rename atômico verificado (`\\wsl$`, DrvFs, 9p) | Edição, cópia e checkbox desabilitados com motivo; o núcleo recusa (SAV-11) |
| `mountinfo` ilegível | Classe `Local` e log; a escrita segue as demais garantias (TD-22) |
| `language` desconhecido no `config.toml` | Tratado como ausente; aviso nas Preferências; arquivo intocado (DEB-10) |
| Auxiliar do CLI sem `quire.exe` ao lado | Mensagem no console e código 1 (DEB-06) |
| Pasta sem permissão, disco cheio, I/O | Original intacto, temporário removido, buffer mantido, motivo exibido (SAV-02) |
| Arquivo em uso (Windows) | Nova tentativa até 1 s; depois erro "arquivo em uso" (SAV-08) |
| Link simbólico quebrado | Edição desabilitada (EDT-08) |
| Fins de linha mistos | Aviso e confirmação no primeiro save (SAV-03) |
| Não UTF-8, binário, acima do limite | Edição desabilitada (EDT-08) |
| Temporário órfão | Não listado; aviso com remoção por clique (SAV-10) |
| Linha da tarefa mudou | Não escreve; recarrega a aba (CHK-03) |
| Consulta inválida para o FTS | Impossível por construção (TD-16); erro do SQLite vira "nenhum resultado" e log |
| `QUIRE_CONFIG`/`QUIRE_INDEX` relativos, ou índice dentro de `.specs` | Recusa com mensagem; config fica não editável (DEB-05) |

## Tech Decisions

As decisões TD-01 a TD-10 da v0.1 continuam valendo. As novas:

- **TD-11 CodeMirror 6 por import dinâmico** como editor (ver Abordagens). O módulo do editor e o `@codemirror/merge` ficam fora do chunk inicial (EDT-02). O orçamento de bundle sai do spike T9.
- **TD-12 Pipeline de escrita no núcleo** (`write::save`): temporário oculto na mesma pasta, fsync do arquivo, duas verificações de conflito, rename, fsync da pasta no Unix, retry de compartilhamento no Windows. Nunca há escrita direta no original. Preserva BOM, EOL, newline final e permissões; segue links simbólicos até o destino.
- **TD-13 Conflito por hash do conteúdo bruto** (xxh3-128), e não por `mtime`. O `mtime` tem granularidade de até 2 s em FAT/exFAT e é preservado por algumas ferramentas, então não prova nada sozinho (A3).
- **TD-14 Supressão da própria escrita por comparação de impressão digital, não por tempo.** O watcher não muda: ele continua relatando a escrita do app (o índice precisa dela), e a UI descarta o evento quando o disco é igual à `base` nova. Isso também impede um ciclo de recarga (risco "loop de watcher"): nenhuma recarga escreve.
- **TD-15 FTS5 no índice SQLite existente**, com `unicode61 remove_diacritics 2`, uma linha por seção e documentos deduplicados por hash. O cache continua descartável (TD-06). Indexa só `.md` (P7, decidido em 2026-11-16): os JSONL ficam fora.
- **TD-16 Consulta construída, nunca repassada:** os termos do usuário são sempre escapados como strings do FTS5, com prefixo só no último.
- **TD-17 Casamento aproximado da palette feito em casa** (subsequência com bônus para início de palavra e para a chave da issue), com testes, em vez de uma dependência para algumas dezenas de linhas.
- **TD-18 Alternância do checkbox sobre bytes no núcleo**, com a gramática do `parse::tasks` e a escrita do TD-12. A UI nunca reescreve o `tasks.md` inteiro para marcar uma tarefa. Muda só o caractere entre os colchetes, sem anotação nem hash (P4, decidido em 2026-11-16): o hash do commit continua sendo escrito pelo agente.
- **TD-19 Catálogo `en` tipado pelo `pt-BR`** (`typeof ptBR` como contrato) e erros do backend como códigos (`CmdError.code`) traduzidos na UI (DEB-01, DEB-02). O idioma é trocado em tempo de execução, sem reiniciar. Com P8 (2026-11-16), o `pt-BR` continua sendo a fonte das chaves, mas deixa de ser o idioma padrão: o padrão vem do TD-23.
- **TD-20 Console do CLI no Windows por binário auxiliar (DEB-06; P9 decidida em 2026-11-16):** o `unsafe_code = forbid` do workspace impede chamar `AttachConsole` no nosso código. Entre (a) um binário de console mínimo e (b) uma dependência que encapsula `AttachConsole`, o autor escolheu (a), no estilo do `code.cmd` do VS Code: `quire-cli.exe` (crate `quire-cli`, sem Tauri) instalado como `externalBin` e `bin\quire.cmd` no `PATH` pelo fragmento WiX (ver "CLI no Windows"). A gramática vem do `plugins.cli` do `tauri.conf.json`, então não há segunda definição. Descartados: `quire.com` ao lado do `quire.exe` (o `quire` digitado dependeria da ordem do `PATHEXT`, e a pasta da instalação teria de entrar no `PATH`, expondo o binário GUI) e a opção (b).
- **TD-21 Variáveis de ambiente de caminho só absolutas:** `QUIRE_CONFIG` e `QUIRE_INDEX` relativos são recusados, porque o diretório de trabalho do processo varia (`tauri dev` roda em `src-tauri`, o que já gerou um `config.toml` perdido na v0.1). O índice dentro de uma pasta observada também é recusado.
- **TD-22 Escrita condicionada à classe do sistema de arquivos (SAV-11; P6 decidida em 2026-11-16):** `fs_kind` classifica o caminho (`Local`, `WslUnc`, `DrvFs`, `NineP`) sem `unsafe`. Fora de `Local`, só há escrita se a classe estiver em `VERIFIED_ATOMIC`, uma lista no código que só muda com o dado do spike T11 registrado no Journal. Sem dado, a classe fica bloqueada. A alternativa (b) da P6, escrita não atômica com confirmação, foi descartada pelo autor: ela transfere ao usuário o risco de perda de dados. A recusa fica no núcleo (`write::save`), e a UI só mostra o motivo.
- **TD-23 Idioma padrão do sistema com fallback em inglês (DEB-10; P8 decidida em 2026-11-16):** a ordem é `language` do `config.toml`, depois a primeira entrada de `navigator.languages` que seja `pt*` ou `en*`, depois inglês. A detecção fica no frontend, porque o webview já expõe as preferências do sistema e isso não exige dependência nova no Rust nem `unsafe`. O seletor das Preferências tem "Sistema", e escolher "Sistema" remove a chave do arquivo.
- **TD-24 Sem autosave e sem rascunho (P2 e P3, decididas em 2026-11-16):** não existe gravação por tempo, por perda de foco nem por troca de tela, e nenhuma opção para ligá-la. O texto do buffer não vai para `localStorage`, `sessionStorage`, IndexedDB, índice nem arquivo auxiliar. A proteção contra perda é a guarda de alterações não salvas (SAV-06), inclusive no fechamento da janela, e o histórico em memória do CodeMirror (EDT-07). O custo aceito é perder as alterações não salvas num crash. A regra do `AGENTS.md` não ganha exceção.

## Risks & Concerns

| Risco | Impacto | Mitigação | Verificação |
| --- | --- | --- | --- |
| **Perda de dados** (save trunca, corrompe ou sobrescreve) | Crítico | TD-12 (nunca escreve no original; temporário + fsync + rename); SAV-02; CNF | Testes de perda de dados na T12: falha injetada em cada etapa, conteúdo do original conferido; roteiro E2E na T26 |
| **Edição concorrente com agente** | Crítico | Duas verificações por hash (TD-13); aviso de mudança no disco com buffer sujo (CNF-05); diff antes de sobrescrever | Testes que escrevem por fora entre as etapas; janela residual documentada (A6) |
| **Loop de watcher** (o save do app dispara recarga, que dispara outro save) | Alto | Nenhum caminho de recarga escreve (SAV-07); supressão por impressão digital (TD-14); o temporário é oculto e ignorado pelo watcher | Teste do shell: save → nenhuma recarga nem aviso em 3 s, uma única reconciliação |
| **CRLF/BOM/newline final** | Alto (diff ruidoso no git, agentes confusos) | `TextFormat` detectado na leitura e reaplicado na escrita; mistos com confirmação | Matriz de testes com as combinações; `.gitattributes` da v0.1 força LF no repositório, então as fixtures de CRLF são geradas pelo script |
| **Links simbólicos** | Alto (trocar o link por arquivo quebra o repo) | Escrever no destino (passo 1 do `save`) | Testes com link para arquivo e para pasta (Unix; no Windows, só se o runner permitir criar links) |
| **Permissões** | Médio | Copiar o modo; recusar somente leitura e dono diferente | Testes Unix de modo 0644/0600/0755; somente leitura no Windows |
| **Arquivo grande** | Médio (editor lento, memória) | Limite de edição (A8) medido no spike T9; preview com debounce e cache de diagramas | Spike T9 e medição de memória (T6, T26) |
| **Escrita em `\\wsl$`, DrvFs, 9p** | Alto (rename pode não ser atômico, eventos não chegam) | TD-22: classe bloqueada até o spike T11 mostrar rename atômico; o conflito por hash não depende de eventos | Spike T11 (parte em CI no NTFS; WSL com o autor); testes do `fs_kind` com `mountinfo` sintético |
| **Classe de FS não reconhecida** (unidade mapeada para `\\wsl$`, montagem exótica) | Médio (escrita liberada onde deveria ser bloqueada) | Ficam as demais garantias (duas verificações, temporário + rename); a limitação é documentada no README | Registrado como limitação; reavaliar com o dado do spike |
| **Auxiliar do CLI divergir do app** | Baixo | Gramática única vinda do `tauri.conf.json`; teste de paridade da ajuda no Linux | T5 |
| **Instalação do MSI não testada por humano** (o autor não tem Windows) | Médio | O CI instala o MSI em modo silencioso e roda `quire --version` pelo `PATH` | T5; o que o CI não cobrir fica NÃO VERIFICADO no Journal |
| **Arquivo aberto por outro processo no Windows** | Médio | Nova tentativa por até 1 s (SAV-08) | Teste no CI do Windows segurando o arquivo aberto sem `FILE_SHARE_DELETE` |
| **Teclas mortas e IME no WebKitGTK/Wayland** | Alto para pt-BR | Spike T9 antes de qualquer feature; plano B é o modo Ler e o editor externo, o que adiaria EDT | Spike T9 com roteiro manual do autor |
| **Bundle e memória** | Médio | Editor e merge carregados sob demanda; orçamento medido (DEB-07) | T6, T9, T26 |
| **Crescimento do índice FTS** | Baixo | Deduplicação por hash e cache descartável | Spike T10 |

## Confirmado no código e não verificado

**Confirmado no código (2026-11-16, branch `docs/v0-2-spec`, rebaseada em `main` depois do merge da v0.1 em `f256af3`):**

- `libsqlite3-sys` 0.40.0 (via `rusqlite` 0.42 `bundled`) compila o SQLite com `-DSQLITE_ENABLE_FTS5`: o FTS5 está disponível sem dependência nova.
- `read_file` (`apps/desktop/src-tauri/src/commands.rs`) devolve `String::from_utf8` dos bytes, com BOM e CRLF preservados, sem impressão digital; `LARGE_FILE_BYTES` = 2 MB.
- `read_known` aceita só nomes de um componente que estejam na lista de arquivos da instância (base para a validação de escrita, SAV-09).
- A aba "outros" lista todos os arquivos da pasta, inclusive ocultos, pois o `scan::list_files` não filtra. O temporário órfão precisa ser filtrado explicitamente (SAV-10).
- O watcher ignora nomes ocultos e de backup (`watch::transient`), então o temporário oculto do TD-12 não gera evento por si; o rename para o nome final gera.
- `ProjectConfig` tem `deny_unknown_fields` e só `preset`, `alias` e `issue_url_base`; o `Config` não tem `language`.
- `suggested_preset` existe no índice e no IPC (`ProjectSummary.suggestedPreset`), mas nenhum componente da UI o usa (DEB-03).
- `parse::summary_progress` existe e só é usado em testes; continua assim até a v0.3 (P5).
- `lib/i18n/index.ts` tem só o catálogo `pt-BR` e cria `Intl.PluralRules("pt-BR")` fixo; `setLocale` existe, mas nenhum componente re-renderiza com a troca (TD-19, TD-23).
- A gramática do CLI fica em `plugins.cli` do `tauri.conf.json`, e o `cli.rs` lê `help`/`version` das `matches` do `tauri-plugin-cli` (base do TD-20).
- `tauri-specta` =2.0.0-rc.29 e `specta` =2.0.0-rc.28 continuam fixados (DEB-08).
- Os wireframes não cobrem o modo editar/dividir nem a palette (`docs/design/README.md`, "Fora do wireframe v1").

**Não verificado (e nenhum artefato pode afirmar o contrário até os spikes):**

- Comportamento do CodeMirror 6 no WebKitGTK sob Wayland com teclas mortas (´ ~ ^ \` ¨), composição e IME (ibus, fcitx5); idem no WebView2 do Windows.
- Latência de digitação e memória do CodeMirror com arquivos de 40 KB, 500 KB e 2 MB.
- Tamanho do índice FTS5 e latência de consulta com ~300 specs.
- Atomicidade do `rename` por cima de arquivo existente em DrvFs (`/mnt/c`), em 9p (WSL acessando o Windows e o Windows acessando o WSL via `\\wsl$`) e no NTFS com o arquivo aberto por outro processo.
- Se o `std::fs::rename` do Windows (`MoveFileExW` com `MOVEFILE_REPLACE_EXISTING`) preserva o atributo somente leitura e as ACLs do original. A hipótese é que não, e por isso o passo 5 reaplica o atributo; as ACLs ficam como risco.
- Preservação de `owner`/`group` no Unix quando o usuário é dono do arquivo, mas não da pasta.
- Como DrvFs e 9p aparecem no `/proc/self/mountinfo` das versões atuais do WSL2 (tipo `9p` com `aname=drvfs`, ou tipo `drvfs`). O `fs_kind` aceita as duas formas até o spike T11 mostrar a real.
- Se `navigator.languages` reflete o idioma do sistema no WebKitGTK (variáveis `LANG` e afins) e no WebView2 (idioma de exibição do Windows).
- Se a ajuda gerada pelo `clap` no `quire-cli` sai igual à do `tauri-plugin-cli`; se o fragmento WiX com `Environment` no `PATH` funciona com o `bundle.windows.wix` do Tauri 2.12; e se o `.cmd` repassa o código de saída no PowerShell.
