import { create } from "zustand";
import type { CmdError } from "../../lib/ipc";
import { useEditor } from "../editor/store";

export type Choice = "save" | "discard" | "cancel";

interface UnsavedState {
  /** The navigation waiting for the user's choice (SAV-06). */
  pending: (() => void | Promise<void>) | null;
  busy: boolean;
  /** Why "Salvar" did not go through (conflict, in use…): the navigation stays on hold. */
  error: CmdError | null;
  /** Saving needs the mixed line endings confirmation first. */
  needsMixed: boolean;
  /** Runs `next` now when nothing is unsaved; otherwise asks first (SAV-06). Nothing is
   *  written without the explicit choice (SAV-07). */
  guard: (next: () => void | Promise<void>) => void;
  choose: (choice: Choice, opts?: { confirmMixed?: boolean }) => Promise<void>;
}

export function hasUnsaved(): boolean {
  const e = useEditor.getState();
  return e.dirty && e.buffer !== null;
}

export const useUnsaved = create<UnsavedState>((set, get) => ({
  pending: null,
  busy: false,
  error: null,
  needsMixed: false,
  guard: (next) => {
    if (!hasUnsaved()) {
      void next();
      return;
    }
    set({ pending: next, error: null, needsMixed: false });
  },
  choose: async (choice, opts = {}) => {
    const next = get().pending;
    if (!next || get().busy) return;
    if (choice === "cancel") {
      set({ pending: null, error: null, needsMixed: false });
      return;
    }
    if (choice === "discard") {
      // The buffer is dropped in memory; the file on disk stays as it is.
      useEditor.getState().setDirty(false);
      set({ pending: null, error: null, needsMixed: false });
      await next();
      return;
    }
    set({ busy: true, error: null });
    const result = await useEditor.getState().save({ confirmMixed: opts.confirmMixed ?? false });
    if (result === "saved" || result === "unchanged") {
      set({ busy: false, pending: null, needsMixed: false });
      await next();
    } else if (result === "needs-confirm") {
      useEditor.getState().cancelMixed();
      set({ busy: false, needsMixed: true });
    } else {
      set({ busy: false, error: useEditor.getState().error });
    }
  },
}));

/** `guard` as a plain function, for call sites outside React. */
export function guard(next: () => void | Promise<void>): void {
  useUnsaved.getState().guard(next);
}
