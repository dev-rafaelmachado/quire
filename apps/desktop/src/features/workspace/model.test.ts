import { describe, expect, it } from "vitest";
import type { InstanceSummary, ProjectSummary, SpecSummary } from "../../lib/ipc";
import { counts, filterProjects, specState, specTitle } from "./model";

function instance(over: Partial<InstanceSummary> = {}): InstanceSummary {
  return {
    id: 1,
    path: "/ws/a/.specs/features/x",
    repoRoot: "/ws/a",
    branch: "main",
    kind: "checkout",
    mtimeMs: 1,
    sameAsDefault: true,
    title: null,
    status: null,
    issueUrl: null,
    progress: null,
    ...over,
  };
}

function spec(folder: string, over: Partial<InstanceSummary> = {}, key: string | null = null): SpecSummary {
  return { id: 1, folder, issueKey: key, instances: [instance(over)], defaultIndex: 0, versionsBadge: null };
}

function project(name: string, specs: SpecSummary[]): ProjectSummary {
  return { id: 1, key: `/ws/${name}/.git`, name, alias: null, preset: "exban", suggestedPreset: null, gitWarning: null, specs };
}

describe("workspace model", () => {
  it("derives the state from task progress", () => {
    expect(specState(spec("a", { progress: [3, 3] }))).toBe("complete");
    expect(specState(spec("a", { progress: [1, 3] }))).toBe("inProgress");
    expect(specState(spec("a", { progress: [0, 3] }))).toBe("pending");
    expect(specState(spec("a"))).toBe("noTasks");
  });

  it("strips the issue key from the title", () => {
    expect(specTitle(spec("acme-1-x", { title: "ACME-1 — Tokens" }, "ACME-1"))).toBe("Tokens");
    expect(specTitle(spec("loose", { title: null }))).toBe("loose");
  });

  it("filters by text and status and drops empty projects", () => {
    const list = [
      project("gw", [spec("acme-1-tokens", { progress: [2, 2] }, "ACME-1"), spec("acme-2-rate", { progress: [1, 3] }, "ACME-2")]),
      project("bill", [spec("bill-3-export", { progress: [0, 1] }, "BILL-3")]),
    ];
    expect(filterProjects(list, "acme-2", "all").map((p) => p.specs.map((s) => s.folder))).toEqual([["acme-2-rate"]]);
    expect(filterProjects(list, "", "done").map((p) => p.name)).toEqual(["gw"]);
    expect(filterProjects(list, "", "inProgress")[0]?.specs[0]?.folder).toBe("acme-2-rate");
    expect(filterProjects(list, "", "all")).toHaveLength(2);
  });

  it("counts projects, specs and worktrees", () => {
    const list = [
      project("gw", [
        { ...spec("a"), instances: [instance(), instance({ kind: "worktree", repoRoot: "/wt/1" })] },
        { ...spec("b"), instances: [instance({ kind: "worktree", repoRoot: "/wt/1" })] },
      ]),
    ];
    expect(counts(list)).toEqual({ projects: 1, specs: 2, worktrees: 1 });
  });
});
