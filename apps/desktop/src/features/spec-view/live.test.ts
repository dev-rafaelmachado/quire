import { describe, expect, it } from "vitest";
import type { InstanceSummary, ProjectSummary, SpecDetail } from "../../lib/ipc";
import { affects, reconcileSelection } from "./live";

const inst = (id: number, path: string): InstanceSummary => ({
  id,
  path,
  repoRoot: path.split("/.specs")[0] ?? path,
  branch: "main",
  kind: "checkout",
  mtimeMs: 1,
  sameAsDefault: true,
  contentHash: "h",
  title: null,
  status: null,
  issueUrl: null,
  progress: null,
});

const a = inst(1, "/ws/a/.specs/features/x");
const b = inst(2, "/ws/b/.specs/features/x");
const spec = (instances: InstanceSummary[]) => ({ id: 5, folder: "x", issueKey: null, instances, defaultIndex: 0, versionsBadge: null });
const projects = (instances: InstanceSummary[]): ProjectSummary[] => [
  { id: 1, key: "/ws/a/.git", name: "p", alias: null, preset: "exban", suggestedPreset: null, gitWarning: null, specs: instances.length ? [spec(instances)] : [] },
];
const detail = (shown: InstanceSummary): SpecDetail => ({
  project: { id: 1, name: "p", displayName: "p", preset: "exban", issueUrlBase: null },
  spec: spec([shown]),
  instanceIndex: 0,
  files: [],
  tabs: [],
  others: [],
  header: [],
  headerFiles: [],
});

describe("live updates", () => {
  it("knows when a change touches the instance on screen", () => {
    expect(affects(["/ws/a/.specs/features/x"], detail(a))).toBe(true);
    expect(affects(["/ws/a/.specs/features/x/"], detail(a))).toBe(true);
    expect(affects(["/ws/b/.specs/features/x"], detail(a))).toBe(false);
    expect(affects(["/ws/a/.specs/features/x"], null)).toBe(false);
    // Windows: the event may carry the verbatim prefix and `\` where the index does not.
    const w = inst(3, "C:\\ws\\a\\.specs\\features\\x");
    expect(affects(["\\\\?\\C:\\ws\\a\\.specs\\features\\x"], detail(w))).toBe(true);
    expect(affects(["C:/ws/a/.specs/features/x"], detail(w))).toBe(true);
    expect(affects(["\\\\?\\C:\\ws\\b\\.specs\\features\\x"], detail(w))).toBe(false);
  });

  it("falls back to another instance, or reports the spec gone (WAT-04)", () => {
    expect(reconcileSelection(projects([a, b]), 5, detail(a))).toEqual({ kind: "none" });
    expect(reconcileSelection(projects([b]), 5, detail(a))).toEqual({ kind: "switch", to: b });
    expect(reconcileSelection(projects([]), 5, detail(a))).toEqual({ kind: "removed" });
    expect(reconcileSelection(projects([a]), null, null)).toEqual({ kind: "none" });
  });
});
