// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const disk = { text: "agent text\n" as string | null, fingerprint: "11-agent" as string | null };
const fileFingerprint = vi.fn(async () => disk.fingerprint);
const openForEdit = vi.fn(async () => {
  if (disk.text === null) throw { code: "gone", message: "removed" };
  return { name: "spec.md", text: disk.text, format: LF, fingerprint: disk.fingerprint, locked: null, readOnly: false };
});
const saveFile = vi.fn();
const saveCopy = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => (typeof e === "object" && e !== null && "code" in e ? e : { code: "x", message: String(e) }),
  commands: {
    fileFingerprint: (...a: unknown[]) => fileFingerprint(...(a as [])),
    openForEdit: (...a: unknown[]) => openForEdit(...(a as [])),
    saveFile: (...a: unknown[]) => saveFile(...a),
    saveCopy: (...a: unknown[]) => saveCopy(...a),
  },
}));
vi.mock("./diff", () => ({ createDiff: () => () => undefined }));

const LF = { bom: false, eol: "lf" as const, mixed: false, finalNewline: true };
const { diskDecision, stamp, useConflict } = await import("./store");
const { useEditor } = await import("../editor/store");
const { ConflictDialog } = await import("./ConflictDialog");

let bufferText = "mine\n";
function buffer(dirty: boolean, base = "5-base") {
  useEditor.setState({
    mode: "edit",
    buffer: { instanceId: 3, name: "spec.md", savedText: "base\n", base, format: LF, readOnly: false, locked: null },
    dirty,
    saving: false,
    error: null,
    confirmingMixed: false,
    getText: () => bufferText,
  });
}

beforeEach(() => {
  disk.text = "agent text\n";
  disk.fingerprint = "11-agent";
  bufferText = "mine\n";
  saveFile.mockReset();
  saveCopy.mockReset();
  useConflict.getState().clear();
});
afterEach(cleanup);

describe("TD-14 decision", () => {
  it("own write ignored, outside change reloads clean and warns dirty, deletion noticed", () => {
    expect(diskDecision("5-a", false, "5-a")).toBe("ignore");
    expect(diskDecision("5-a", true, "5-a")).toBe("ignore");
    expect(diskDecision("5-a", false, "6-b")).toBe("reload");
    expect(diskDecision("5-a", true, "6-b")).toBe("notice");
    expect(diskDecision("5-a", true, null)).toBe("deleted");
    expect(diskDecision("5-a", false, null)).toBe("gone");
  });

  it("a conflict copy is stamped with the local time", () => {
    expect(stamp(new Date(2026, 10, 16, 9, 5, 7))).toBe("20261116-090507");
  });
});

describe("changes on disk while editing (CNF-04..07)", () => {
  it("the app's own save is not a change: no reload, no notice (CNF-06)", async () => {
    buffer(false, "11-agent");
    expect(await useConflict.getState().onDiskChange()).toBe("ignore");
    expect(openForEdit).not.toHaveBeenCalledWith();
    expect(useConflict.getState().notice).toBeNull();
    expect(useEditor.getState().buffer?.savedText).toBe("base\n");
  });

  it("a clean buffer reloads the disk content (CNF-04)", async () => {
    buffer(false);
    expect(await useConflict.getState().onDiskChange()).toBe("reload");
    expect(useEditor.getState().buffer).toMatchObject({ savedText: "agent text\n", base: "11-agent" });
    expect(useEditor.getState().dirty).toBe(false);
  });

  it("a dirty buffer is kept and gets the fixed notice (CNF-05)", async () => {
    buffer(true);
    expect(await useConflict.getState().onDiskChange()).toBe("notice");
    expect(useConflict.getState().notice).toBe("changed");
    expect(useEditor.getState().buffer?.savedText).toBe("base\n");
    expect(saveFile).not.toHaveBeenCalled();
  });

  it("deleted under a dirty buffer: kept, and only re-created on an explicit choice (CNF-07)", async () => {
    buffer(true);
    disk.text = null;
    disk.fingerprint = null;
    expect(await useConflict.getState().onDiskChange()).toBe("deleted");
    expect(saveFile).not.toHaveBeenCalled();
    await useConflict.getState().openDialog();
    render(<ConflictDialog />);
    expect(screen.queryByRole("button", { name: "Recarregar do disco" })).toBeNull();
    saveFile.mockResolvedValue({ fingerprint: "5-mine", format: LF });
    fireEvent.click(screen.getByRole("button", { name: "Recriar o arquivo" }));
    await waitFor(() => expect(saveFile).toHaveBeenCalledWith(expect.objectContaining({ base: null, text: "mine\n" })));
  });
});

describe("conflict dialog (CNF-02, CNF-08, CNF-09)", () => {
  it("a save that hits a conflict opens the dialog with the disk content", async () => {
    buffer(true);
    saveFile.mockRejectedValueOnce({ code: "write_conflict", message: "changed" });
    expect(await useEditor.getState().save()).toBe("failed");
    await waitFor(() => expect(useConflict.getState().open).toBe(true));
    await waitFor(() => expect(useConflict.getState().disk?.text).toBe("agent text\n"));
  });

  it("Ver diferenças, then Sobrescrever writes the buffer against the disk and takes the new base", async () => {
    buffer(true);
    await useConflict.getState().openDialog();
    render(<ConflictDialog />);
    fireEvent.click(screen.getByRole("button", { name: "Ver diferenças" }));
    expect(useConflict.getState().showDiff).toBe(true);
    expect(saveFile).not.toHaveBeenCalled();
    saveFile.mockResolvedValue({ fingerprint: "5-mine", format: LF });
    fireEvent.click(screen.getByRole("button", { name: "Sobrescrever" }));
    await waitFor(() => expect(useConflict.getState().open).toBe(false));
    expect(saveFile).toHaveBeenCalledWith(expect.objectContaining({ base: "11-agent", text: "mine\n" }));
    expect(useEditor.getState().buffer?.base).toBe("5-mine");
  });

  it("Recarregar drops the edits for the disk content", async () => {
    buffer(true);
    await useConflict.getState().openDialog();
    render(<ConflictDialog />);
    fireEvent.click(screen.getByRole("button", { name: "Recarregar do disco" }));
    await waitFor(() => expect(useConflict.getState().open).toBe(false));
    expect(useEditor.getState().buffer).toMatchObject({ savedText: "agent text\n", base: "11-agent" });
    expect(saveFile).not.toHaveBeenCalled();
  });

  it("Salvar como cópia keeps the edits in a new file and the buffer follows the disk", async () => {
    buffer(true);
    await useConflict.getState().openDialog();
    saveCopy.mockResolvedValue({ name: "spec.conflito-20261116-090507.md", fingerprint: "5-mine" });
    render(<ConflictDialog />);
    fireEvent.click(screen.getByRole("button", { name: "Salvar como cópia" }));
    await waitFor(() => expect(saveCopy).toHaveBeenCalled());
    const [, name, text, , when] = saveCopy.mock.calls[0] ?? [];
    expect([name, text]).toEqual(["spec.md", "mine\n"]);
    expect(String(when)).toMatch(/^\d{8}-\d{6}$/);
    await waitFor(() => expect(useConflict.getState().copied).toBe("spec.conflito-20261116-090507.md"));
    expect(useEditor.getState().buffer?.savedText).toBe("agent text\n");
    expect(saveFile).not.toHaveBeenCalled();
  });
});
