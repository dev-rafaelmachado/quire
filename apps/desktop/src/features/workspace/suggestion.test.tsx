// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ConfigView, ProjectSummary } from "../../lib/ipc";

const setConfig = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => ({ code: "x", message: String(e) }),
  commands: { setConfig: (s: unknown) => setConfig(s) },
}));

const { Sidebar } = await import("./Sidebar");
const { settingsWithPreset, useWorkspace } = await import("./store");

const config: ConfigView = {
  path: "/cfg/config.toml",
  error: null,
  scanPaths: ["/ws"],
  pollIntervalMs: 3000,
  projects: [{ name: "acme_gateway", preset: "exban", alias: "gw", issueUrlBase: null }],
  presets: ["exban", "generic"],
  theme: "light",
  language: null,
  scanPathStatus: [],
  editable: true,
};
const billing: ProjectSummary = { id: 7, key: "/ws/b/.git", name: "acme_billing", alias: null, preset: "generic", suggestedPreset: "exban", gitWarning: null, specs: [] };

beforeEach(() => {
  setConfig.mockReset();
  useWorkspace.setState({ projects: [billing], config, dismissedSuggestions: {}, collapsed: {}, text: "", status: "all", scan: null, error: null });
});
afterEach(cleanup);

describe("preset suggestion (DEB-03)", () => {
  it("only changes the preset of that project", () => {
    expect(settingsWithPreset(config, "acme_billing", "exban")).toEqual({
      scanPaths: ["/ws"],
      pollIntervalMs: 3000,
      theme: "light",
      language: null,
      projects: [
        { name: "acme_gateway", preset: "exban", alias: "gw", issueUrlBase: null },
        { name: "acme_billing", preset: "exban", alias: null, issueUrlBase: null },
      ],
    });
  });

  it("shows the suggestion and writes nothing until the click", async () => {
    setConfig.mockImplementation(async (s: ConfigView) => ({ ...config, projects: s.projects }));
    render(<Sidebar />);
    expect(screen.getByRole("status").textContent).toContain("Estrutura do preset exban detectada");
    expect(setConfig).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Usar exban" }));
    await waitFor(() => expect(setConfig).toHaveBeenCalledTimes(1));
    expect(setConfig.mock.calls[0]?.[0]).toMatchObject({ projects: expect.arrayContaining([expect.objectContaining({ name: "acme_billing", preset: "exban" })]) });
  });

  it("dismissing hides it for the session without writing", () => {
    render(<Sidebar />);
    fireEvent.click(screen.getByRole("button", { name: "Dispensar sugestão" }));
    expect(screen.queryByText("Estrutura do preset exban detectada")).toBeNull();
    expect(setConfig).not.toHaveBeenCalled();
  });

  it("cannot be applied when config.toml is not editable", () => {
    useWorkspace.setState({ config: { ...config, editable: false } });
    render(<Sidebar />);
    expect((screen.getByRole("button", { name: "Usar exban" }) as HTMLButtonElement).disabled).toBe(true);
  });
});
