import { devParam } from "../../lib/devParams";
import { create } from "zustand";
import { asCmdError, type CmdError, commands, type SearchGroup, type SearchHit, type SearchResults } from "../../lib/ipc";
import { useSpecView } from "../spec-view/store";
import { guard } from "../unsaved/store";
import { useWorkspace } from "../workspace/store";

/** Pause in typing before searching. */
export const SEARCH_DEBOUNCE_MS = 120;

interface SearchState {
  open: boolean;
  text: string;
  projectId: number | null;
  file: string | null;
  /** Only the spec open in the center column (SRC-05). */
  inSpec: boolean;
  results: SearchResults | null;
  loading: boolean;
  error: CmdError | null;
  /** Terms of the last opened result, highlighted in the file (SRC-06). */
  terms: string[];
  setOpen: (open: boolean) => void;
  setText: (text: string) => void;
  setFilters: (f: { projectId?: number | null; file?: string | null; inSpec?: boolean }) => void;
  run: () => Promise<void>;
  /** Opens the spec of a result in that copy, on that file, at that section (SRC-06). */
  openHit: (group: SearchGroup, hit: SearchHit) => void;
}

let timer: ReturnType<typeof setTimeout> | null = null;
// `?search=<text>` opens the panel with that text; `?highlight=<terms>` marks terms in the open
// file (screenshots and demos, like `?open=`).
const URL_SEARCH = devParam("search");
const URL_HIGHLIGHT = devParam("highlight")?.split(/\s+/).filter(Boolean) ?? [];
let seq = 0;

export const useSearch = create<SearchState>((set, get) => ({
  open: URL_SEARCH !== null,
  text: URL_SEARCH ?? "",
  projectId: null,
  file: null,
  inSpec: false,
  results: null,
  loading: false,
  error: null,
  terms: URL_HIGHLIGHT,
  setOpen: (open) => set({ open }),
  setText: (text) => {
    set({ text });
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      void get().run();
    }, SEARCH_DEBOUNCE_MS);
  },
  setFilters: (f) => {
    set(f);
    void get().run();
  },
  run: async () => {
    const { text, projectId, file, inSpec } = get();
    const id = ++seq;
    if (text.trim() === "") {
      set({ results: null, loading: false, error: null });
      return;
    }
    set({ loading: true, error: null });
    try {
      const specId = inSpec ? useSpecView.getState().specId : null;
      const results = await commands.search({ text, projectId, file, specId });
      if (id === seq) set({ results, loading: false });
    } catch (e) {
      if (id === seq) set({ loading: false, error: asCmdError(e) });
    }
  },
  openHit: (group, hit) => {
    const terms = get().results?.terms ?? [];
    guard(async () => {
      set({ open: false, terms });
      const ws = useWorkspace.getState();
      ws.setPrefsOpen(false);
      const view = useSpecView.getState();
      view.requestLine(hit.line);
      if (ws.selectedSpecId === group.specId && view.detail) {
        const current = view.detail.spec.instances[view.detail.instanceIndex]?.id;
        if (current !== hit.instanceId) await view.switchInstance(hit.instanceId);
        await useSpecView.getState().showFile(hit.file);
      } else {
        // The spec opens on that copy and that tab.
        useSpecView.setState(({ chosen }) => ({ chosen: { ...chosen, [group.specId]: hit.instanceId }, requestedTab: hit.file }));
        ws.select(group.specId);
      }
    });
  },
}));
