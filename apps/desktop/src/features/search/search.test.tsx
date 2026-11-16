// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { SearchResults } from "../../lib/ipc";

const search = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => e,
  commands: { search: (...a: unknown[]) => search(...a), getSpec: async () => null },
  events: {},
}));

const { snippetParts } = await import("./Snippet");
const { findTerms } = await import("./highlight");
const { useSearch } = await import("./store");
const { SearchPanel } = await import("./SearchPanel");
const { useWorkspace } = await import("../workspace/store");
const { useSpecView } = await import("../spec-view/store");
const { useEditor } = await import("../editor/store");

const RESULTS: SearchResults = {
  truncated: false,
  terms: ["integracoes"],
  groups: [
    {
      specId: 6,
      folder: "acme-142-service-tokens",
      issueKey: "ACME-142",
      title: "ACME-142 — Tokens de serviço",
      projectId: 1,
      project: "acme_gateway",
      hits: [{ instanceId: 12, file: "spec.md", heading: "Problema", line: 14, snippet: "Usam \u0002integrações\u0003 <b>não</b>" }],
    },
  ],
};

beforeEach(() => {
  search.mockReset();
  useSearch.setState({ open: true, text: "", projectId: null, file: null, inSpec: false, results: null, loading: false, error: null, terms: [] });
  useWorkspace.setState({ projects: [], selectedSpecId: null });
  useSpecView.setState({ specId: null, detail: null, chosen: {}, requestedTab: null, pendingLine: null });
  useEditor.setState({ dirty: false, buffer: null });
});
afterEach(cleanup);

describe("search (SRC-03..06)", () => {
  it("turns markers into <mark> and keeps any HTML of the file as text", () => {
    expect(snippetParts("a \u0002b\u0003 c")).toEqual([
      { text: "a ", mark: false },
      { text: "b", mark: true },
      { text: " c", mark: false },
    ]);
    search.mockResolvedValue(RESULTS);
    useSearch.setState({ results: RESULTS, text: "integracoes" });
    const { container } = render(<SearchPanel />);
    expect(container.querySelector("mark")?.textContent).toBe("integrações");
    expect(container.querySelector("b")).toBeNull();
    expect(screen.getByText(/<b>não<\/b>/)).toBeTruthy();
  });

  it("finds the terms in the rendered text ignoring case and accents", () => {
    expect(findTerms("Integrações e INTEGRACOES", ["integracoes"])).toEqual([
      [0, 11],
      [14, 25],
    ]);
    expect(findTerms("nada aqui", ["xyz"])).toEqual([]);
  });

  it("searches with the filters after a pause in typing", async () => {
    search.mockResolvedValue(RESULTS);
    useSpecView.setState({ specId: 6 });
    useSearch.setState({ inSpec: true, file: "spec.md" });
    useSearch.getState().setText("integra");
    await waitFor(() => expect(search).toHaveBeenCalledWith({ text: "integra", projectId: null, file: "spec.md", specId: 6 }));
    expect(useSearch.getState().results).toEqual(RESULTS);
  });

  it("opening a result opens that copy, that tab and that section", async () => {
    useSearch.setState({ results: RESULTS, text: "integracoes" });
    render(<SearchPanel />);
    fireEvent.click(screen.getByRole("button", { name: /Problema/ }));
    await waitFor(() => expect(useWorkspace.getState().selectedSpecId).toBe(6));
    expect(useSpecView.getState()).toMatchObject({ chosen: { 6: 12 }, requestedTab: "spec.md", pendingLine: 14 });
    expect(useSearch.getState()).toMatchObject({ open: false, terms: ["integracoes"] });
  });
});
