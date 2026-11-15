import { create } from "zustand";
import { asCmdError, commands, events, type ConfigView, type ProjectSummary, type ScanProgress, type WatchStatus } from "../../lib/ipc";
import type { StatusFilter } from "./model";

interface WorkspaceState {
  projects: ProjectSummary[];
  /** The first list (from the cache) arrived. */
  loaded: boolean;
  error: string | null;
  config: ConfigView | null;
  scan: ScanProgress | null;
  /** File watcher state for the status bar (WAT-03). */
  watch: WatchStatus | null;
  text: string;
  status: StatusFilter;
  collapsed: Readonly<Record<number, true>>;
  selectedSpecId: number | null;
  /** Specs of the worktree `quire .` ran in (CLI-01). */
  highlight: { query: string; specIds: number[] } | null;
  /** Specs a command-line key matched in several places (CLI-02). */
  pick: { query: string; specIds: number[] } | null;
  setHighlight: (highlight: { query: string; specIds: number[] } | null) => void;
  setPick: (pick: { query: string; specIds: number[] } | null) => void;
  setText: (text: string) => void;
  setStatus: (status: StatusFilter) => void;
  toggleProject: (id: number) => void;
  select: (specId: number | null) => void;
  /** Lists from the cache (fast, IDX-03), then scans in the background (DSC-04). */
  init: () => Promise<void>;
  reload: () => Promise<void>;
  rescan: () => Promise<void>;
}

// One init per app run: React StrictMode runs effects twice in development.
let initRun: Promise<void> | null = null;
// `?open=<folder>` selects that spec once it is listed (screenshots and demos, until the CLI).
let pendingOpen: string | null =
  typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("open");
let reloadTimer: ReturnType<typeof setTimeout> | null = null;

export const useWorkspace = create<WorkspaceState>((set, get) => ({
  projects: [],
  loaded: false,
  error: null,
  config: null,
  scan: null,
  watch: null,
  text: "",
  status: "all",
  collapsed: {},
  selectedSpecId: null,
  highlight: null,
  pick: null,
  setHighlight: (highlight) => set({ highlight }),
  setPick: (pick) => set({ pick }),
  setText: (text) => set({ text }),
  setStatus: (status) => set({ status }),
  toggleProject: (id) =>
    set(({ collapsed }) => {
      const next = { ...collapsed };
      if (next[id]) delete next[id];
      else next[id] = true;
      return { collapsed: next };
    }),
  select: (selectedSpecId) => set({ selectedSpecId }),
  reload: async () => {
    try {
      const projects = await commands.listProjects(null);
      set({ projects, loaded: true, error: null });
      if (pendingOpen !== null) {
        const spec = projects.flatMap((p) => p.specs).find((s) => s.folder === pendingOpen);
        if (spec) {
          pendingOpen = null;
          set({ selectedSpecId: spec.id });
        }
      }
    } catch (e) {
      set({ loaded: true, error: asCmdError(e).message });
    }
  },
  rescan: async () => {
    try {
      await commands.startScan();
    } catch (e) {
      set({ error: asCmdError(e).message });
    }
  },
  init: () => {
    initRun ??= (async () => {
      await events.scanProgress.listen((e) => {
        set({ scan: e.payload });
        // The watcher follows the index; its status changes when a scan ends.
        if (e.payload.phase === "done") void commands.getWatchStatus().then((watch) => set({ watch }));
      });
      await events.watchStatus.listen((e) => set({ watch: e.payload }));
      // Batches of the reconciliation arrive quickly; reload at most every 150 ms.
      await events.specsChanged.listen(() => {
        if (reloadTimer) return;
        reloadTimer = setTimeout(() => {
          reloadTimer = null;
          void get().reload();
        }, 150);
      });
      try {
        set({ config: await commands.getConfig(), watch: await commands.getWatchStatus() });
      } catch (e) {
        set({ error: asCmdError(e).message });
      }
      await get().reload();
      await get().rescan();
    })();
    return initRun;
  },
}));
