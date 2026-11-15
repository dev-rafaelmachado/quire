import type { ProjectSummary, SpecSummary } from "../../lib/ipc";

export type SpecState = "complete" | "inProgress" | "pending" | "noTasks";
export type StatusFilter = "all" | "inProgress" | "done";

/** State shown in the sidebar, from the default instance's task progress (TSK-01 rule). */
export function specState(spec: SpecSummary): SpecState {
  const progress = spec.instances[spec.defaultIndex]?.progress;
  if (!progress) return "noTasks";
  const [done, total] = progress;
  if (total === 0) return "noTasks";
  if (done >= total) return "complete";
  return done > 0 ? "inProgress" : "pending";
}

/** Title from the default instance's `# …` heading, without a leading `KEY —`. */
export function specTitle(spec: SpecSummary): string {
  const raw = spec.instances[spec.defaultIndex]?.title ?? "";
  const key = spec.issueKey;
  const stripped = key ? raw.replace(new RegExp(`^${escapeRegExp(key)}\\s*[—–:-]\\s*`, "i"), "") : raw;
  return stripped.trim() || spec.folder;
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function matchesText(spec: SpecSummary, needle: string): boolean {
  const hay = [spec.folder, spec.issueKey ?? "", specTitle(spec)].join("\n").toLowerCase();
  return hay.includes(needle);
}

function matchesStatus(spec: SpecSummary, status: StatusFilter): boolean {
  if (status === "all") return true;
  const state = specState(spec);
  return status === "done" ? state === "complete" : state === "inProgress";
}

/** Applies the key/title text filter and the status filter; drops empty projects. */
export function filterProjects(projects: readonly ProjectSummary[], text: string, status: StatusFilter): ProjectSummary[] {
  const needle = text.trim().toLowerCase();
  return projects
    .map((p) => ({
      ...p,
      specs: p.specs.filter((s) => (!needle || matchesText(s, needle)) && matchesStatus(s, status)),
    }))
    .filter((p) => p.specs.length > 0 || (!needle && status === "all"));
}

export interface WorkspaceCounts {
  projects: number;
  specs: number;
  /** Linked worktrees found (in or outside the scan paths). */
  worktrees: number;
}

export function counts(projects: readonly ProjectSummary[]): WorkspaceCounts {
  const roots = new Set<string>();
  let specs = 0;
  for (const p of projects) {
    specs += p.specs.length;
    for (const s of p.specs) for (const i of s.instances) if (i.kind === "worktree") roots.add(i.repoRoot);
  }
  return { projects: projects.length, specs, worktrees: roots.size };
}
