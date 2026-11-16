import type { ProjectSummary, SpecSummary } from "../../lib/ipc";

/** The selected spec and its project, if still in the list. */
export function findSelected(
  projects: readonly ProjectSummary[],
  specId: number | null,
): { project: ProjectSummary; spec: SpecSummary } | null {
  if (specId === null) return null;
  for (const project of projects) {
    const spec = project.specs.find((s) => s.id === specId);
    if (spec) return { project, spec };
  }
  return null;
}
