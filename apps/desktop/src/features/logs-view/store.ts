import { create } from "zustand";
import { asCmdError, type CmdError, commands, type GatesView, type LessonsView } from "../../lib/ipc";

export type LevelFilter = "all" | "fase" | "final" | "e2e";
export type StatusFilter = "all" | "pass" | "fail";

interface LogsState {
  instanceId: number | null;
  gates: GatesView | null;
  lessons: LessonsView | null;
  level: LevelFilter;
  status: StatusFilter;
  /** Index (in `gates.runs`) of expanded failed runs. */
  open: ReadonlySet<number>;
  error: CmdError | null;
  load: (instanceId: number, files: { gates: boolean; lessons: boolean }) => Promise<void>;
  /** Re-reads after a change on disk, keeping filters and expanded runs. */
  reload: () => Promise<void>;
  setLevel: (level: LevelFilter) => void;
  setStatus: (status: StatusFilter) => void;
  toggleRun: (index: number) => void;
}

export const useLogs = create<LogsState>((set, get) => ({
  instanceId: null,
  gates: null,
  lessons: null,
  level: "all",
  status: "all",
  open: new Set(),
  error: null,
  load: async (instanceId, files) => {
    if (get().instanceId === instanceId && (get().gates || get().lessons)) return;
    set({ instanceId, gates: null, lessons: null, open: new Set(), error: null });
    try {
      const [gates, lessons] = await Promise.all([
        files.gates ? commands.getGates(instanceId) : Promise.resolve(null),
        files.lessons ? commands.getLessons(instanceId) : Promise.resolve(null),
      ]);
      if (get().instanceId !== instanceId) return;
      // Failed runs start expanded, like wireframe 3 (LOG-02).
      const open = new Set((gates?.runs ?? []).flatMap((r, i) => (isFail(r) ? [i] : [])).slice(0, 1));
      set({ gates, lessons, open });
    } catch (e) {
      set({ error: asCmdError(e) });
    }
  },
  reload: async () => {
    const { instanceId, gates, lessons } = get();
    if (instanceId === null) return;
    try {
      const [g, l] = await Promise.all([
        gates ? commands.getGates(instanceId) : Promise.resolve(null),
        lessons ? commands.getLessons(instanceId) : Promise.resolve(null),
      ]);
      if (get().instanceId === instanceId) set({ gates: g, lessons: l });
    } catch {
      // Keep what is on screen.
    }
  },
  setLevel: (level) => set({ level }),
  setStatus: (status) => set({ status }),
  toggleRun: (index) =>
    set(({ open }) => {
      const next = new Set(open);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return { open: next };
    }),
}));

export function isFail(run: { status: string | null; exit: number | null }): boolean {
  return run.status ? run.status.toLowerCase() === "fail" : run.exit !== null && run.exit !== 0;
}
