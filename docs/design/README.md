# Design

Direção visual: **neutro moderno, escuro primeiro**, layout de **três colunas**. Fontes: Inter (UI) e JetBrains Mono (código, IDs, hashes).

## Referência

Os wireframes são HTML estático (nomes sintéticos), abertos direto no navegador. Adicione `?light` na URL para ver o tema claro.

| Tela | Arquivo | O que mostra |
| --- | --- | --- |
| Leitura de uma spec | `wireframes/1-leitura.html` | Projetos e specs, abas dos arquivos, conteúdo, outline, hover de rastreabilidade de ID, `arquivo:linha` clicável |
| Tasks por fase | `wireframes/2-tasks.html` | Fases colapsáveis, diagrama de dependência em chips, detalhe da task, checkbox somente leitura |
| Gates e lessons | `wireframes/3-gates.html` | Tabela de gates com filtros, linha de falha expandida, lessons no painel lateral |
| Instâncias e diff | `wireframes/4-instancias.html` | Badge "N versões", popover de instâncias, diff lado a lado |

## Estrutura da janela

- **Barra superior (48 px):** logo, breadcrumb projeto / spec, busca (Ctrl K), badge da instância, preferências.
- **Coluna esquerda (272 px):** projetos (git) com specs, filtro, status e progresso, rodapé com o scan.
- **Centro:** abas dos arquivos que existem na spec (com contagens), faixa de metadados e ações, conteúdo.
- **Coluna direita (248–320 px, contextual):** outline na leitura, detalhe da task em tasks, lessons em gates, arquivos alterados no diff.
- **Barra de status (28 px):** watcher, contagem de specs, estado do índice.

## Tokens

`tokens.css` define as variáveis do tema escuro (`.qd`) e do claro (`.ql`) com os mesmos nomes. Mapeie para o Tailwind (`theme.extend.colors`) apontando para `var(--...)`, de modo que trocar o tema é só trocar a classe na raiz.

## Princípios

- **Cor com significado, nunca só cor:** estados usam ícone e texto além da cor (✓, ✕, +, −). Azul e laranja em vez de verde e vermelho nos diffs.
- **Contraste:** texto mínimo 4.5:1 nos dois temas, inclusive os rótulos pequenos.
- **Abas refletem os arquivos existentes** na spec; arquivos fora do conjunto primário ficam em "outros".
- **Leitura primeiro:** o conteúdo renderizado tem largura máxima de ~736 px; tabelas largas rolam dentro do próprio bloco.
- **Somente leitura é visível:** arquivos gerados por agente e o checkbox desativado mostram um cadeado e dizem o motivo.
- Elementos interativos são `button`, `a` e `input` reais, com `aria-label` em botões só de ícone.

## Fora do wireframe v1

Home sem spec aberta (estado vazio e recentes), modo editar/dividir, command palette, configuração de scan paths e de presets.
