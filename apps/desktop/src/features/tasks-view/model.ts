import type { PhaseView, Task, TaskDetail } from "../../lib/ipc";

export type TaskMark = "done" | "next" | "pending";

/** The next task: first pending task of the first phase that is not complete. */
export function nextTaskId(phases: readonly PhaseView[]): string | null {
  for (const p of phases) {
    const t = p.tasks.find((x) => !x.done);
    if (t) return t.id;
  }
  return null;
}

export function markOf(task: Pick<Task, "id" | "done">, next: string | null): TaskMark {
  if (task.done) return "done";
  return task.id === next ? "next" : "pending";
}

export function phaseKey(p: PhaseView, index: number): string {
  return p.number === null ? `i${index}` : `f${p.number}`;
}

/** Phases expanded by default: those in progress, or the first not complete (wireframe 2). */
export function defaultExpanded(phases: readonly PhaseView[]): Set<string> {
  const open = phases.flatMap((p, i) => (p.state === "inProgress" ? [phaseKey(p, i)] : []));
  if (open.length === 0) {
    const i = phases.findIndex((p) => p.state !== "complete");
    const p = phases[i];
    if (p) open.push(phaseKey(p, i));
  }
  return new Set(open);
}

/** Text of a detail field without surrounding backticks. */
export function fieldText(detail: TaskDetail | undefined, name: string): string | null {
  const f = detail?.fields.find((x) => x.name.toLowerCase() === name.toLowerCase());
  if (!f) return null;
  const text = f.text.replace(/`/g, "").trim();
  return text || null;
}

export function fieldItems(detail: TaskDetail | undefined, name: string): string[] {
  return detail?.fields.find((x) => x.name.toLowerCase() === name.toLowerCase())?.items ?? [];
}

/** Comma-separated values of a field (`unit, integration`). */
export function fieldList(detail: TaskDetail | undefined, name: string): string[] {
  const text = fieldText(detail, name);
  return text ? text.split(/\s*,\s*/).filter(Boolean) : [];
}

/** Task ids in "Depende de" (`nenhuma` -> []). */
export function dependsOn(detail: TaskDetail | undefined): string[] {
  return (fieldText(detail, "Depende de") ?? "").match(/\bT\d+\b/g) ?? [];
}
