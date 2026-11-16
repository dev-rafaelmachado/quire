# Quire

> A quire is a gathering of folded sheets bound together. A spec folder is the same thing: `spec`, `design`, `tasks`, `summary` and friends, read as one unit.

Quire is a lightweight, local-first desktop app (Tauri 2) to **read spec-driven folders** (`spec.md`, `design.md`, `tasks.md`, `summary.md`, `validation.md`, `gates.jsonl`, `lessons.jsonl`) across many repositories and git worktrees, in a single place.

**Status:** v0.1 (reading). Editing, full-text search and the command palette are planned for v0.2.

## What it does

- **Finds your specs.** Point Quire at one or more folders; it finds every `.specs/` below them (respecting `.gitignore`, skipping `node_modules`, `target`, `dist`), and also the worktrees of those repositories that live elsewhere.
- **One entry per spec.** Specs are grouped by project (git repository) and deduplicated across worktrees. When copies differ you get an "N versões" badge and can switch the copy you read.
- **Reads each file properly.** Tabs per file, GitHub-flavored markdown with syntax highlighting and Mermaid, an outline and collapsible sections, `tasks.md` by phase with dependencies, `gates.jsonl` and `lessons.jsonl` as read-only tables.
- **Stays live.** Edits made by agents or editors show up in about a second, without losing your tab, scroll position or collapsed sections.
- **Opens from the terminal.** `quire .`, `quire open ACME-142`, `quire open path/to/spec/tasks.md`.

Everything is read from disk: Quire keeps no state of its own except a disposable index cache, sends nothing over the network and never writes to your specs.

## Install

### Linux

Download the `.deb` or the `.AppImage` from the [releases page](https://github.com/dev-rafaelmachado/quire/releases).

```bash
sudo apt install ./Quire_0.1.0_amd64.deb      # installs /usr/bin/quire and a menu entry
```

For the AppImage, make it executable and put it on your `PATH` as `quire`:

```bash
chmod +x Quire_0.1.0_amd64.AppImage
mkdir -p ~/.local/bin && mv Quire_0.1.0_amd64.AppImage ~/.local/bin/quire
```

Quire needs WebKitGTK 4.1 (`libwebkit2gtk-4.1`), present on current desktop distributions.

### Windows

Run the `.msi` from the releases page. To use the command line, add the install folder (by default `C:\Program Files\Quire`) to your `PATH`. Command-line output on Windows is not finished yet: `quire --help` may print nothing there.

### From source

See [CONTRIBUTING.md](CONTRIBUTING.md) for the toolchain, then:

```bash
pnpm install
pnpm --filter desktop tauri build          # binary in target/release/quire, packages in target/release/bundle/
```

## Use

```text
quire                      open the app
quire .                    open the spec of the current folder or worktree
                           (several specs in the worktree: they are highlighted in the list)
quire open ACME-142        open the spec whose folder starts with acme-142
                           (found in more than one project: you pick)
quire open <path>          open the spec folder, or the spec on that file's tab
quire --help | --version
```

A second `quire …` call does not start a second window: the running one comes to front and navigates.

## Configure

The configuration lives in `config.toml` in your configuration directory (`~/.config/quire/` on Linux, `%APPDATA%\quire\` on Windows). Most of it can be edited from **Preferences** (the sliders button at the top right); saving keeps your comments and any section the screen does not show.

```toml
scan_paths = ["~/projects", "~/work"]
poll_interval_ms = 3000          # polling interval where file events are not delivered
theme = "dark"                   # or "light"

[projects."acme_gateway"]        # keyed by project name (the origin repository name)
preset = "exban"                 # how the folder is read; default "generic"
alias = "gateway"
issue_url_base = "https://tracker.example/browse/"

[watch]
mode = "auto"                    # auto | native | poll
[watch.paths]
"/mnt/c/projects" = "poll"       # force polling for a path
```

- **Presets.** `generic` reads any `.md` and `.jsonl`. `exban` reads the convention described in [docs/spec-format.md](docs/spec-format.md) (primary files in tab order, bold header fields, `tasks.md` by phase, requirement ids). Quire suggests `exban` when a project looks like it. You can override a preset or define your own under `[presets.<name>]` (`extends`, `primary_files`, `primary_extensions`, `header_files`, `tasks = "phases" | "none"`, `id_pattern`, `issue_key_pattern`).
- **Watching.** Quire watches only the `.specs` folders and the repository roots. It switches to polling on network and WSL mounts (9p, drvfs, cifs, nfs, fuse) and on `\\wsl$` / UNC paths, or where `[watch]` says so.
- **Index cache.** `~/.local/share/quire/index.sqlite` (Linux). Deleting it is safe: the next start rebuilds it.

`QUIRE_CONFIG` and `QUIRE_INDEX` point to another config file and cache, which is how the demo below runs. Give them absolute paths: `pnpm --filter desktop tauri dev` runs the app from `apps/desktop/src-tauri`, so a relative path would point to a file that does not exist there, and saving Preferences would create it.

## Try it on the synthetic workspace

```bash
scripts/make-fixtures.sh                                         # builds fixtures/workspace (git repos and worktrees)
QUIRE_CONFIG="$PWD/fixtures/quire.toml" pnpm --filter desktop tauri dev
```

## Docs

- [v0.1 spec, design and tasks](.specs/features/quire-v0-1/) (this project is built with its own workflow)
- [Spec folder format](docs/spec-format.md) · [Ideation and MoSCoW](docs/ideation/moscow.md) · [Design and wireframes](docs/design/README.md)
- [End-to-end test script](docs/e2e.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
