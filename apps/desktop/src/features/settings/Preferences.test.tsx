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
const { setLocale } = await import("../../lib/i18n");
const { ptBR } = await import("../../lib/i18n/pt-BR");
const { en } = await import("../../lib/i18n/en");

/** pt-BR strings that differ from English and are long enough to be recognized in a page. */
function portugueseOnly(): string[] {
  const out: string[] = [];
  const walk = (pt: unknown, english: unknown) => {
    if (typeof pt === "string") {
      const fixed = pt.split(/\{\w+\}/).map((p) => p.trim()).filter((p) => p.length >= 6);
      if (pt !== english) out.push(...fixed.filter((p) => typeof english !== "string" || !english.includes(p)));
      return;
    }
    for (const [k, v] of Object.entries(pt as Record<string, unknown>)) walk(v, (english as Record<string, unknown>)[k]);
  };
  walk(ptBR, en);
  return out;
}

const config: ConfigView = {
  path: "/home/dev/.config/quire/config.toml",
  error: null,
  scanPaths: ["/ws/projects", "/missing"],
  pollIntervalMs: 3000,
  projects: [{ name: "acme_gateway", preset: "exban", alias: null, issueUrlBase: null }],
  presets: ["exban", "generic"],
  theme: null,
  language: null,
  scanPathStatus: [
    { path: "/ws/projects", error: null },
    { path: "/missing", error: { code: "path_not_found", message: "path does not exist: /missing" } },
  ],
  editable: true,
};

beforeEach(() => {
  setConfig.mockReset();
  useWorkspace.setState({ projects: [], config });
  useTheme.getState().setTheme("dark");
});
afterEach(() => {
  cleanup();
  setLocale("pt-BR");
});

describe("preferences", () => {
  it("shows each scan path with its status and the reason when invalid (DSC-05)", () => {
    render(<Preferences config={config} />);
    expect(screen.getByText("a pasta não existe")).toBeTruthy();
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
      language: null,
      projects: [{ name: "acme_gateway", preset: null, alias: "gateway", issueUrlBase: null }],
    });
    expect(useTheme.getState().theme).toBe("light");
  });

  it("is read-only when config.toml cannot be saved", () => {
    render(<Preferences config={{ ...config, editable: false, error: { code: "config_parse", message: "config.toml line 2, column 1: bad" } }} />);
    expect(screen.getByRole("alert").textContent).toContain("config.toml com erro de sintaxe");
    expect(screen.getByRole("alert").textContent).toContain("line 2");
    expect((screen.getByRole("button", { name: "Salvar" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("shows no Portuguese text when the language is English (DEB-01)", () => {
    setLocale("en");
    render(<Preferences config={{ ...config, editable: false, error: { code: "config_parse", message: "line 2" } }} />);
    const text = document.body.textContent ?? "";
    expect(text).toContain("Scan folders");
    expect(text).toContain("syntax error in config.toml");
    expect(portugueseOnly().length).toBeGreaterThan(100);
    expect(portugueseOnly().filter((p) => text.includes(p))).toEqual([]);
  });

  it("switches the language on save without a restart, and System removes it (DEB-10)", async () => {
    setConfig.mockImplementation(async (s: { language: string | null }) => ({ ...config, language: s.language }));
    render(<Preferences config={config} />);
    fireEvent.click(within(screen.getByRole("group", { name: "Idioma" })).getByRole("button", { name: "English" }));
    fireEvent.click(screen.getByRole("button", { name: "Salvar" }));
    expect(await screen.findByRole("button", { name: "Save" })).toBeTruthy();
    expect(setConfig.mock.calls[0]?.[0]).toMatchObject({ language: "en" });
    fireEvent.click(within(screen.getByRole("group", { name: "Language" })).getByRole("button", { name: "System" }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("button", { name: "Salvar" })).toBeTruthy();
    expect(setConfig.mock.calls[1]?.[0]).toMatchObject({ language: null });
  });

  it("keeps an unknown configured language untouched and warns (DEB-10)", () => {
    setConfig.mockImplementation(async (s: unknown) => ({ ...config, ...(s as object) }));
    render(<Preferences config={{ ...config, language: "fr" }} />);
    expect(screen.getByText("Idioma desconhecido no config.toml: fr. Usando o do sistema.")).toBeTruthy();
    const system = within(screen.getByRole("group", { name: "Idioma" })).getByRole("button", { name: "Sistema" });
    expect(system.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "Salvar" }));
    expect(setConfig.mock.calls[0]?.[0]).toMatchObject({ language: "fr" });
  });
});
