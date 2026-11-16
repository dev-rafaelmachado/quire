# Changelog

All notable changes to Quire. Versions follow [Semantic Versioning](https://semver.org/).
Release candidates are tagged `vX.Y.Z-rc.N` and built from the same `X.Y.Z` version: the
Windows installer accepts only numeric versions.

## v0.2.0

Editing, search and the command palette.

- **Edit Markdown safely:** Ler / Editar / Dividir on every `.md` of a spec, with a CodeMirror
  editor, a live preview that scrolls along and Ctrl+S to save.
- **Atomic writes:** every save goes through a hidden temporary file, fsync and rename, and keeps
  the line endings, BOM, final newline, permissions and symlinks of the file. Read-only files
  and WSL mounts (DrvFs, 9p, `\\wsl$`), where the rename is not verified, stay read-only.
- **Unsaved changes are never dropped silently:** switching file, tab, mode, spec or opening the
  search asks first; closing the window too.
- **Agent conflicts:** when the file you edit changes on disk, Quire reloads it if you have no
  unsaved changes, and otherwise lets you reload, overwrite, save a copy or see the line diff.
- **Leftover temporary files** from a crash are found and removed only on request.
- **Full-text search** across every spec (accents and case ignored, prefix on the last word),
  grouped by spec and filtered by project, file or the open spec, straight to the section.
- **Command palette** (Ctrl+K): open a spec, switch tab or copy, change mode, save, search,
  switch theme.
- **Task checkboxes**, opt-in per project and off by default: only the character between the
  brackets changes.
- **English or Portuguese**, following the system language, with a selector in Preferences.
- **Windows:** the `quire` command works in `cmd` and PowerShell through a console helper
  installed by the MSI.

## v0.1.0

First release: a reader for spec-driven folders.

- Finds every `.specs/` below the configured folders, including worktrees that live elsewhere.
- One entry per spec, grouped by project and deduplicated across worktrees, with an
  "N versões" badge when copies differ.
- Tabs per file: GitHub-flavored Markdown with syntax highlighting and Mermaid, an outline,
  collapsible sections, `tasks.md` by phase, `gates.jsonl` and `lessons.jsonl` as tables.
- Live updates from disk in about a second, keeping the tab, scroll and collapsed sections,
  with a polling fallback where file events do not arrive (WSL).
- Opens from the terminal: `quire .`, `quire open ACME-142`, `quire open path/to/file`.
- Dark and light themes; no telemetry and no network.
