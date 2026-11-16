// @vitest-environment jsdom
// Regression tests for two ways of losing unsaved text (v0.2 T18, T22): the whole app is
// mounted on a mocked IPC, with the real CodeMirror editor.
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ConfigView, ProjectSummary, SpecDetail } from "./lib/ipc";
import type { EditableFile_Serialize as EditableFile } from "./lib/ipc/bindings";

const LF = { bom: false, eol: "lf" as const, mixed: false, finalNewline: true };
const TEXT = "# ACME-142 — Tokens\n\n## Problema\n\nTexto do disco.\n";
const TYPED = "Digitado e não salvo.";

const instance = {
  id: 10,
  path: "/ws/acme_gateway/.specs/features/acme-142-service-tokens",
  repoRoot: "/ws/acme_gateway",
  branch: "main",
  kind: "checkout" as const,
  mtimeMs: 1,
  sameAsDefault: true,
  contentHash: "h",
  title: "ACME-142 — Tokens",
  status: null,
  issueUrl: null,
  progress: null,
};
const projects: ProjectSummary[] = [
  {
    id: 1,
    key: "k",
    name: "acme_gateway",
    alias: null,
    preset: "exban",
    suggestedPreset: null,
    gitWarning: null,
    specs: [{ id: 5, folder: "acme-142-service-tokens", issueKey: "ACME-142", instances: [instance], defaultIndex: 0, versionsBadge: null }],
  },
];
const detail: SpecDetail = {
  project: { id: 1, name: "acme_gateway", displayName: "acme_gateway", preset: "exban", issueUrlBase: null },
  spec: projects[0]!.specs[0]!,
  instanceIndex: 0,
  files: [],
  tabs: ["spec.md"],
  others: [],
  header: [],
  headerFiles: [],
};
const config: ConfigView = {
  path: "/cfg/config.toml",
  error: null,
  scanPaths: ["/ws"],
  pollIntervalMs: 3000,
  projects: [],
  presets: ["exban", "generic"],
  theme: null,
  language: null,
  scanPathStatus: [],
  editable: true,
};
const editable: EditableFile = { name: "spec.md", text: TEXT, format: LF, fingerprint: "40-aa", locked: null, readOnly: false };

const saveFile = vi.fn();
const handlers: Record<string, (...a: unknown[]) => unknown> = {
  getConfig: () => config,
  getWatchStatus: () => ({ repos: 1, native: 1, polled: 0, pollIntervalMs: 3000, fallbacks: [] }),
  listProjects: () => projects,
  startScan: () => null,
  takeCliOpen: () => null,
  getSpec: () => detail,
  readFile: () => ({ name: "spec.md", kind: "markdown", size: TEXT.length, text: TEXT, large: false }),
  openForEdit: () => editable,
  orphanTemps: () => [],
  fileFingerprint: () => "40-aa",
  search: () => ({ groups: [], truncated: false, terms: [] }),
  saveFile: (...a) => saveFile(...a),
};
vi.mock("./lib/ipc", () => ({
  asCmdError: (e: unknown) => (typeof e === "object" && e !== null && "code" in e ? e : { code: "x", message: String(e) }),
  commands: new Proxy({}, { get: (_, name: string) => async (...a: unknown[]) => handlers[name]?.(...a) ?? null }),
  events: new Proxy({}, { get: () => ({ listen: async () => () => undefined }) }),
}));
vi.mock("./lib/markdown/highlight", () => ({ highlight: async () => null, cachedHighlight: () => undefined, canHighlight: () => false }));

const { App } = await import("./App");
const { currentEditor } = await import("./features/editor/EditorPane");
const { useEditor } = await import("./features/editor/store");
const { useSearch } = await import("./features/search/store");
const { useUnsaved } = await import("./features/unsaved/store");
const { useWorkspace } = await import("./features/workspace/store");

/** Mounts the app on ACME-142 in Editar and types `TYPED` without saving. */
async function editWithoutSaving() {
  useWorkspace.setState({ selectedSpecId: 5 });
  useEditor.setState({ mode: "edit" });
  render(<App />);
  await waitFor(() => expect(currentEditor()).not.toBeNull(), { timeout: 5000 });
  const view = currentEditor()!.view;
  act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: TYPED }, userEvent: "input.type" }));
  expect(useEditor.getState().dirty).toBe(true);
  return currentEditor()!;
}

function modeButton(name: "Ler" | "Editar" | "Dividir") {
  return within(screen.getByRole("group", { name: "Modo do arquivo" })).getByRole("button", { name });
}

beforeEach(() => {
  saveFile.mockReset();
  useUnsaved.setState({ pending: null, busy: false, error: null, needsMixed: false });
  useSearch.setState({ open: false, text: "", results: null, terms: [] });
  useEditor.setState({ mode: "read", buffer: null, dirty: false, getText: null });
});
afterEach(cleanup);

describe("unsaved text survives (regressions)", () => {
  it("Editar → Dividir → Ler → Editar keeps the typed text and the unsaved state", async () => {
    const first = await editWithoutSaving();
    fireEvent.click(modeButton("Dividir"));
    await waitFor(() => expect(useEditor.getState().mode).toBe("split"));
    // Same editor, same text: switching to Dividir must not remount it.
    expect(currentEditor()).toBe(first);
    expect(currentEditor()?.getText()).toContain(TYPED);
    expect(useEditor.getState().dirty).toBe(true);

    // Ler drops the editor, so it asks first; Cancelar keeps everything.
    fireEvent.click(modeButton("Ler"));
    expect(screen.getByRole("alertdialog", { name: "Salvar as alterações em spec.md?" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(useEditor.getState().mode).toBe("split");

    fireEvent.click(modeButton("Editar"));
    await waitFor(() => expect(useEditor.getState().mode).toBe("edit"));
    expect(currentEditor()).toBe(first);
    expect(currentEditor()?.getText()).toContain(TYPED);
    expect(useEditor.getState().dirty).toBe(true);
    expect(saveFile).not.toHaveBeenCalled();
  });

  it("opening the search by Ctrl+Shift+F or the magnifier asks first; Cancelar keeps the editor", async () => {
    const first = await editWithoutSaving();
    for (const open of [
      () => fireEvent.keyDown(window, { key: "F", ctrlKey: true, shiftKey: true }),
      () => fireEvent.click(screen.getByRole("button", { name: "Buscar nas specs (Ctrl+Shift+F)" })),
    ]) {
      open();
      expect(screen.getByRole("alertdialog", { name: "Salvar as alterações em spec.md?" })).toBeTruthy();
      expect(useSearch.getState().open).toBe(false);
      fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
      expect(screen.queryByRole("alertdialog")).toBeNull();
      expect(useSearch.getState().open).toBe(false);
      expect(currentEditor()).toBe(first);
      expect(currentEditor()?.getText()).toContain(TYPED);
      expect(useEditor.getState().dirty).toBe(true);
    }
    expect(saveFile).not.toHaveBeenCalled();
  });
});
