import { create } from "zustand";
import { asCmdError, type CmdError, commands, type TextFormat } from "../../lib/ipc";

export type EditMode = "read" | "edit" | "split";

/** The file of the center column as the editor knows it. The text being edited lives only in
 *  the CodeMirror state in memory (TD-24): never in storage, never written on its own. */
export interface Buffer {
  instanceId: number;
  name: string;
  /** Text as last loaded or saved (LF, no BOM). */
  savedText: string;
  /** Fingerprint token of the disk content `savedText` came from: the base of the next save. */
  base: string | null;
  format: TextFormat | null;
  readOnly: boolean;
  /** Why Editar and Dividir are disabled (EDT-08). */
  locked: CmdError | null;
  /** Text the editor starts with when it is not the saved one: edits carried over from a
   *  copy that disappeared (INS-04). */
  initialText?: string;
}

export type SaveResult = "saved" | "unchanged" | "failed" | "needs-confirm";

export interface SaveOptions {
  confirmMixed?: boolean;
  /** Base to save against instead of the buffer's: the disk the user chose to overwrite
   *  (CNF-02). `null` re-creates a file deleted during the edit (CNF-07). */
  base?: string | null;
}

let onConflict: (() => void) | null = null;
/** Who handles a `write_conflict` (the conflict dialog, T19). */
export function setConflictHandler(fn: (() => void) | null): void {
  onConflict = fn;
}

interface EditorState {
  mode: EditMode;
  buffer: Buffer | null;
  /** The buffer differs from `savedText` (EDT-06). */
  dirty: boolean;
  saving: boolean;
  error: CmdError | null;
  /** The file mixes line endings and the user has not confirmed the normalization yet. */
  confirmingMixed: boolean;
  /** Bumped when the saved text changes, so the open editor resets what "clean" means. */
  savedVersion: number;
  /** Bumped after each successful save only (the read view and the lists follow it). */
  lastSaved: number;
  /** Replaces the buffer with what is on disk now (CNF-04, "Recarregar"). */
  adopt: (disk: { text: string; fingerprint: string; format: TextFormat }) => void;
  /** Set by the mounted editor; the store pulls the text only when saving. */
  getText: (() => string) | null;
  setGetText: (fn: (() => string) | null) => void;
  setDirty: (dirty: boolean) => void;
  setMode: (mode: EditMode) => void;
  /** Loads what the editor needs for a file of the center column. */
  load: (instanceId: number, name: string) => Promise<void>;
  clear: () => void;
  /** Saves the buffer (EDT-05): nothing is written when it is clean. */
  save: (opts?: SaveOptions) => Promise<SaveResult>;
  cancelMixed: () => void;
  /** Edits waiting to land in another instance of the same file (INS-04). */
  carry: { instanceId: number; name: string; text: string } | null;
  /** Takes the unsaved text to `instanceId` before the view switches to it (INS-04). */
  carryTo: (instanceId: number) => void;
}

/** Lock reason of a file the backend is not even asked about (EDT-08). */
export function lockByName(name: string): CmdError | null {
  const lower = name.toLowerCase();
  if (lower.endsWith(".jsonl")) return { code: "edit_jsonl", message: name };
  if (!lower.endsWith(".md")) return { code: "edit_not_markdown", message: name };
  return null;
}

let loadSeq = 0;
// `?mode=edit|split` starts in that mode (screenshots and measurements, like `?open=`).
const URL_MODE: EditMode =
  typeof window === "undefined"
    ? "read"
    : (() => {
        const m = new URLSearchParams(window.location.search).get("mode");
        return m === "edit" || m === "split" ? m : "read";
      })();

export const useEditor = create<EditorState>((set, get) => ({
  mode: URL_MODE,
  buffer: null,
  dirty: false,
  saving: false,
  error: null,
  confirmingMixed: false,
  savedVersion: 0,
  lastSaved: 0,
  getText: null,
  carry: null,
  carryTo: (instanceId) => {
    const { buffer, getText } = get();
    if (!buffer || !getText) return;
    set({ carry: { instanceId, name: buffer.name, text: getText() } });
  },
  adopt: (disk) => {
    const buffer = get().buffer;
    if (!buffer) return;
    set({
      buffer: { ...buffer, savedText: disk.text, base: disk.fingerprint, format: disk.format },
      dirty: false,
      error: null,
      savedVersion: get().savedVersion + 1,
    });
  },
  setGetText: (getText) => set({ getText }),
  setDirty: (dirty) => set({ dirty }),
  setMode: (mode) => {
    const locked = get().buffer?.locked ?? null;
    set({ mode: mode !== "read" && locked ? "read" : mode, error: null });
  },
  load: async (instanceId, name) => {
    const seq = ++loadSeq;
    const byName = lockByName(name);
    const base: Buffer = { instanceId, name, savedText: "", base: null, format: null, readOnly: false, locked: byName };
    set({ buffer: base, dirty: false, error: null, confirmingMixed: false, savedVersion: get().savedVersion + 1 });
    if (byName) {
      set({ mode: "read" });
      return;
    }
    try {
      const file = await commands.openForEdit(instanceId, name);
      if (seq !== loadSeq) return;
      const carry = get().carry;
      const carried = carry !== null && carry.instanceId === instanceId && carry.name === name && !file.locked ? carry.text : null;
      set({
        buffer: {
          instanceId,
          name,
          savedText: file.text ?? "",
          base: file.fingerprint,
          format: file.format,
          readOnly: file.readOnly,
          locked: file.locked,
          ...(carried !== null ? { initialText: carried } : {}),
        },
        mode: file.locked ? "read" : get().mode,
        savedVersion: get().savedVersion + 1,
        carry: null,
        dirty: carried !== null && carried !== (file.text ?? ""),
      });
      // The carried edits meet this copy's content: the conflict dialog decides (INS-04).
      if (carried !== null && carried !== (file.text ?? "")) onConflict?.();
    } catch (e) {
      if (seq !== loadSeq) return;
      set({ buffer: { ...base, locked: asCmdError(e) }, mode: "read" });
    }
  },
  clear: () => {
    loadSeq++;
    set({ buffer: null, dirty: false, error: null, confirmingMixed: false, getText: null });
  },
  save: async (opts = {}) => {
    const { buffer, dirty, getText, saving } = get();
    if (!buffer || !buffer.format || !getText || saving || buffer.locked) return "unchanged";
    const forced = opts.base !== undefined;
    if (!dirty && !forced) return "unchanged";
    if (buffer.format.mixed && !opts.confirmMixed) {
      set({ confirmingMixed: true });
      return "needs-confirm";
    }
    const text = getText();
    set({ saving: true, error: null, confirmingMixed: false });
    try {
      const saved = await commands.saveFile({
        instanceId: buffer.instanceId,
        name: buffer.name,
        text,
        format: buffer.format,
        base: forced ? (opts.base ?? null) : buffer.base,
        confirmMixed: opts.confirmMixed ?? false,
      });
      const current = get().buffer;
      if (current?.instanceId === buffer.instanceId && current.name === buffer.name) {
        set({
          buffer: { ...current, savedText: text, base: saved.fingerprint, format: saved.format },
          savedVersion: get().savedVersion + 1,
        });
      }
      set({ saving: false, lastSaved: get().lastSaved + 1 });
      return "saved";
    } catch (e) {
      const error = asCmdError(e);
      set({ saving: false, error });
      if (error.code === "write_conflict") onConflict?.();
      return "failed";
    }
  },
  cancelMixed: () => set({ confirmingMixed: false }),
}));
