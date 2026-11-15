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
  open: (specId: number | null) => Promise<void>;
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
  open: async (specId) => {
    set({ specId, detail: null, tab: null, file: null, content: null, doc: null, outline: [], counts: {}, error: null });
    if (specId === null) return;
    try {
      const detail = await commands.getSpec(specId, null);
      if (get().specId !== specId) return;
      set({ detail });
      void loadCounts(detail).then((counts) => get().specId === specId && set({ counts }));
      const wanted = pendingTab !== null && detail.tabs.includes(pendingTab) ? pendingTab : null;
      pendingTab = null;
      await get().selectTab(wanted ?? detail.tabs[0] ?? OTHERS);
    } catch (e) {
      set({ error: asCmdError(e).message });
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
