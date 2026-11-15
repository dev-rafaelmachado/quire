// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { TasksView } from "../../lib/ipc";
import { TaskDetailPanel } from "./TaskDetailPanel";
import { TasksPanel } from "./TasksPanel";
import { useTasks } from "./store";
import { defaultExpanded, nextTaskId } from "./model";

const task = (id: string, done: boolean, hash: string | null = null, requirements: string[] = []) => ({
  id,
  title: `Tarefa ${id}`,
  done,
  hash,
  note: null,
  requirements,
  line: 1,
});

const view: TasksView = {
  structured: true,
  done: 3,
  total: 5,
  phases: [
    { number: 1, title: "Fundações", repo: null, diagram: [["T1", "T2"]], tasks: [task("T1", true, "a1b2c3d"), task("T2", true)], line: 1, done: 2, total: 2, state: "complete" },
    { number: 2, title: "Emissão", repo: null, diagram: [["T3", "T4"], ["T3", "T5"]], tasks: [task("T3", true, "e5f6a7b", ["TOK-01"]), task("T4", false, null, ["TOK-03"]), task("T5", false)], line: 5, done: 1, total: 3, state: "inProgress" },
  ],
  details: [
    {
      id: "T4",
      title: "Revogação",
      line: 20,
      fields: [
        { name: "Onde", text: "`src/tokens/api.ts`", items: [] },
        { name: "Depende de", text: "T3", items: [] },
        { name: "Requirement", text: "TOK-03", items: [] },
        { name: "Pronto quando", text: "", items: ["Falha em até 5 s", "Cache invalidado"] },
        { name: "Testes", text: "integration", items: [] },
        { name: "Gate", text: "fase", items: [] },
      ],
    },
  ],
};

beforeEach(() => {
  useTasks.setState({ view, gates: [], selected: nextTaskId(view.phases), expanded: defaultExpanded(view.phases), filter: "all" });
});
afterEach(cleanup);

describe("tasks view", () => {
  it("lists phases with count, bar and state (TSK-01)", () => {
    render(<TasksPanel view={view} />);
    expect(screen.getByRole("button", { name: "Expandir Fase 1: Fundações" })).toBeTruthy();
    expect(screen.getByText("completa")).toBeTruthy();
    expect(screen.getByText("em andamento")).toBeTruthy();
    expect(screen.getByRole("progressbar", { name: "Progresso: 1 de 3 tarefas" })).toBeTruthy();
  });

  it("shows the dependency chains and task rows with ids, hash and requirements (TSK-02, TSK-03)", () => {
    render(<TasksPanel view={view} />);
    const diagram = screen.getByRole("group", { name: "Dependências da fase" });
    expect(within(diagram).getAllByText("T3")).toHaveLength(2);
    expect(screen.getByText("e5f6a7b")).toBeTruthy();
    expect(screen.getByText("TOK-03")).toBeTruthy();
    expect(screen.getByRole("img", { name: "próxima" })).toBeTruthy();
  });

  it("selects a task and shows its detail; everything is read-only (TSK-04, TSK-05)", () => {
    render(
      <>
        <TasksPanel view={view} />
        <TaskDetailPanel />
      </>,
    );
    const detail = screen.getByRole("region", { name: "Detalhe da tarefa" });
    expect(within(detail).getByText("src/tokens/api.ts")).toBeTruthy();
    expect(within(detail).getByText("integration")).toBeTruthy();
    const boxes = within(detail).getAllByRole("checkbox") as HTMLInputElement[];
    expect(boxes).toHaveLength(2);
    expect(boxes.every((b) => b.disabled)).toBe(true);
    expect(within(detail).getByText(/Quem marca é o agente/)).toBeTruthy();
    fireEvent.click(screen.getByText("Tarefa T5"));
    expect(useTasks.getState().selected).toBe("T5");
    expect(screen.getByText("Sem detalhamento para T5 no tasks.md.")).toBeTruthy();
  });

  it("filters pending tasks", () => {
    render(<TasksPanel view={view} />);
    fireEvent.click(screen.getByRole("button", { name: "Pendentes" }));
    expect(screen.queryByText("Tarefa T3")).toBeNull();
    expect(screen.getByText("Tarefa T4")).toBeTruthy();
  });
});
