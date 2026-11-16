// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ProjectSummary, SpecSummary } from "../../lib/ipc";

vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => ({ code: "x", message: String(e) }),
  commands: { getSpec: async () => null, takeCliOpen: async () => null },
  events: {},
}));

const { applyCliOpen } = await import("./cli");
const { CliPicker } = await import("./CliPicker");
const { Sidebar } = await import("./Sidebar");
const { useWorkspace } = await import("./store");
const { useSpecView } = await import("../spec-view/store");

const spec = (id: number, folder: string, key: string): SpecSummary => ({
  id,
  folder,
  issueKey: key,
  instances: [
    {
      id: id * 10,
      path: `/ws/p/.specs/features/${folder}`,
      repoRoot: "/ws/p",
      branch: "main",
      kind: "checkout",
      mtimeMs: 1,
      sameAsDefault: true,
      contentHash: "h",
      title: `${key} — Título ${id}`,
      status: null,
      issueUrl: null,
      progress: null,
    },
  ],
  defaultIndex: 0,
  versionsBadge: null,
});
const project = (id: number, name: string, specs: SpecSummary[]): ProjectSummary => ({
  id,
  key: `/ws/${name}/.git`,
  name,
  alias: null,
  preset: "exban",
  suggestedPreset: null,
  gitWarning: null,
  specs,
});

beforeEach(() => {
  useWorkspace.setState({
    projects: [project(1, "acme_gateway", [spec(1, "acme-200-auth", "ACME-200"), spec(2, "acme-142-x", "ACME-142")]), project(2, "acme_billing", [spec(3, "acme-200-auth", "ACME-200")])],
    loaded: true,
    selectedSpecId: null,
    highlight: null,
    pick: null,
    text: "",
    status: "all",
    collapsed: {},
    config: null,
    scan: null,
  });
  useSpecView.setState({ chosen: {}, notice: null });
});
afterEach(cleanup);

describe("command line navigation", () => {
  it("opens a spec in the requested instance (CLI-03)", () => {
    applyCliOpen({ kind: "spec", specId: 2, instanceId: 20, file: "design.md" });
    expect(useWorkspace.getState().selectedSpecId).toBe(2);
    expect(useSpecView.getState().chosen).toEqual({ 2: 20 });
    expect(useSpecView.getState().requestedTab).toBe("design.md");
  });

  it("highlights the specs of the worktree (CLI-01)", () => {
    applyCliOpen({ kind: "highlight", query: "/ws/p", specIds: [1, 2] });
    render(<Sidebar />);
    expect(screen.getByRole("status").textContent).toContain("2 specs nesta worktree");
    fireEvent.click(screen.getByRole("button", { name: "Limpar destaque" }));
    expect(useWorkspace.getState().highlight).toBeNull();
  });

  it("asks which spec when a key is in two projects (CLI-02)", () => {
    applyCliOpen({ kind: "pick", query: "acme-200", specIds: [1, 3] });
    render(<CliPicker />);
    expect(screen.getByRole("dialog", { name: "Mais de uma spec para acme-200" })).toBeTruthy();
    expect(screen.getByText("acme_billing")).toBeTruthy();
    fireEvent.click(screen.getByText("Título 3"));
    expect(useWorkspace.getState().selectedSpecId).toBe(3);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("says when nothing matches", () => {
    applyCliOpen({ kind: "notFound", query: "NOPE-1" });
    expect(useSpecView.getState().notice).toBe("Nenhuma spec encontrada para NOPE-1.");
  });
});
