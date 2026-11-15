# AGENTS.md — Quire

Instruções para agentes (Claude Code, Codex, opencode) que trabalham neste repositório. Idioma de conversa com o autor: pt-BR. Código, identificadores, commits e README: inglês.

## O que é o Quire

App desktop local (Tauri 2) e open source para ler e editar pastas de spec spec-driven (`spec.md`, `design.md`, `tasks.md`, `summary.md`, `validation.md`, `gates.jsonl`, `lessons.jsonl`) de vários repositórios e worktrees num só lugar. A unidade do app é a **pasta da spec**, não o arquivo.

Leia antes de qualquer trabalho:

1. `.specs/features/quire-v0-1/spec.md` — o que construir e os critérios de aceite (IDs rastreáveis).
2. `.specs/features/quire-v0-1/design.md` — arquitetura e decisões.
3. `.specs/features/quire-v0-1/tasks.md` — plano de execução; é a fonte da ordem do trabalho.
4. `docs/ideation/moscow.md`, `docs/spec-format.md`, `docs/design/README.md` — contexto de produto, formato dos arquivos e referência visual.

## Fluxo de trabalho (spec-driven)

- Implemente as tasks do `tasks.md` **na ordem das fases**, uma por vez. Cada task tem "Pronto quando" e "Gate".
- Só marque `- [x]` depois que o gate da task passou, e acrescente o hash curto do commit no fim da linha, como em `- [x] T1: título (a1b2c3d)`.
- Se um critério da spec estiver ambíguo ou uma decisão do design não funcionar na prática, **pare e registre** em `.specs/features/quire-v0-1/summary.md` (seção Journal) em vez de improvisar.
- Atualize `summary.md` (Onde parou + Journal) ao fim de cada sessão.
- Não implemente nada marcado como Won't ou fora do escopo da v0.1 sem pedir.

## Estrutura alvo

```
quire/
├─ apps/desktop/          # Vite + React + TS; src-tauri/ com o shell Tauri
├─ crates/quire-core/     # lib Rust: scan, parsers, índice, presets (testável sem Tauri)
├─ fixtures/              # workspace sintético para testes e demo
├─ docs/                  # ideação, formato, design (wireframes, tokens)
└─ .specs/features/       # specs do próprio Quire (dogfooding)
```

## Convenções

- **TypeScript estrito:** `strict: true`, sem `any` implícito nem explícito, sem `@ts-ignore` sem comentário justificando.
- **Rust:** `cargo fmt` e `cargo clippy -- -D warnings` limpos. Erros tipados (`thiserror`) na lib; sem `unwrap()` fora de testes.
- **Modularidade:** `quire-core` não depende de Tauri. O frontend fala com o core só por comandos e eventos tipados do Tauri.
- **Arquivos são a fonte da verdade.** O app nunca guarda estado de spec fora do disco além do cache de índice, que pode ser apagado e reconstruído.
- **Escrita no disco só por ação explícita do usuário.** `gates.jsonl` e `lessons.jsonl` são somente leitura. O checkbox do `tasks.md` é opt-in por projeto (desligado por padrão).
- **Presets, não hardcode.** Nada específico de uma empresa no núcleo; a convenção da Exban vive no preset `exban`.
- **UI:** strings centralizadas em catálogo (pt-BR primeiro, en depois). Tokens de cor vêm de `docs/design/tokens.css`; tema escuro é o padrão e o claro usa os mesmos tokens.
- **Acessibilidade:** elementos interativos são `button`/`a`/`input` reais, com `aria-label` em botões só de ícone; contraste mínimo 4.5:1.

## Dados e privacidade

- **Nunca** commite specs, logs ou caminhos reais de projetos da Exban ou de qualquer empresa. Testes e demos usam `fixtures/` (conteúdo sintético inspirado no formato, sem dados reais).
- Sem telemetria. Sem rede, exceto abrir links externos a pedido do usuário.

## Ambiente

- Linux é o alvo principal; Windows nativo é o segundo. No WSL, eventos de arquivo em `\\wsl$` não chegam: o watcher precisa de fallback por polling (ver design.md, TD-05).
- Gerenciador de pacotes JS: `pnpm`. Rust estável via `rustup`.
- Comandos (confirmar no spike, T1–T3): `pnpm install`, `pnpm --filter desktop tauri dev`, `cargo test --workspace`, `pnpm -r typecheck`, `pnpm -r test`.

## Git

- Commits no padrão **Conventional Commits** (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, com escopo opcional, ex.: `feat(core): scan .specs folders`).
- **Não** adicione `Co-Authored-By` nem trailers de atribuição a commits ou PRs.
- Branches: `feat/<assunto>`, `fix/<assunto>`, `docs/<assunto>`. Nunca faça push, abra PR ou crie tag sem o autor pedir.
