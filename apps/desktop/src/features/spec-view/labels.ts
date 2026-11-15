import type { InstanceSummary, SpecDetail } from "../../lib/ipc";

/** Tab label: the file name without extension (`gates.jsonl` -> `gates`). */
export function tabLabel(file: string): string {
  const dot = file.lastIndexOf(".");
  return dot > 0 ? file.slice(0, dot) : file;
}

/** Worktree name shown next to the branch: `parent/repo` for worktrees, `repo` for checkouts. */
export function worktreeName(instance: InstanceSummary): string {
  const parts = instance.repoRoot.split(/[\\/]/).filter(Boolean);
  const last = parts.at(-1) ?? instance.repoRoot;
  return instance.kind === "worktree" && parts.length > 1 ? `${parts.at(-2) ?? ""}/${last}` : last;
}

/** Issue URL: the header link, else the project's tracker base + key (VIW-07). */
export function issueUrl(detail: SpecDetail): string | null {
  const field = detail.header.find((f) => f.key.toLowerCase() === "issue");
  if (field?.url) return field.url;
  const base = detail.project.issueUrlBase;
  const key = detail.spec.issueKey;
  return base && key ? `${base}${key}` : null;
}

export function displayed(detail: SpecDetail): InstanceSummary | undefined {
  return detail.spec.instances[detail.instanceIndex];
}

/** "há 3 min" / "há 2 dias" (DUP-05). */
export function ago(mtimeMs: number | null, now: number, tr: (key: "instances.agoNow" | "instances.agoMinutes" | "instances.agoHours" | "instances.agoDays" | "instances.unknownTime", p?: { n: number }) => string): string {
  if (mtimeMs === null) return tr("instances.unknownTime");
  const minutes = Math.floor(Math.max(0, now - mtimeMs) / 60000);
  if (minutes < 1) return tr("instances.agoNow");
  if (minutes < 60) return tr("instances.agoMinutes", { n: minutes });
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return tr("instances.agoHours", { n: hours });
  return tr("instances.agoDays", { n: Math.floor(hours / 24) });
}

/** Last three segments of a path, for the instance list. */
export function shortInstancePath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length <= 3 ? path : `…/${parts.slice(-3).join("/")}`;
}
