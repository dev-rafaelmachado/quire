// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ProjectSummary, SpecSummary } from "../../lib/ipc";

const search = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => e,
  commands: { search: (...a: unknown[]) => search(...a), getSpec: async () => null },
  events: {},
}));

const { Palette, usePaletteShortcut } = await import("./Palette");
const { usePalette } = await import("./store");
const { useWorkspace } = await import("../workspace/store");
const { useEditor } = await import("../editor/store");
const { useUnsaved } = await import("../unsaved/store");
const { useSearch } = await import("../search/store");
const { useTheme } = await import("../../app/theme");

const spec = (id: number, folder: string, key: string, title: string): SpecSummary => ({
  id,
  folder,
  issueKey: key,
  instances: [{ id: id * 10, path: "/p", repoRoot: "/r", branch: "main", kind: "checkout", mtimeMs: 1, sameAsDefault: true, contentHash: "h", title: `${key} — ${title}`, status: null, issueUrl: null, progress: null }],
  defaultIndex: 0,
  versionsBadge: null,
});
const projects: ProjectSummary[] = [
  { id: 1, key: "k", name: "acme_gateway", alias: null, preset: "exban", suggestedPreset: null, gitWarning: null, specs: [spec(1, "acme-142-service-tokens", "ACME-142", "Tokens de serviço"), spec(2, "acme-170-rate-limit", "ACME-170", "Limite de taxa por cliente")] },
];

function Harness() {
  usePaletteShortcut();
  return (
    <>
      <div contentEditable data-testid="editor" />
      <Palette />
    </>
  );
}

beforeEach(() => {
  search.mockReset();
  usePalette.setState({ open: false, query: "", recent: [] });
  useWorkspace.setState({ projects, selectedSpecId: null, prefsOpen: false });
  useEditor.setState({ dirty: false, buffer: null, mode: "read" });
  useUnsaved.setState({ pending: null });
  useSearch.setState({ open: false, text: "", results: null, terms: [] });
});
afterEach(cleanup);

describe("command palette (CMD-01..05)", () => {
  it("Ctrl+K opens it from inside the editor, with the focus in the field", () => {
    render(<Harness />);
    const editor = screen.getByTestId("editor");
    editor.focus();
    fireEvent.keyDown(editor, { key: "k", ctrlKey: true });
    const input = screen.getByRole("combobox", { name: "Comando, spec ou / para buscar no conteúdo" });
    expect(document.activeElement).toBe(input);
    expect(screen.getByRole("dialog", { name: "Paleta de comandos" })).toBeTruthy();
    expect(input.getAttribute("aria-controls")).toBe("palette-list");
  });

  it("matches loosely and runs the choice by keyboard only", async () => {
    render(<Harness />);
    act(() => usePalette.getState().setOpen(true));
    const input = screen.getByRole("combobox");
    fireEvent.change(input, { target: { value: "ldt" } });
    const options = screen.getAllByRole("option");
    expect(options[0]?.textContent).toContain("ACME-170");
    expect(options[0]?.getAttribute("aria-selected")).toBe("true");
    expect(input.getAttribute("aria-activedescendant")).toBe("palette-opt-0");
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(useWorkspace.getState().selectedSpecId).toBe(2));
    expect(usePalette.getState().open).toBe(false);
  });

  it("arrows move, Esc closes, and recent items come first with an empty field", () => {
    usePalette.setState({ recent: ["theme"] });
    render(<Harness />);
    act(() => usePalette.getState().setOpen(true));
    const input = screen.getByRole("combobox");
    expect(screen.getAllByRole("option")[0]?.textContent).toContain("Alternar tema");
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(screen.getAllByRole("option")[1]?.getAttribute("aria-selected")).toBe("true");
    fireEvent.keyDown(input, { key: "ArrowUp" });
    const before = useTheme.getState().theme;
    fireEvent.keyDown(input, { key: "Enter" });
    expect(useTheme.getState().theme).not.toBe(before);
    act(() => usePalette.getState().setOpen(true));
    fireEvent.keyDown(screen.getByRole("combobox"), { key: "Escape" });
    expect(usePalette.getState().open).toBe(false);
  });

  it("searches the content after the slash, in the same panel (CMD-03)", async () => {
    search.mockResolvedValue({
      truncated: false,
      terms: ["integra"],
      groups: [{ specId: 1, folder: "acme-142-service-tokens", issueKey: "ACME-142", title: null, projectId: 1, project: "acme_gateway", hits: [{ instanceId: 10, file: "spec.md", heading: "Problema", line: 9, snippet: "\u0002integra\u0003ções", branch: null }] }],
    });
    render(<Harness />);
    act(() => usePalette.getState().setOpen(true));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "/integra" } });
    await waitFor(() => expect(search).toHaveBeenCalledWith(expect.objectContaining({ text: "integra" })));
    await screen.findByText("ACME-142 · spec.md · Problema");
    expect(screen.getByRole("listbox", { name: "Resultados da busca" })).toBeTruthy();
    fireEvent.keyDown(screen.getByRole("combobox"), { key: "Enter" });
    await waitFor(() => expect(useWorkspace.getState().selectedSpecId).toBe(1));
  });

  it("navigating with unsaved changes asks first (CMD-05)", () => {
    useEditor.setState({ dirty: true, buffer: { instanceId: 1, name: "spec.md", savedText: "", base: "1-a", format: { bom: false, eol: "lf", mixed: false, finalNewline: true }, readOnly: false, locked: null } });
    render(<Harness />);
    act(() => usePalette.getState().setOpen(true));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "acme170" } });
    fireEvent.keyDown(screen.getByRole("combobox"), { key: "Enter" });
    expect(useWorkspace.getState().selectedSpecId).toBeNull();
    expect(useUnsaved.getState().pending).not.toBeNull();
  });
});
