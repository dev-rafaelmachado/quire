import { create } from "zustand";
import { asCmdError, type CmdError, commands, type GateRun, type Task, type TasksView } from "../../lib/ipc";
import { defaultExpanded, nextTaskId } from "./model";

export type TaskFilter = "all" | "pending";

let onToggled: (() => void) | null = null;
/** Who follows a checkbox write (the spec view and the list, PRG-01). */
export function setToggledHandler(fn: (() => void) | null): void {
  onToggled = fn;
}

interface TasksState {
  instanceId: number | null;
  view: TasksView | null;
  gates: GateRun[];
  selected: string | null;
  expanded: ReadonlySet<string>;
  filter: TaskFilter;
  error: CmdError | null;
  /** Task being written right now (one at a time). */
  toggling: string | null;
  /** The tasks.md changed under the click: reloaded, nothing written (CHK-03). */
  changedNotice: boolean;
  /** Why the last checkbox write did not happen. */
  toggleError: CmdError | null;
  /** Checks or unchecks a task, only when the project opted in (CHK-02, CHK-03). */
  toggle: (task: Task) => Promise<void>;
  dismissChanged: () => void;
  load: (instanceId: number) => Promise<void>;
  /** Re-reads after a change on disk, keeping selection and expanded phases. */
  reload: () => Promise<void>;
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
  toggling: null,
  changedNotice: false,
  toggleError: null,
  toggle: async (task) => {
    const { instanceId, view, toggling } = get();
    if (instanceId === null || !view?.checkbox || view.locked || !view.fingerprint || toggling) return;
    set({ toggling: task.id, toggleError: null, changedNotice: false });
    try {
      const next = await commands.toggleTask(instanceId, task.id, task.line, task.done, view.fingerprint);
      if (get().instanceId === instanceId) set({ view: next, toggling: null });
      onToggled?.();
    } catch (e) {
      const error = asCmdError(e);
      set({ toggling: null });
      if (error.code === "toggle_moved" || error.code === "write_conflict") {
        // The file is not what the view showed: reload it and say so; nothing was written.
        await get().reload();
        set({ changedNotice: true });
      } else {
        set({ toggleError: error });
      }
    }
  },
  dismissChanged: () => set({ changedNotice: false, toggleError: null }),
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
      set({ error: asCmdError(e) });
    }
  },
  reload: async () => {
    const { instanceId, selected } = get();
    if (instanceId === null || !get().view) return;
    try {
      const [view, gates] = await Promise.all([
        commands.getTasks(instanceId),
        commands.getGates(instanceId).then((g) => g.runs).catch(() => [] as GateRun[]),
      ]);
      if (get().instanceId !== instanceId) return;
      const ids = new Set(view.phases.flatMap((p) => p.tasks.map((t) => t.id)));
      set({ view, gates, selected: selected !== null && ids.has(selected) ? selected : nextTaskId(view.phases) });
    } catch {
      // Keep what is on screen; the next change or list reload settles it.
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
