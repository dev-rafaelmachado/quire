// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ProjectSummary } from "../../lib/ipc";

const saveFile = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => (typeof e === "object" && e !== null && "code" in e ? e : { code: "x", message: String(e) }),
  commands: { saveFile: (...a: unknown[]) => saveFile(...a), getSpec: async () => null, takeCliOpen: async () => null },
  events: {},
}));

const { useEditor } = await import("../editor/store");
const { useUnsaved, guard } = await import("./store");
const { UnsavedDialog } = await import("./UnsavedDialog");
const { Sidebar } = await import("../workspace/Sidebar");
const { useWorkspace } = await import("../workspace/store");
const { applyCliOpen } = await import("../workspace/cli");

const LF = { bom: false, eol: "lf" as const, mixed: false, finalNewline: true };
const BUFFER_TEXT = "texto que só existe na memória — ação";

function dirtyBuffer(format = LF) {
  useEditor.setState({
    mode: "edit",
    buffer: { instanceId: 1, name: "spec.md", savedText: "old\n", base: "4-aa", format, readOnly: false, locked: null },
    dirty: true,
    saving: false,
    error: null,
    confirmingMixed: false,
    getText: () => BUFFER_TEXT,
  });
}

const projects: ProjectSummary[] = [
  {
    id: 1,
    key: "k",
    name: "acme_gateway",
    alias: null,
    preset: "exban",
    suggestedPreset: null,
    gitWarning: null,
    specs: [
      {
        id: 5,
        folder: "acme-142-x",
        issueKey: "ACME-142",
        instances: [{ id: 50, path: "/p", repoRoot: "/r", branch: "main", kind: "checkout", mtimeMs: 1, sameAsDefault: true, contentHash: "h", title: "ACME-142 — X", status: null, issueUrl: null, progress: null }],
        defaultIndex: 0,
        versionsBadge: null,
      },
    ],
  },
];

beforeEach(() => {
  saveFile.mockReset();
  useUnsaved.setState({ pending: null, busy: false, error: null, needsMixed: false });
  useWorkspace.setState({ projects, loaded: true, selectedSpecId: null, collapsed: {}, text: "", status: "all", highlight: null, pick: null, dismissedSuggestions: {}, config: null, scan: null, error: null });
});
afterEach(cleanup);

describe("unsaved changes guard (SAV-06, SAV-07)", () => {
  it("navigates at once when nothing is unsaved", () => {
    useEditor.setState({ dirty: false, buffer: null });
    const next = vi.fn();
    guard(next);
    expect(next).toHaveBeenCalledOnce();
  });

  it("holds the navigation until a choice; Cancelar keeps everything", async () => {
    dirtyBuffer();
    const next = vi.fn();
    guard(next);
    expect(next).not.toHaveBeenCalled();
    await useUnsaved.getState().choose("cancel");
    expect(next).not.toHaveBeenCalled();
    expect(useEditor.getState().dirty).toBe(true);
    expect(saveFile).not.toHaveBeenCalled();
  });

  it("Descartar navigates without writing", async () => {
    dirtyBuffer();
    const next = vi.fn();
    guard(next);
    await useUnsaved.getState().choose("discard");
    expect(next).toHaveBeenCalledOnce();
    expect(saveFile).not.toHaveBeenCalled();
  });

  it("Salvar writes, then navigates; a failed save keeps the navigation on hold", async () => {
    dirtyBuffer();
    saveFile.mockRejectedValueOnce({ code: "write_conflict", message: "changed" });
    const next = vi.fn();
    guard(next);
    await useUnsaved.getState().choose("save");
    expect(next).not.toHaveBeenCalled();
    expect(useUnsaved.getState().error?.code).toBe("write_conflict");
    saveFile.mockResolvedValueOnce({ fingerprint: "4-bb", format: LF });
    await useUnsaved.getState().choose("save");
    expect(saveFile).toHaveBeenLastCalledWith(expect.objectContaining({ text: BUFFER_TEXT, base: "4-aa" }));
    expect(next).toHaveBeenCalledOnce();
  });

  it("mixed line endings are confirmed inside the dialog", async () => {
    dirtyBuffer({ ...LF, mixed: true });
    const next = vi.fn();
    guard(next);
    await useUnsaved.getState().choose("save");
    expect(useUnsaved.getState().needsMixed).toBe(true);
    expect(saveFile).not.toHaveBeenCalled();
    saveFile.mockResolvedValueOnce({ fingerprint: "4-cc", format: LF });
    await useUnsaved.getState().choose("save", { confirmMixed: true });
    expect(saveFile.mock.calls[0]?.[0]).toMatchObject({ confirmMixed: true });
    expect(next).toHaveBeenCalledOnce();
  });

  it("the sidebar does not change spec until the choice", async () => {
    dirtyBuffer();
    render(
      <>
        <Sidebar />
        <UnsavedDialog />
      </>,
    );
    fireEvent.click(screen.getByText("ACME-142"));
    expect(useWorkspace.getState().selectedSpecId).toBeNull();
    expect(screen.getByRole("alertdialog", { name: "Salvar as alterações em spec.md?" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(useWorkspace.getState().selectedSpecId).toBeNull();
    fireEvent.click(screen.getByText("ACME-142"));
    fireEvent.click(screen.getByRole("button", { name: "Descartar" }));
    await waitFor(() => expect(useWorkspace.getState().selectedSpecId).toBe(5));
  });

  it("a `quire open` navigation passes the guard too", async () => {
    dirtyBuffer();
    guard(() => applyCliOpen({ kind: "spec", specId: 5, instanceId: null, file: null }));
    expect(useWorkspace.getState().selectedSpecId).toBeNull();
    await useUnsaved.getState().choose("cancel");
    expect(useWorkspace.getState().selectedSpecId).toBeNull();
    guard(() => applyCliOpen({ kind: "spec", specId: 5, instanceId: null, file: null }));
    await useUnsaved.getState().choose("discard");
    expect(useWorkspace.getState().selectedSpecId).toBe(5);
  });

  it("no buffer text ever goes to browser storage (TD-24)", async () => {
    const stored: string[] = [];
    const spy = (s: Storage) => vi.spyOn(s, "setItem").mockImplementation((k, v) => void stored.push(`${k}=${v}`));
    const local = spy(window.localStorage);
    const session = spy(window.sessionStorage);
    const idb = vi.fn();
    Object.defineProperty(window, "indexedDB", { value: { open: idb }, configurable: true });
    dirtyBuffer();
    render(
      <>
        <Sidebar />
        <UnsavedDialog />
      </>,
    );
    fireEvent.click(screen.getByText("ACME-142"));
    fireEvent.click(screen.getByRole("button", { name: "Descartar" }));
    dirtyBuffer();
    guard(() => undefined);
    saveFile.mockResolvedValueOnce({ fingerprint: "4-bb", format: LF });
    await useUnsaved.getState().choose("save");
    expect(stored.filter((s) => s.includes("memória"))).toEqual([]);
    expect(idb).not.toHaveBeenCalled();
    local.mockRestore();
    session.mockRestore();
  });
});
