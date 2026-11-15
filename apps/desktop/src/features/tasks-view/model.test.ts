import { describe, expect, it } from "vitest";
import type { PhaseView, TaskDetail } from "../../lib/ipc";
import { defaultExpanded, dependsOn, fieldList, fieldText, markOf, nextTaskId } from "./model";

const task = (id: string, done: boolean) => ({ id, title: id, done, hash: null, note: null, requirements: [], line: 1 });
const phase = (number: number, tasks: ReturnType<typeof task>[], state: PhaseView["state"]): PhaseView => ({
  number,
  title: `F${number}`,
  repo: null,
  diagram: [],
  tasks,
  line: 1,
  done: tasks.filter((t) => t.done).length,
  total: tasks.length,
  state,
});

describe("tasks model", () => {
  const phases = [
    phase(1, [task("T1", true)], "complete"),
    phase(2, [task("T2", true), task("T3", false), task("T4", false)], "inProgress"),
    phase(3, [task("T5", false)], "pending"),
  ];

  it("finds the next task and marks tasks", () => {
    expect(nextTaskId(phases)).toBe("T3");
    expect(markOf(task("T3", false), "T3")).toBe("next");
    expect(markOf(task("T4", false), "T3")).toBe("pending");
    expect(markOf(task("T1", true), "T3")).toBe("done");
  });

  it("expands phases in progress by default", () => {
    expect([...defaultExpanded(phases)]).toEqual(["f2"]);
    expect([...defaultExpanded([phases[0] as PhaseView, phases[2] as PhaseView])]).toEqual(["f3"]);
  });

  it("reads detail fields", () => {
    const d: TaskDetail = {
      id: "T2",
      title: "x",
      line: 1,
      fields: [
        { name: "Onde", text: "`src/a.ts`, `src/b.ts`", items: [] },
        { name: "Depende de", text: "T1, T7", items: [] },
        { name: "Testes", text: "unit, integration", items: [] },
      ],
    };
    expect(fieldText(d, "onde")).toBe("src/a.ts, src/b.ts");
    expect(dependsOn(d)).toEqual(["T1", "T7"]);
    expect(fieldList(d, "Testes")).toEqual(["unit", "integration"]);
    expect(dependsOn({ ...d, fields: [{ name: "Depende de", text: "nenhuma", items: [] }] })).toEqual([]);
  });
});
