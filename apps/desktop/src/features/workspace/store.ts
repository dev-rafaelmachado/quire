import { create } from "zustand";
import { asCmdError, commands, events, type ConfigView, type ProjectSummary, type ScanProgress } from "../../lib/ipc";
import type { StatusFilter } from "./model";

interface WorkspaceState {
  projects: ProjectSummary[];
  /** The first list (from the cache) arrived. */
  loaded: boolean;
  error: string | null;
  config: ConfigView | null;
  scan: ScanProgress | null;
  text: string;
  status: StatusFilter;
  collapsed: Readonly<Record<number, true>>;
  selectedSpecId: number | null;
  setText: (text: string) => void;
  setStatus: (status: StatusFilter) => void;
  toggleProject: (id: number) => void;
  select: (specId: number | null) => void;
  /** Lists from the cache (fast, IDX-03), then scans in the background (DSC-04). */
  init: () => Promise<void>;
  reload: () => Promise<void>;
  rescan: () => Promise<void>;
}

let listening = false;
let reloadTimer: ReturnType<typeof setTimeout> | null = null;

export const useWorkspace = create<WorkspaceState>((set, get) => ({
  projects: [],
  loaded: false,
  error: null,
  config: null,
  scan: null,
  text: "",
  status: "all",
  collapsed: {},
  selectedSpecId: null,
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
  init: async () => {
    if (!listening) {
      listening = true;
      await events.scanProgress.listen((e) => set({ scan: e.payload }));
      // Batches of the reconciliation arrive quickly; reload at most every 150 ms.
      await events.specsChanged.listen(() => {
        if (reloadTimer) return;
        reloadTimer = setTimeout(() => {
          reloadTimer = null;
          void get().reload();
        }, 150);
      });
    }
    try {
      set({ config: await commands.getConfig() });
    } catch (e) {
      set({ error: asCmdError(e).message });
    }
    await get().reload();
    await get().rescan();
  },
}));
