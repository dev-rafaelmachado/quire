// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { InstanceSummary, ProjectSummary, SpecSummary } from "../../lib/ipc";
import { Sidebar } from "./Sidebar";
import { useWorkspace } from "./store";

function instance(id: number, over: Partial<InstanceSummary> = {}): InstanceSummary {
  return {
    id,
    path: `/ws/p/.specs/features/s${id}`,
    repoRoot: "/ws/p",
    branch: "main",
    kind: "checkout",
    mtimeMs: 1,
    sameAsDefault: true,
    contentHash: "h",
    title: null,
    status: null,
    issueUrl: null,
    progress: null,
    ...over,
  };
}

const s142: SpecSummary = {
  id: 1,
  folder: "acme-142-service-tokens",
  issueKey: "ACME-142",
  instances: [
    instance(1, { title: "ACME-142 — Tokens de serviço", progress: [7, 8] }),
    instance(2, { kind: "worktree", repoRoot: "/wt/a", progress: [8, 8], sameAsDefault: false }),
  ],
  defaultIndex: 0,
  versionsBadge: 2,
};
const s170: SpecSummary = {
  id: 2,
  folder: "acme-170-rate-limit",
  issueKey: "ACME-170",
  instances: [instance(3, { title: "ACME-170 — Limite de taxa", progress: [3, 3] })],
  defaultIndex: 0,
  versionsBadge: null,
};
const projects: ProjectSummary[] = [
  { id: 1, key: "/ws/p/.git", name: "acme_gateway", alias: null, preset: "exban", suggestedPreset: null, gitWarning: null, specs: [s142, s170] },
];

beforeEach(() => {
  useWorkspace.setState({
    projects,
    loaded: true,
    error: null,
    text: "",
    status: "all",
    collapsed: {},
    selectedSpecId: null,
    config: { path: null, error: null, scanPaths: ["/ws"], pollIntervalMs: 3000, projects: [], presets: [], theme: null, scanPathStatus: [], editable: true },
    scan: { phase: "done", found: 2, processed: 2, total: 2, errors: [{ path: "/missing", code: "path_not_found", message: "path does not exist: /missing" }], message: null },
  });
});
afterEach(cleanup);

describe("Sidebar", () => {
  it("lists projects and specs with state, progress and the versions badge", () => {
    render(<Sidebar />);
    const group = screen.getByRole("group", { name: "acme_gateway" });
    expect(within(group).getByText("Tokens de serviço")).toBeTruthy();
    expect(within(group).getByLabelText("2 versões")).toBeTruthy();
    expect(within(group).getByText("Em andamento")).toBeTruthy();
    expect(within(group).getByRole("progressbar", { name: "Progresso: 7 de 8 tarefas" })).toBeTruthy();
    expect(within(group).getByText("Concluído")).toBeTruthy();
  });

  it("filters by text and by status", () => {
    render(<Sidebar />);
    fireEvent.change(screen.getByLabelText("Filtrar specs"), { target: { value: "170" } });
    expect(screen.queryByText("Tokens de serviço")).toBeNull();
    expect(screen.getByText("Limite de taxa")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Filtrar specs"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Em andamento" }));
    expect(screen.getByText("Tokens de serviço")).toBeTruthy();
    expect(screen.queryByText("Limite de taxa")).toBeNull();
  });

  it("shows scan path errors with the reason (DSC-05)", () => {
    render(<Sidebar />);
    expect(screen.getByText("/missing: a pasta não existe")).toBeTruthy();
    expect(screen.getByText("1 projeto · 1 worktree extra · 2 specs")).toBeTruthy();
  });

  it("selects a spec and collapses a project", () => {
    render(<Sidebar />);
    fireEvent.click(screen.getByText("Limite de taxa"));
    expect(useWorkspace.getState().selectedSpecId).toBe(2);
    fireEvent.click(screen.getByRole("button", { name: "Recolher acme_gateway" }));
    expect(screen.queryByText("Limite de taxa")).toBeNull();
  });
});
