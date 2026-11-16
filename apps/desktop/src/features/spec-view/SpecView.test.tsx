// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FileContent, SpecDetail } from "../../lib/ipc";

const openExternal = vi.fn(async (_url: string) => null);
vi.mock("../../lib/ipc", () => ({ commands: { openExternal: (url: string) => openExternal(url) } }));
vi.mock("../../lib/markdown/highlight", () => ({ highlight: async () => null, canHighlight: () => true }));

const { Tabs } = await import("./Tabs");
const { MetaStrip } = await import("./MetaStrip");
const { FileView } = await import("./FileView");
const { InstanceBadge } = await import("./InstanceBadge");
const { OTHERS, useSpecView } = await import("./store");

function detail(over: Partial<SpecDetail> = {}): SpecDetail {
  return {
    project: { id: 1, name: "acme_gateway", displayName: "acme_gateway", preset: "exban", issueUrlBase: "https://tracker.example/browse/" },
    spec: {
      id: 1,
      folder: "acme-170-rate-limit",
      issueKey: "ACME-170",
      instances: [
        {
          id: 9,
          path: "/ext/acme-170/acme_gateway/.specs/features/acme-170-rate-limit",
          repoRoot: "/ext/acme-170/acme_gateway",
          branch: "feat/acme-170-rate-limit",
          kind: "worktree",
          mtimeMs: 1,
          sameAsDefault: true,
    contentHash: "h",
          title: null,
          status: null,
          issueUrl: null,
          progress: [2, 3],
        },
      ],
      defaultIndex: 0,
      versionsBadge: null,
    },
    instanceIndex: 0,
    files: [],
    tabs: ["spec.md", "design.md", "tasks.md"],
    others: [],
    header: [
      { key: "Status", value: "Em andamento · 1 de 2 fases", text: "Em andamento · 1 de 2 fases", url: null, line: 3 },
      { key: "Idioma", value: "pt-BR", text: "pt-BR", url: null, line: 4 },
    ],
    headerFiles: ["spec.md", "summary.md"],
    ...over,
  };
}

const file = (over: Partial<FileContent>): FileContent => ({ name: "x", kind: "text", size: 10, text: "", large: false, ...over });

afterEach(cleanup);

describe("spec view", () => {
  it("shows only the tabs of files that exist, with the task count (VIW-01)", () => {
    const onSelect = vi.fn();
    render(<Tabs detail={detail()} counts={{}} tab="spec.md" onSelect={onSelect} />);
    const tabs = screen.getAllByRole("tab").map((t) => t.textContent);
    expect(tabs).toEqual(["spec", "design", "tasks2/3"]);
    fireEvent.click(screen.getByRole("tab", { name: /design/ }));
    expect(onSelect).toHaveBeenCalledWith("design.md");
  });

  it("adds an 'outros' tab with the count when there are other files", () => {
    const onSelect = vi.fn();
    render(<Tabs detail={detail({ others: ["notes.md", "payload.json"] })} counts={{ gates: 5 }} tab={null} onSelect={onSelect} />);
    fireEvent.click(screen.getByRole("tab", { name: "Outros arquivos (2)" }));
    expect(onSelect).toHaveBeenCalledWith(OTHERS);
  });

  it("renders header chips and opens the issue from the tracker base (VIW-07)", () => {
    render(<MetaStrip detail={detail()} />);
    expect(screen.getByText("Em andamento")).toBeTruthy();
    expect(screen.getByText("1 de 2 fases")).toBeTruthy();
    expect(screen.getByText("Idioma: pt-BR")).toBeTruthy();
    fireEvent.click(screen.getByRole("link", { name: "Abrir ACME-170 no navegador" }));
    expect(openExternal).toHaveBeenCalledWith("https://tracker.example/browse/ACME-170");
  });

  it("shows a notice for binaries and a warning for large files (VIW-08)", () => {
    const { rerender } = render(<FileView content={file({ name: "img.png", kind: "binary", text: null })} doc={null} collapsed={new Set()} onToggle={() => {}} />);
    expect(screen.getByText(/img\.png é binário/)).toBeTruthy();
    rerender(<FileView content={file({ kind: "json", text: "{}", size: 3 * 1024 * 1024, large: true })} doc={null} collapsed={new Set()} onToggle={() => {}} />);
    expect(screen.getByRole("status").textContent).toContain("3,0 MB");
  });

  it("shows branch and worktree of the displayed instance (DUP-06)", () => {
    useSpecView.setState({ detail: detail() });
    render(<InstanceBadge />);
    expect(screen.getByText("feat/acme-170-rate-limit")).toBeTruthy();
    expect(screen.getByText("acme-170/acme_gateway")).toBeTruthy();
  });
});
