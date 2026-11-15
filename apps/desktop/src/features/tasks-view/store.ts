import { create } from "zustand";
import { asCmdError, commands, type GateRun, type TasksView } from "../../lib/ipc";
import { defaultExpanded, nextTaskId } from "./model";

export type TaskFilter = "all" | "pending";

interface TasksState {
  instanceId: number | null;
  view: TasksView | null;
  gates: GateRun[];
  selected: string | null;
  expanded: ReadonlySet<string>;
  filter: TaskFilter;
  error: string | null;
  load: (instanceId: number) => Promise<void>;
  select: (id: string) => void;
  togglePhase: (key: string) => void;
  setFilter: (filter: TaskFilter) => void;
}

export const useTasks = create<TasksState>((set, get) => ({
  instanceId: null,
  view: null,
  gates: [],
  selected: null,
  expanded: new Set(),
  filter: "all",
  error: null,
  load: async (instanceId) => {
    set({ instanceId, view: null, error: null });
    try {
      const [view, gates] = await Promise.all([
        commands.getTasks(instanceId),
        commands.getGates(instanceId).then((g) => g.runs).catch(() => [] as GateRun[]),
      ]);
      if (get().instanceId !== instanceId) return;
      set({ view, gates, selected: nextTaskId(view.phases), expanded: defaultExpanded(view.phases) });
    } catch (e) {
      set({ error: asCmdError(e).message });
    }
  },
  select: (selected) => set({ selected }),
  togglePhase: (key) =>
    set(({ expanded }) => {
      const next = new Set(expanded);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return { expanded: next };
    }),
  setFilter: (filter) => set({ filter }),
}));
