// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { InstanceSummary, ProjectSummary, SpecDetail } from "../../lib/ipc";

const openForEdit = vi.fn();
const getSpec = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => e,
  commands: {
    openForEdit: (...a: unknown[]) => openForEdit(...a),
    getSpec: (...a: unknown[]) => getSpec(...a),
    fileFingerprint: async () => null,
  },
  events: {},
}));

const { useEditor } = await import("./store");
const { EditBar } = await import("./EditBar");
const { useConflict } = await import("../conflict/store");
const { applyListChange } = await import("../spec-view/live");
const { useSpecView } = await import("../spec-view/store");
const { useWorkspace } = await import("../workspace/store");

const LF = { bom: false, eol: "lf" as const, mixed: false, finalNewline: true };
const inst = (id: number, branch: string, root: string): InstanceSummary => ({
  id, path: `${root}/.specs/features/acme-142`, repoRoot: root, branch, kind: "worktree", mtimeMs: 1,
  sameAsDefault: true, contentHash: "h", title: "ACME-142 — X", status: null, issueUrl: null, progress: null,
});
const a = inst(10, "main", "/ws/acme_gateway");
const b = inst(11, "feat/acme-142", "/ws/.worktrees/acme-142/acme_gateway");
const detail = (instances: InstanceSummary[], versionsBadge: number | null): SpecDetail =>
  ({ spec: { id: 5, folder: "acme-142", issueKey: "ACME-142", instances, defaultIndex: 0, versionsBadge }, instanceIndex: 0 }) as unknown as SpecDetail;
const project = (instances: InstanceSummary[]): ProjectSummary => ({
  id: 1, key: "k", name: "acme_gateway", alias: null, preset: "exban", suggestedPreset: null, gitWarning: null,
  specs: [{ id: 5, folder: "acme-142", issueKey: "ACME-142", instances, defaultIndex: 0, versionsBadge: null }],
});

function dirtyOn(instanceId: number) {
  useEditor.setState({
    mode: "edit",
    buffer: { instanceId, name: "spec.md", savedText: "disk\n", base: "5-a", format: LF, readOnly: false, locked: null },
    dirty: true,
    getText: () => "my edits\n",
    carry: null,
  });
}

beforeEach(() => {
  openForEdit.mockReset();
  getSpec.mockReset();
  useConflict.getState().clear();
  useSpecView.setState({ specId: 5, detail: detail([a, b], null), notice: null });
});
afterEach(cleanup);

describe("editing with several copies (INS-01, INS-02, INS-04)", () => {
  it("names the copy that receives the edits and warns about other versions", () => {
    dirtyOn(10);
    useSpecView.setState({ detail: detail([a, b], 2) });
    render(<EditBar />);
    expect(screen.getByText("Edições vão para main · ws/acme_gateway")).toBeTruthy();
    expect(screen.getByRole("note").textContent).toContain("Esta spec tem 2 cópias, e elas não são todas iguais");
  });

  it("the copy on screen disappears with unsaved changes: no silent switch, buffer kept", () => {
    dirtyOn(10);
    const select = vi.spyOn(useWorkspace.getState(), "select");
    const switchInstance = vi.spyOn(useSpecView.getState(), "switchInstance");
    applyListChange([project([b])]);
    expect(switchInstance).not.toHaveBeenCalled();
    expect(select).not.toHaveBeenCalled();
    expect(useConflict.getState().gone?.to?.id).toBe(11);
    expect(useEditor.getState()).toMatchObject({ dirty: true, buffer: { instanceId: 10 } });
  });

  it("without unsaved changes it falls back to the other copy, as in v0.1", () => {
    useEditor.setState({ dirty: false, buffer: null });
    const switchInstance = vi.spyOn(useSpecView.getState(), "switchInstance").mockResolvedValue(undefined);
    applyListChange([project([b])]);
    expect(switchInstance).toHaveBeenCalledWith(11);
    expect(useConflict.getState().gone).toBeNull();
  });

  it("carried edits land in the other copy as unsaved changes and open the conflict dialog", async () => {
    dirtyOn(10);
    useEditor.getState().carryTo(11);
    openForEdit.mockResolvedValue({ name: "spec.md", text: "other copy\n", format: LF, fingerprint: "11-b", locked: null, readOnly: false });
    await useEditor.getState().load(11, "spec.md");
    expect(useEditor.getState().buffer).toMatchObject({ instanceId: 11, savedText: "other copy\n", base: "11-b", initialText: "my edits\n" });
    expect(useEditor.getState().dirty).toBe(true);
    expect(useEditor.getState().carry).toBeNull();
    await vi.waitFor(() => expect(useConflict.getState().open).toBe(true));
  });
});
