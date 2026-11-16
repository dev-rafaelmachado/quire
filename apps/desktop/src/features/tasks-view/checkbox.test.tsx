// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TasksView } from "../../lib/ipc";

const toggleTask = vi.fn();
const getTasks = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => e,
  commands: {
    toggleTask: (...a: unknown[]) => toggleTask(...a),
    getTasks: (...a: unknown[]) => getTasks(...a),
    getGates: async () => ({ runs: [] }),
  },
}));

const { TasksPanel } = await import("./TasksPanel");
const { useTasks } = await import("./store");
const { useEditor } = await import("../editor/store");

const task = (id: string, done: boolean, line: number) => ({ id, title: `Tarefa ${id}`, done, commits: [], note: null, requirements: [], line });
function view(over: Partial<TasksView> = {}): TasksView {
  const tasks = [task("T1", true, 5), task("T2", false, 6)];
  return {
    structured: true,
    done: 1,
    total: 2,
    fingerprint: "40-aa",
    checkbox: true,
    locked: null,
    details: [],
    phases: [{ number: 1, title: "Base", repo: null, diagram: [], tasks, line: 3, done: 1, total: 2, state: "inProgress" }],
    ...over,
  } as TasksView;
}

beforeEach(() => {
  toggleTask.mockReset();
  getTasks.mockReset();
  useEditor.setState({ dirty: false, buffer: null });
  useTasks.setState({ instanceId: 7, view: view(), selected: "T2", expanded: new Set(["f1"]), filter: "all", toggling: null, changedNotice: false, toggleError: null });
});
afterEach(cleanup);

function renderPanel() {
  render(<TasksPanel view={useTasks.getState().view as TasksView} />);
}

describe("tasks.md checkbox (CHK-01..05)", () => {
  it("off by default: no checkbox, read-only as in v0.1", () => {
    useTasks.setState({ view: view({ checkbox: false }) });
    renderPanel();
    expect(screen.queryAllByRole("checkbox")).toHaveLength(0);
    expect(screen.getByText("checkbox somente leitura")).toBeTruthy();
  });

  it("on: a real checkbox writes that task with the base of the view and updates progress", async () => {
    toggleTask.mockResolvedValue(view({ done: 2, phases: [{ ...view().phases[0], tasks: [task("T1", true, 5), task("T2", true, 6)], done: 2, state: "complete" } as TasksView["phases"][number]] }));
    renderPanel();
    const box = screen.getByRole("checkbox", { name: "Marcar T2: Tarefa T2" }) as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);
    await waitFor(() => expect(toggleTask).toHaveBeenCalledWith(7, "T2", 6, false, "40-aa"));
    await waitFor(() => expect(useTasks.getState().view?.done).toBe(2));
  });

  it("disabled while tasks.md has unsaved edits, with the reason (CHK-04)", () => {
    useEditor.setState({ dirty: true, buffer: { instanceId: 7, name: "tasks.md", savedText: "", base: null, format: null, readOnly: false, locked: null } });
    renderPanel();
    for (const box of screen.getAllByRole("checkbox")) expect((box as HTMLInputElement).disabled).toBe(true);
    expect(screen.getByText("o tasks.md tem alterações não salvas no editor")).toBeTruthy();
  });

  it("disabled on a file system without verified atomic writes (SAV-11)", () => {
    useTasks.setState({ view: view({ locked: { code: "write_unreliable_fs", message: "9p" } }) });
    renderPanel();
    for (const box of screen.getAllByRole("checkbox")) expect((box as HTMLInputElement).disabled).toBe(true);
    expect(screen.getByText(/escrita desativada neste sistema de arquivos/)).toBeTruthy();
  });

  it("a moved task or a changed file reloads the tab and writes nothing (CHK-03)", async () => {
    toggleTask.mockRejectedValue({ code: "toggle_moved", message: "moved" });
    getTasks.mockResolvedValue(view({ fingerprint: "41-bb" }));
    renderPanel();
    fireEvent.click(screen.getByRole("checkbox", { name: "Marcar T2: Tarefa T2" }));
    await waitFor(() => expect(useTasks.getState().changedNotice).toBe(true));
    expect(getTasks).toHaveBeenCalledWith(7);
    expect(useTasks.getState().view?.fingerprint).toBe("41-bb");
    expect(toggleTask).toHaveBeenCalledTimes(1);
  });
});
