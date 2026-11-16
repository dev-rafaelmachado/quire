# Quire

> A quire is a gathering of folded sheets bound together. A spec folder is the same thing: `spec`, `design`, `tasks`, `summary` and friends, read as one unit.

Quire is a lightweight, local-first desktop app (Tauri 2) to **read and edit spec-driven folders** (`spec.md`, `design.md`, `tasks.md`, `summary.md`, `validation.md`, `gates.jsonl`, `lessons.jsonl`) across many repositories and git worktrees, in a single place.

**Status:** v0.2 (editing, search and command palette).

## What it does

- **Finds your specs.** Point Quire at one or more folders; it finds every `.specs/` below them (respecting `.gitignore`, skipping `node_modules`, `target`, `dist`), and also the worktrees of those repositories that live elsewhere.
- **One entry per spec.** Specs are grouped by project (git repository) and deduplicated across worktrees. When copies differ you get an "N versões" badge and can switch the copy you read.
- **Reads each file properly.** Tabs per file, GitHub-flavored markdown with syntax highlighting and Mermaid, an outline and collapsible sections, `tasks.md` by phase with dependencies, `gates.jsonl` and `lessons.jsonl` as read-only tables.
- **Stays live.** Edits made by agents or editors show up in about a second, without losing your tab, scroll position or collapsed sections.
- **Edits Markdown safely.** Ler / Editar / Dividir on every `.md` of a spec: a CodeMirror editor with a live preview that scrolls along, Ctrl+S to save. See "Writing: guarantees and limits" below.
- **Notices the agent.** When an agent changes the file you are editing, Quire reloads it if you have no unsaved changes, and otherwise warns you and lets you reload, overwrite, save a copy or look at the line diff.
- **Searches every spec.** Full-text search (accents and case ignored, prefix on the last word) grouped by spec, filtered by project, file or the open spec, straight to the section.
- **Command palette.** Ctrl+K anywhere: open a spec, switch tab or copy, change mode, save, search (`/` + words), switch theme.
- **Checks tasks, if you want.** Per project and off by default, the `tasks.md` tab can check and uncheck tasks; only the character between the brackets changes.
- **Opens from the terminal.** `quire .`, `quire open ACME-142`, `quire open path/to/spec/tasks.md`.
- **English or Portuguese.** Follows the system language (English for anything else); change it in Preferences.

Everything comes from disk: Quire keeps no state of its own except a disposable index cache and sends nothing over the network. It writes to your specs only when you ask: Save, a choice in the conflict or unsaved-changes dialog, or a task checkbox you turned on.

## Writing: guarantees and limits

- **Never in place.** A save writes a hidden temporary file next to the original, flushes it to disk, renames it over the original and flushes the folder. A failure at any step leaves the original untouched and removes the temporary file. A crash in between can leave a `.<name>.quire-<pid>-<n>.tmp` file: Quire shows it on the spec with a Remove button and never deletes it on its own (a `git add -A` would pick it up).
- **Same bytes as before.** Line endings (LF or CRLF), a UTF-8 BOM and the presence or absence of a final newline are kept, so `git diff` shows only the lines you changed. A file that mixes line endings is normalized to the most used one, after you confirm. Permissions are kept; read-only files and files of another user are never overwritten (you get "Save as a copy"); a symbolic link stays a link and its target gets the content.
- **No silent overwrite.** A save compares the disk with what you loaded (size and hash of the raw bytes, not the modification time), right before writing the temporary file and again right before the rename. If the file changed, nothing is written and you choose. Between that last check and the rename there is a window of microseconds in which another program's write would be replaced; agents do not honor any lock, so it cannot be closed, only kept that small.
- **No autosave, no drafts.** Unsaved text lives only in memory, with its undo history; leaving a file, switching copies, closing the window or a `quire open` asks Save / Discard / Cancel first. A crash loses unsaved changes; Quire does not keep drafts anywhere.
- **Where it does not write.** Only `.md` files are editable: `gates.jsonl` and `lessons.jsonl` stay read-only, and so does any file above 2 MB, not in UTF-8 or binary. On `\\wsl$` / `\\wsl.localhost` (Windows reaching into WSL), on DrvFs (`/mnt/c` inside WSL) and on other 9p mounts, editing is off until an atomic rename is verified there; a mapped drive pointing to `\\wsl$` is not recognized and is treated as a local disk.
- **One copy at a time.** Edits go to the copy on screen (the bar says which branch and worktree) and never spread to the other worktrees.

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

Run the `.msi` from the releases page. It installs `quire-cli.exe` next to `quire.exe` and a `bin\quire.cmd` that it adds to the system `PATH`, so `quire --help`, `quire .` and `quire open …` work from `cmd` and PowerShell (open a new terminal after installing). The app itself starts detached, so the terminal is free at once.

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

| Keys | Action |
| --- | --- |
| Ctrl+K | Command palette (also inside the editor); `/` + words searches the content |
| Ctrl+Shift+F | Search panel |
| Ctrl+S | Save, in Editar and Dividir |

A second `quire …` call does not start a second window: the running one comes to front and navigates.

## Configure

The configuration lives in `config.toml` in your configuration directory (`~/.config/quire/` on Linux, `%APPDATA%\quire\` on Windows). Most of it can be edited from **Preferences** (the sliders button at the top right); saving keeps your comments and any section the screen does not show.

```toml
scan_paths = ["~/projects", "~/work"]
poll_interval_ms = 3000          # polling interval where file events are not delivered
theme = "dark"                   # or "light"
language = "en"                  # "pt-BR" or "en"; absent: the system language, else English

[projects."acme_gateway"]        # keyed by project name (the origin repository name)
preset = "exban"                 # how the folder is read; default "generic"
alias = "gateway"
issue_url_base = "https://tracker.example/browse/"
tasks_checkbox = true            # check tasks from the app; default false

[watch]
mode = "auto"                    # auto | native | poll
[watch.paths]
"/mnt/c/projects" = "poll"       # force polling for a path
```

- **Presets.** `generic` reads any `.md` and `.jsonl`. `exban` reads the convention described in [docs/spec-format.md](docs/spec-format.md) (primary files in tab order, bold header fields, `tasks.md` by phase, requirement ids). Quire suggests `exban` when a project looks like it. You can override a preset or define your own under `[presets.<name>]` (`extends`, `primary_files`, `primary_extensions`, `header_files`, `tasks = "phases" | "none"`, `id_pattern`, `issue_key_pattern`).
- **Watching.** Quire watches only the `.specs` folders and the repository roots. It switches to polling on network and WSL mounts (9p, drvfs, cifs, nfs, fuse) and on `\\wsl$` / UNC paths, or where `[watch]` says so.
- **Index cache.** `~/.local/share/quire/index.sqlite` (Linux). Deleting it is safe: the next start rebuilds it.

`QUIRE_CONFIG` and `QUIRE_INDEX` point to another config file and cache, which is how the demo below runs. They must be absolute paths (relative ones are refused: `pnpm --filter desktop tauri dev` runs the app from `apps/desktop/src-tauri`), and the cache cannot live inside a `.specs` folder.

## Try it on the synthetic workspace

```bash
scripts/make-fixtures.sh                                         # builds fixtures/workspace (git repos and worktrees)
QUIRE_CONFIG="$PWD/fixtures/quire.toml" pnpm --filter desktop tauri dev
```

## Docs

- [v0.1 spec, design and tasks](.specs/features/quire-v0-1/) and [v0.2](.specs/features/quire-v0-2/) (this project is built with its own workflow)
- [Spec folder format](docs/spec-format.md) · [Ideation and MoSCoW](docs/ideation/moscow.md) · [Design and wireframes](docs/design/README.md)
- [End-to-end test script](docs/e2e.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
