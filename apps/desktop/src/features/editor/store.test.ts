import { beforeEach, describe, expect, it, vi } from "vitest";

const openForEdit = vi.fn();
const saveFile = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => (typeof e === "object" && e !== null && "code" in e ? e : { code: "x", message: String(e) }),
  commands: {
    openForEdit: (...a: unknown[]) => openForEdit(...a),
    saveFile: (...a: unknown[]) => saveFile(...a),
  },
}));

const { useEditor, lockByName } = await import("./store");

const LF = { bom: false, eol: "lf", mixed: false, finalNewline: true } as const;

beforeEach(() => {
  openForEdit.mockReset();
  saveFile.mockReset();
  useEditor.setState({ mode: "read", buffer: null, dirty: false, saving: false, error: null, confirmingMixed: false, getText: null });
});

async function loaded(text = "# a\n", format: object = LF) {
  openForEdit.mockResolvedValue({ name: "spec.md", text, format, fingerprint: "4-aa", locked: null, readOnly: false });
  await useEditor.getState().load(7, "spec.md");
}

describe("editor store (EDT-05, SAV-03, SAV-07)", () => {
  it("a clean buffer saves nothing", async () => {
    await loaded();
    useEditor.getState().setGetText(() => "# a\n");
    expect(await useEditor.getState().save()).toBe("unchanged");
    expect(saveFile).not.toHaveBeenCalled();
  });

  it("saves the text with the loaded base and takes the new base", async () => {
    await loaded();
    useEditor.getState().setGetText(() => "# b\n");
    useEditor.getState().setDirty(true);
    saveFile.mockResolvedValue({ fingerprint: "4-bb", format: LF });
    expect(await useEditor.getState().save()).toBe("saved");
    expect(saveFile).toHaveBeenCalledWith({ instanceId: 7, name: "spec.md", text: "# b\n", format: LF, base: "4-aa", confirmMixed: false });
    expect(useEditor.getState().buffer).toMatchObject({ savedText: "# b\n", base: "4-bb" });
  });

  it("mixed line endings ask first, then save normalized", async () => {
    await loaded("a\nb\n", { ...LF, mixed: true });
    useEditor.getState().setGetText(() => "a\nc\n");
    useEditor.getState().setDirty(true);
    expect(await useEditor.getState().save()).toBe("needs-confirm");
    expect(saveFile).not.toHaveBeenCalled();
    expect(useEditor.getState().confirmingMixed).toBe(true);
    saveFile.mockResolvedValue({ fingerprint: "4-cc", format: LF });
    expect(await useEditor.getState().save({ confirmMixed: true })).toBe("saved");
    expect(saveFile.mock.calls[0]?.[0]).toMatchObject({ confirmMixed: true });
    expect(useEditor.getState().buffer?.format?.mixed).toBe(false);
  });

  it("a failed save keeps the buffer dirty and shows why", async () => {
    await loaded();
    useEditor.getState().setGetText(() => "# b\n");
    useEditor.getState().setDirty(true);
    saveFile.mockRejectedValue({ code: "write_conflict", message: "the file changed on disk" });
    expect(await useEditor.getState().save()).toBe("failed");
    expect(useEditor.getState().dirty).toBe(true);
    expect(useEditor.getState().error?.code).toBe("write_conflict");
    expect(useEditor.getState().buffer?.base).toBe("4-aa");
  });

  it("locked files stay in Ler (EDT-08)", async () => {
    expect(lockByName("gates.jsonl")?.code).toBe("edit_jsonl");
    expect(lockByName("payload.json")?.code).toBe("edit_not_markdown");
    expect(lockByName("notes.MD")).toBeNull();
    useEditor.getState().setMode("edit");
    await useEditor.getState().load(7, "lessons.jsonl");
    expect(useEditor.getState().mode).toBe("read");
    expect(openForEdit).not.toHaveBeenCalled();
    openForEdit.mockResolvedValue({ name: "big.md", text: null, format: null, fingerprint: "1-aa", locked: { code: "edit_too_large", message: "" }, readOnly: false });
    useEditor.setState({ mode: "edit" });
    await useEditor.getState().load(7, "big.md");
    expect(useEditor.getState().mode).toBe("read");
    useEditor.getState().setMode("split");
    expect(useEditor.getState().mode).toBe("read");
  });
});
