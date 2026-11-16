import { create } from "zustand";
import { asCmdError, commands, type FileContent, type SpecDetail } from "../../lib/ipc";
import { stripHeader } from "../../lib/markdown/header";
import { ancestry, buildDoc, type Doc, outline, type OutlineEntry, toHast } from "../../lib/markdown/pipeline";
import { displayed } from "./labels";

/** Tab id of the "outros" tab; primary tabs use their file name. */
export const OTHERS = "\u0000others";

export interface TabCounts {
  gates?: number;
  lessons?: number;
}

interface SpecViewState {
  specId: number | null;
  detail: SpecDetail | null;
  tab: string | null;
  /** File shown in the center column. */
  file: string | null;
  content: FileContent | null;
  doc: Doc | null;
  outline: OutlineEntry[];
  counts: TabCounts;
  collapsed: ReadonlySet<string>;
  active: string | null;
  /** Bumped to ask the view to scroll to `active`. */
  scrollRequest: number;
  error: string | null;
  /** Instance chosen per spec; lasts for the session (DUP-04). */
  chosen: Readonly<Record<number, number>>;
  /** File to show when the next spec opens (`quire open <file>`). */
  requestedTab: string | null;
  /** Shows a file of the open spec: its tab, or the "outros" tab on that file. */
  showFile: (file: string) => Promise<void>;
  /** Message about a change made outside the app (WAT-04). */
  notice: string | null;
  setNotice: (notice: string | null) => void;
  open: (specId: number | null) => Promise<void>;
  /** Re-reads the open spec after a change on disk, keeping tab, collapsed sections and
   *  scroll position (WAT-02). */
  refresh: () => Promise<void>;
  /** Shows another instance of the open spec, keeping the current tab (DUP-04). */
  switchInstance: (instanceId: number) => Promise<void>;
  selectTab: (tab: string) => Promise<void>;
  show: (file: string) => Promise<void>;
  toggle: (id: string) => void;
  setActive: (id: string | null) => void;
  navigate: (id: string) => void;
}

const EMPTY: ReadonlySet<string> = new Set();
// `?tab=<file>` opens that tab of the first spec opened (screenshots and demos, with `?open=`).
let pendingTab: string | null =
  typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("tab");

async function loadCounts(detail: SpecDetail): Promise<TabCounts> {
  const instance = displayed(detail);
  if (!instance) return {};
  const counts: TabCounts = {};
  const jobs: Promise<void>[] = [];
  if (detail.tabs.includes("gates.jsonl")) {
    jobs.push(commands.getGates(instance.id).then((g) => void (counts.gates = g.counts.runs)));
  }
  if (detail.tabs.includes("lessons.jsonl")) {
    jobs.push(commands.getLessons(instance.id).then((l) => void (counts.lessons = l.lessons.length)));
  }
  await Promise.allSettled(jobs);
  return counts;
}

export const useSpecView = create<SpecViewState>((set, get) => ({
  specId: null,
  detail: null,
  tab: null,
  file: null,
  content: null,
  doc: null,
  outline: [],
  counts: {},
  collapsed: EMPTY,
  active: null,
  scrollRequest: 0,
  error: null,
  chosen: {},
  requestedTab: null,
  notice: null,
  setNotice: (notice) => set({ notice }),
  open: async (specId) => {
    set({ specId, detail: null, tab: null, file: null, content: null, doc: null, outline: [], counts: {}, error: null });
    if (specId === null) return;
    try {
      const detail = await commands.getSpec(specId, get().chosen[specId] ?? null).catch(() =>
        // The chosen copy may be gone (e.g. its worktree was removed): fall back to the default.
        commands.getSpec(specId, null),
      );
      if (get().specId !== specId) return;
      set({ detail });
      void loadCounts(detail).then((counts) => get().specId === specId && set({ counts }));
      const wanted = pendingTab ?? get().requestedTab;
      pendingTab = null;
      set({ requestedTab: null });
      if (wanted !== null && (detail.tabs.includes(wanted) || detail.others.includes(wanted))) await get().showFile(wanted);
      else await get().selectTab(detail.tabs[0] ?? OTHERS);
    } catch (e) {
      set({ error: asCmdError(e).message });
    }
  },
  refresh: async () => {
    const { specId, detail, file } = get();
    const instance = detail ? displayed(detail) : undefined;
    if (specId === null || !detail || !instance) return;
    try {
      const next = await commands.getSpec(specId, instance.id);
      if (get().specId !== specId) return;
      set({ detail: next });
      void loadCounts(next).then((counts) => get().specId === specId && set({ counts }));
      if (!file) return;
      if (!next.files.some((f) => f.name === file)) {
        set({ content: null, doc: null, outline: [], notice: null, error: null });
        await get().selectTab(next.tabs[0] ?? OTHERS);
        return;
      }
      const content = await commands.readFile(instance.id, file);
      if (get().file !== file || get().specId !== specId) return;
      let doc: Doc | null = null;
      if (content.kind === "markdown" && content.text !== null) {
        const text = next.headerFiles.includes(file) ? stripHeader(content.text) : content.text;
        doc = buildDoc(toHast(text));
      }
      // Collapsed sections and the active heading are kept: ids come from the heading text.
      set({ content, doc, outline: doc ? outline(doc) : [] });
    } catch {
      // The instance may be gone; the list reconciliation decides what to show (WAT-04).
    }
  },
  switchInstance: async (instanceId) => {
    const { specId, tab } = get();
    if (specId === null) return;
    try {
      const detail = await commands.getSpec(specId, instanceId);
      if (get().specId !== specId) return;
      set(({ chosen }) => ({ detail, chosen: { ...chosen, [specId]: instanceId } }));
      void loadCounts(detail).then((counts) => get().specId === specId && set({ counts }));
      const keep = tab !== null && (tab === OTHERS ? detail.others.length > 0 : detail.tabs.includes(tab));
      await get().selectTab(keep && tab !== null ? tab : (detail.tabs[0] ?? OTHERS));
    } catch (e) {
      set({ error: asCmdError(e).message });
    }
  },
  showFile: async (file) => {
    const { detail } = get();
    if (!detail) return;
    if (detail.tabs.includes(file)) {
      await get().selectTab(file);
    } else if (detail.others.includes(file)) {
      set({ tab: OTHERS });
      await get().show(file);
    }
  },
  selectTab: async (tab) => {
    const { detail } = get();
    if (!detail) return;
    set({ tab });
    const file = tab === OTHERS ? detail.others[0] : tab;
    if (file) await get().show(file);
    else set({ file: null, content: null, doc: null, outline: [] });
  },
  show: async (file) => {
    const { detail } = get();
    const instance = detail ? displayed(detail) : undefined;
    if (!detail || !instance) return;
    set({ file, content: null, doc: null, outline: [], collapsed: EMPTY, active: null, error: null });
    try {
      const content = await commands.readFile(instance.id, file);
      if (get().file !== file) return;
      let doc: Doc | null = null;
      if (content.kind === "markdown" && content.text !== null) {
        const text = detail.headerFiles.includes(file) ? stripHeader(content.text) : content.text;
        doc = buildDoc(toHast(text));
      }
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
