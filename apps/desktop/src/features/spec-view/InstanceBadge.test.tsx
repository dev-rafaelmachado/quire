// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { InstanceSummary, SpecDetail } from "../../lib/ipc";

const getSpec = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => ({ code: "x", message: String(e) }),
  commands: {
    getSpec: (specId: number, instanceId: number | null) => getSpec(specId, instanceId),
    readFile: async (_: number, name: string) => ({ name, kind: "text", size: 1, text: "x", large: false }),
    getGates: async () => ({ runs: [], counts: { runs: 0, passes: 0, fails: 0 }, skipped: 0, skippedLines: [] }),
    getLessons: async () => ({ lessons: [], skipped: 0, skippedLines: [] }),
  },
}));

const { InstanceBadge } = await import("./InstanceBadge");
const { useSpecView } = await import("./store");

const now = Date.now();
const inst = (id: number, over: Partial<InstanceSummary>): InstanceSummary => ({
  id,
  path: `/ws/${id}/.specs/features/acme-170-rate-limit`,
  repoRoot: `/ws/${id}`,
  branch: "main",
  kind: "checkout",
  mtimeMs: now - 2 * 86400000,
  sameAsDefault: true,
  contentHash: "a",
  title: null,
  status: null,
  issueUrl: null,
  progress: null,
  ...over,
});

function detail(instanceIndex: number, instances: InstanceSummary[], badge: number | null): SpecDetail {
  return {
    project: { id: 1, name: "acme_gateway", displayName: "acme_gateway", preset: "exban", issueUrlBase: null },
    spec: { id: 7, folder: "acme-170-rate-limit", issueKey: "ACME-170", instances, defaultIndex: 0, versionsBadge: badge },
    instanceIndex,
    files: [],
    tabs: ["spec.md", "tasks.md"],
    others: [],
    header: [],
    headerFiles: [],
  };
}

const three = [
  inst(1, { branch: "feat/acme-170-rate-limit", kind: "worktree", repoRoot: "/ws/external/acme-170/acme_gateway", mtimeMs: now - 3 * 60000, contentHash: "b" }),
  inst(2, { repoRoot: "/ws/projects/acme_gateway" }),
  inst(3, { branch: "feat/acme-142", kind: "worktree", repoRoot: "/ws/projects/.worktrees/acme-142/acme_gateway" }),
];

afterEach(() => {
  cleanup();
  getSpec.mockReset();
});

describe("instance switcher", () => {
  it("shows the versions badge and lists every instance with branch, kind, path, age and state (DUP-05)", () => {
    useSpecView.setState({ detail: detail(0, three, 3), specId: 7 });
    render(<InstanceBadge />);
    const trigger = screen.getByRole("button", { name: "Trocar instância da spec (3 cópias)" });
    expect(within(trigger).getByText("3 versões")).toBeTruthy();
    fireEvent.click(trigger);
    const dialog = screen.getByRole("dialog", { name: "3 versões de acme-170-rate-limit" });
    const radios = within(dialog).getAllByRole("radio");
    expect(radios).toHaveLength(3);
    expect(within(dialog).getByText("aberta")).toBeTruthy();
    expect(within(dialog).getAllByText("diverge")).toHaveLength(2);
    expect(within(dialog).getByText("modificada há 3 min")).toBeTruthy();
    expect(within(dialog).getAllByText("modificada há 2 dias")).toHaveLength(2);
    expect(within(dialog).getByText("…/external/acme-170/acme_gateway")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("switches instance keeping the tab and remembers it for the session (DUP-04)", async () => {
    useSpecView.setState({ detail: detail(0, three, 3), specId: 7, tab: "tasks.md", chosen: {} });
    getSpec.mockImplementation(async (_s: number, i: number | null) => detail(i === 2 ? 1 : 0, three, 3));
    render(<InstanceBadge />);
    fireEvent.click(screen.getByRole("button", { name: /Trocar instância/ }));
    fireEvent.click(screen.getAllByRole("radio")[1] as HTMLElement);
    fireEvent.click(screen.getByRole("button", { name: "Abrir esta" }));
    await vi.waitFor(() => expect(useSpecView.getState().detail?.instanceIndex).toBe(1));
    expect(getSpec).toHaveBeenCalledWith(7, 2);
    expect(useSpecView.getState().tab).toBe("tasks.md");
    expect(useSpecView.getState().chosen).toEqual({ 7: 2 });
    await useSpecView.getState().open(7);
    expect(getSpec).toHaveBeenLastCalledWith(7, 2);
  });

  it("a spec with one instance has no badge and no switcher", () => {
    useSpecView.setState({ detail: detail(0, [inst(1, {})], null) });
    render(<InstanceBadge />);
    expect(screen.queryByRole("button")).toBeNull();
    expect(screen.queryByText(/versões/)).toBeNull();
    expect(screen.getByRole("group", { name: "Instância exibida" })).toBeTruthy();
  });
});
