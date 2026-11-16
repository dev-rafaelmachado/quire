import { create } from "zustand";
import { asCmdError, type CmdError, commands, type TextFormat } from "../../lib/ipc";
import { currentEditor } from "../editor/EditorPane";
import { setConflictHandler, useEditor } from "../editor/store";

/** What is on disk now; `text` and `fingerprint` are `null` when the file is gone. */
export interface Disk {
  text: string | null;
  fingerprint: string | null;
  format: TextFormat | null;
}

export type DiskDecision = "ignore" | "reload" | "notice" | "deleted" | "gone";

/** TD-14: what a change reported on disk means for the buffer, by fingerprint, never by time.
 *  The app's own write leaves the disk equal to the new base, so it is ignored (CNF-06); an
 *  outside change reloads a clean buffer (CNF-04) and only warns over a dirty one (CNF-05). */
export function diskDecision(base: string | null, dirty: boolean, disk: string | null): DiskDecision {
  if (disk !== null && disk === base) return "ignore";
  if (disk === null) return dirty ? "deleted" : "gone";
  return dirty ? "notice" : "reload";
}

/** `AAAAMMDD-HHMMSS` in local time, for the name of a conflict copy (CNF-08). */
export function stamp(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
}

interface ConflictState {
  /** The conflict dialog is open (CNF-02). */
  open: boolean;
  disk: Disk | null;
  /** Fixed notice over the editor: the file changed or was deleted while dirty (CNF-05, CNF-07). */
  notice: "changed" | "deleted" | null;
  showDiff: boolean;
  busy: boolean;
  error: CmdError | null;
  /** Name of the copy just saved (CNF-08). */
  copied: string | null;
  openDialog: () => Promise<void>;
  close: () => void;
  toggleDiff: () => void;
  /** Called on `spec://changed` for the folder on screen (TD-14). */
  onDiskChange: () => Promise<DiskDecision | null>;
  reload: () => Promise<void>;
  overwrite: () => Promise<void>;
  saveCopy: () => Promise<void>;
  clear: () => void;
}

async function readDisk(): Promise<Disk | null> {
  const b = useEditor.getState().buffer;
  if (!b) return null;
  try {
    const f = await commands.openForEdit(b.instanceId, b.name);
    return { text: f.text, fingerprint: f.fingerprint, format: f.format };
  } catch (e) {
    if (asCmdError(e).code === "gone" || asCmdError(e).code === "not_found") return { text: null, fingerprint: null, format: null };
    throw e;
  }
}

export const useConflict = create<ConflictState>((set, get) => ({
  open: false,
  disk: null,
  notice: null,
  showDiff: false,
  busy: false,
  error: null,
  copied: null,
  openDialog: async () => {
    set({ open: true, busy: true, error: null, copied: null });
    try {
      set({ disk: await readDisk(), busy: false });
    } catch (e) {
      set({ busy: false, error: asCmdError(e) });
    }
  },
  close: () => set({ open: false, showDiff: false, error: null }),
  toggleDiff: () => set({ showDiff: !get().showDiff }),
  onDiskChange: async () => {
    const e = useEditor.getState();
    const b = e.buffer;
    if (!b || b.locked) return null;
    let fp: string | null;
    try {
      fp = await commands.fileFingerprint(b.instanceId, b.name);
    } catch {
      fp = null; // no longer a file of the instance
    }
    const now = useEditor.getState();
    if (now.buffer !== b) return null; // another file meanwhile
    const decision = diskDecision(b.base, now.dirty && now.mode !== "read", fp);
    if (decision === "reload") await get().reload();
    else if (decision === "notice") set({ notice: "changed" });
    else if (decision === "deleted") set({ notice: "deleted" });
    return decision;
  },
  reload: async () => {
    set({ busy: true, error: null });
    try {
      const disk = await readDisk();
      if (!disk || disk.text === null || disk.fingerprint === null || disk.format === null) {
        set({ busy: false, disk, notice: disk ? "deleted" : null });
        return;
      }
      // One undoable transaction in the editor (design: a reload enters the history).
      currentEditor()?.replaceText(disk.text);
      useEditor.getState().adopt({ text: disk.text, fingerprint: disk.fingerprint, format: disk.format });
      set({ busy: false, open: false, showDiff: false, notice: null, disk });
    } catch (e) {
      set({ busy: false, error: asCmdError(e) });
    }
  },
  overwrite: async () => {
    const disk = get().disk ?? (await readDisk());
    set({ busy: true, error: null });
    // An explicit choice: save against what is on disk now, or re-create a deleted file.
    const result = await useEditor.getState().save({ base: disk?.fingerprint ?? null, confirmMixed: true });
    if (result === "saved") set({ busy: false, open: false, showDiff: false, notice: null, disk: null });
    else set({ busy: false, error: useEditor.getState().error });
  },
  saveCopy: async () => {
    const e = useEditor.getState();
    const b = e.buffer;
    if (!b || !b.format || !e.getText) return;
    set({ busy: true, error: null });
    try {
      const copy = await commands.saveCopy(b.instanceId, b.name, e.getText(), b.format, stamp(new Date()));
      set({ copied: copy.name, busy: false });
      // The edits are safe in the copy: the buffer goes back to what is on disk, if anything.
      const disk = await readDisk();
      if (disk && disk.text !== null && disk.fingerprint !== null && disk.format !== null) {
        currentEditor()?.replaceText(disk.text);
        useEditor.getState().adopt({ text: disk.text, fingerprint: disk.fingerprint, format: disk.format });
        set({ open: false, showDiff: false, notice: null, disk });
      }
    } catch (err) {
      set({ busy: false, error: asCmdError(err) });
    }
  },
  clear: () => set({ open: false, disk: null, notice: null, showDiff: false, busy: false, error: null, copied: null }),
}));

setConflictHandler(() => void useConflict.getState().openDialog());

// A conflict belongs to one file: switching files (after the guard) forgets it.
useEditor.subscribe((s, prev) => {
  const a = s.buffer;
  const b = prev.buffer;
  if (a?.instanceId !== b?.instanceId || a?.name !== b?.name) useConflict.getState().clear();
});
