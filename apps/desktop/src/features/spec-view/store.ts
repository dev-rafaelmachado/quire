import { create } from "zustand";
import { asCmdError, commands, type FileContent, type SpecDetail } from "../../lib/ipc";
import { ancestry, buildDoc, type Doc, outline, type OutlineEntry, toHast } from "../../lib/markdown/pipeline";

interface SpecViewState {
  specId: number | null;
  detail: SpecDetail | null;
  /** File shown in the center column. */
  file: string | null;
  content: FileContent | null;
  doc: Doc | null;
  outline: OutlineEntry[];
  collapsed: ReadonlySet<string>;
  active: string | null;
  /** Bumped to ask the view to scroll to `active`. */
  scrollRequest: number;
  loading: boolean;
  error: string | null;
  open: (specId: number | null) => Promise<void>;
  show: (file: string) => Promise<void>;
  toggle: (id: string) => void;
  setActive: (id: string | null) => void;
  navigate: (id: string) => void;
}

const EMPTY: ReadonlySet<string> = new Set();

export const useSpecView = create<SpecViewState>((set, get) => ({
  specId: null,
  detail: null,
  file: null,
  content: null,
  doc: null,
  outline: [],
  collapsed: EMPTY,
  active: null,
  scrollRequest: 0,
  loading: false,
  error: null,
  open: async (specId) => {
    set({ specId, detail: null, file: null, content: null, doc: null, outline: [], collapsed: EMPTY, active: null, error: null });
    if (specId === null) return;
    set({ loading: true });
    try {
      const detail = await commands.getSpec(specId, null);
      if (get().specId !== specId) return;
      set({ detail });
      const first = detail.tabs.find((f) => f.endsWith(".md")) ?? detail.tabs[0] ?? detail.others[0];
      if (first) await get().show(first);
    } catch (e) {
      set({ error: asCmdError(e).message });
    } finally {
      set({ loading: false });
    }
  },
  show: async (file) => {
    const { detail } = get();
    const instance = detail?.spec.instances[detail.instanceIndex];
    if (!detail || !instance) return;
    set({ file, content: null, doc: null, outline: [], collapsed: EMPTY, active: null });
    try {
      const content = await commands.readFile(instance.id, file);
      if (get().file !== file) return;
      const doc = content.kind === "markdown" && content.text !== null ? buildDoc(toHast(content.text)) : null;
      set({ content, doc, outline: doc ? outline(doc) : [] });
    } catch (e) {
      set({ error: asCmdError(e).message });
    }
  },
  toggle: (id) =>
    set(({ collapsed }) => {
      const next = new Set(collapsed);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return { collapsed: next };
    }),
  setActive: (active) => set({ active }),
  navigate: (id) => {
    const { doc, collapsed, scrollRequest } = get();
    const path = doc ? ancestry(doc, id) : [];
    const next = new Set(collapsed);
    for (const p of path) next.delete(p);
    set({ collapsed: next, active: id, scrollRequest: scrollRequest + 1 });
  },
}));
