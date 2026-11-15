# Quire

> A quire is a gathering of folded sheets bound together. A spec folder is the same thing: `spec`, `design`, `tasks`, `summary` and friends, read as one unit.

Quire is a lightweight, local-first desktop app (Tauri) to **read and edit spec-driven folders** (`spec.md`, `design.md`, `tasks.md`, ...) across many repositories and git worktrees, in a single place.

**Status:** pre-alpha. This repository currently holds the product ideation, the visual wireframe and the specs for v0.1. No application code yet.

## Why

When every issue gets a spec folder, the files end up scattered across repos and worktrees and get opened in generic markdown clients that know nothing about specs. Quire treats the **spec folder** as the unit: it knows its status, task progress, requirement IDs and agent logs, and shows all copies of the same spec (one per worktree) as one entry.

## What v0.1 will do

- Scan configured paths for `.specs/` folders, group specs by project (git repository) and deduplicate copies across worktrees.
- Show each spec with tabs for its files: `spec`, `design`, `tasks`, `summary`, `validation`, `gates`, `lessons` and a generic fallback for any other file.
- Render markdown (GFM, tables, code, Mermaid) with an outline, and `tasks.md` by phase with dependencies.
- Show `gates.jsonl` and `lessons.jsonl` as read-only timelines.
- React to file changes made by other tools or agents (file watcher).
- Open from the terminal: `quire .`, `quire open EX-1452`.

The file formats follow a configurable **preset**. A generic core reads any markdown and JSONL; a preset describes a specific spec-driven convention (the first one mirrors the author's team workflow).

## Docs

- [Ideation and MoSCoW](docs/ideation/moscow.md)
- [Spec folder format notes](docs/spec-format.md)
- [Wireframes and design tokens](docs/design/README.md)
- [v0.1 specs](.specs/features/quire-v0-1/spec.md) (this project is built with its own workflow)

## Stack (planned)

Tauri 2, Rust core (`ignore`, `notify`, SQLite), React + TypeScript + Tailwind, CodeMirror 6, unified/remark/rehype with Mermaid. Linux first, Windows second.

## Contributing

The project is in the design phase. See [AGENTS.md](AGENTS.md) for conventions. Commits follow [Conventional Commits](https://www.conventionalcommits.org/).

## License

[MIT](LICENSE)
