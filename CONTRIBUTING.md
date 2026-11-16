# Contributing to Quire

Thanks for helping. Quire is small on purpose: read [AGENTS.md](AGENTS.md) first, it holds the conventions for people and coding agents alike.

## Toolchain

- **Rust** stable via `rustup`, with `clippy` and `rustfmt`.
- **Node** 24+ and **pnpm** (the version pinned in `package.json`).
- **Linux:** WebKitGTK 4.1 and the Tauri build dependencies. On Debian/Ubuntu:
  `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
  On Arch: `sudo pacman -S --needed webkit2gtk-4.1 base-devel`.
- **Windows:** WebView2 (preinstalled on Windows 11) and the MSVC build tools.
- `bash` and `git` for the test fixtures (Git Bash on Windows).

## Run

```bash
pnpm install
scripts/make-fixtures.sh
QUIRE_CONFIG="$PWD/fixtures/quire.toml" pnpm --filter desktop tauri dev
```

## Layout

```
crates/quire-core/      scan, git, model, presets, parsers, SQLite index, watcher, config (no Tauri)
apps/desktop/src-tauri/ Tauri shell: typed commands and events, CLI, single instance
apps/desktop/src/       React UI: features/, lib/ (ipc, markdown, i18n), ui/ (base components)
fixtures/               synthetic workspace templates; scripts/make-fixtures.sh builds it
.specs/features/        Quire's own specs
```

The UI talks to the core only through the commands in `src-tauri/src/commands.rs`. Their TypeScript types are generated into `apps/desktop/src/lib/ipc/bindings.ts` by `cargo test -p quire-desktop`; that test fails when the file is stale, so commit the regenerated file.

## Before a commit (phase gate)

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm -r typecheck
pnpm -r test
```

CI runs the same on Linux and Windows. A release build is `pnpm --filter desktop tauri build` (a plain `cargo build --release` compiles, but the binary expects the Vite dev server).

## Rules that matter

- **Synthetic data only.** Never commit specs, logs or paths from a real company or project; tests and demos use `fixtures/`.
- **Files are the source of truth.** Quire writes to disk only on an explicit user action (saving preferences). `gates.jsonl` and `lessons.jsonl` are read-only.
- **Presets, not hard-coding.** Nothing team-specific in the core; conventions live in presets.
- **Strings in the catalog.** Every visible string is in `apps/desktop/src/lib/i18n/pt-BR.ts`; a test fails on literal text in components.
- **Accessibility.** Interactive elements are real `button`, `a` or `input`; icon-only buttons have a label (`IconButton` requires it). Text contrast is at least 4.5:1 in both themes (tested).
- **Rust:** typed errors (`thiserror`), no `unwrap()` outside tests, no `unsafe`.
- **TypeScript:** strict, no `any`.

## Commits and branches

[Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, optional scope such as `feat(core): …`). Branches `feat/<topic>`, `fix/<topic>`, `docs/<topic>`.

Work follows the spec-driven flow in `.specs/features/<feature>/`: one task at a time, its gate green, then the task is checked in `tasks.md` with the short commit hash, and `summary.md` is updated.
