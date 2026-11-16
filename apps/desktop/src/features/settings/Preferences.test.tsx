// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ConfigView } from "../../lib/ipc";

const setConfig = vi.fn();
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => ({ code: "x", message: String(e) }),
  commands: { setConfig: (s: unknown) => setConfig(s) },
}));

const { Preferences } = await import("./Preferences");
const { useWorkspace } = await import("../workspace/store");
const { useTheme } = await import("../../app/theme");

const config: ConfigView = {
  path: "/home/dev/.config/quire/config.toml",
  error: null,
  scanPaths: ["/ws/projects", "/missing"],
  pollIntervalMs: 3000,
  projects: [{ name: "acme_gateway", preset: "exban", alias: null, issueUrlBase: null }],
  presets: ["exban", "generic"],
  theme: null,
  scanPathStatus: [
    { path: "/ws/projects", error: null },
    { path: "/missing", error: "path does not exist: /missing" },
  ],
  editable: true,
};

beforeEach(() => {
  setConfig.mockReset();
  useWorkspace.setState({ projects: [], config });
  useTheme.getState().setTheme("dark");
});
afterEach(cleanup);

describe("preferences", () => {
  it("shows each scan path with its status and the reason when invalid (DSC-05)", () => {
    render(<Preferences config={config} />);
    expect(screen.getByText("path does not exist: /missing")).toBeTruthy();
    expect(screen.getByText("ok")).toBeTruthy();
  });

  it("edits paths and project settings and saves them (PST-02)", async () => {
    setConfig.mockImplementation(async (s: { scanPaths: string[] }) => ({ ...config, scanPaths: s.scanPaths, theme: "light", scanPathStatus: [] }));
    render(<Preferences config={config} />);
    fireEvent.click(screen.getByRole("button", { name: "Remover /missing" }));
    fireEvent.change(screen.getByLabelText("Nova pasta de busca"), { target: { value: "~/code" } });
    fireEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(screen.getByText("ainda não salva")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Apelido de acme_gateway"), { target: { value: "gateway" } });
    fireEvent.change(screen.getByLabelText("Preset de acme_gateway"), { target: { value: "" } });
    fireEvent.click(within(screen.getByRole("group", { name: "Tema" })).getByRole("button", { name: "Claro" }));
    fireEvent.click(screen.getByRole("button", { name: "Salvar" }));
    await screen.findByText("Preferências salvas.");
    expect(setConfig).toHaveBeenCalledWith({
      scanPaths: ["/ws/projects", "~/code"],
      pollIntervalMs: 3000,
      theme: "light",
      projects: [{ name: "acme_gateway", preset: null, alias: "gateway", issueUrlBase: null }],
    });
    expect(useTheme.getState().theme).toBe("light");
  });

  it("is read-only when config.toml cannot be saved", () => {
    render(<Preferences config={{ ...config, editable: false, error: "config.toml line 2, column 1: bad" }} />);
    expect(screen.getByRole("alert").textContent).toContain("line 2");
    expect((screen.getByRole("button", { name: "Salvar" }) as HTMLButtonElement).disabled).toBe(true);
  });
});
