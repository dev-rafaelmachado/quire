// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { formatWhen } from "../../features/logs-view/format";
import { errorText } from "./errors";
import { en } from "./en";
import { formatNumber, setLocale, t } from "./index";
import { ptBR } from "./pt-BR";

type Tree = { readonly [k: string]: unknown };

/** Every leaf as `path -> sorted {params}`; plurals contribute `.one` and `.other`. */
function leaves(node: unknown, prefix = ""): Map<string, string> {
  const out = new Map<string, string>();
  if (typeof node === "string") {
    out.set(prefix, [...node.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort().join(","));
    return out;
  }
  for (const [k, v] of Object.entries(node as Tree)) {
    for (const [p, params] of leaves(v, prefix ? `${prefix}.${k}` : k)) out.set(p, params);
  }
  return out;
}

afterEach(() => setLocale("pt-BR"));

describe("t()", () => {
  it("returns catalog strings", () => {
    expect(t("sidebar.title")).toBe("Projetos");
  });

  it("interpolates and picks plural forms", () => {
    expect(t("sidebar.versions", { n: 1 })).toBe("1 versão");
    expect(t("sidebar.versions", { n: 3 })).toBe("3 versões");
    expect(t("scan.scanning", { path: "~/projects" })).toBe("Varrendo ~/projects");
    expect(t("progress.label", { done: 2, total: 5 })).toBe("Progresso: 2 de 5 tarefas");
  });

  it("leaves unknown placeholders visible", () => {
    expect(t("scan.scanning")).toBe("Varrendo {path}");
  });

  it("switches language at run time, with plurals and numbers of that language", () => {
    setLocale("en");
    expect(t("sidebar.title")).toBe("Projects");
    expect(t("sidebar.versions", { n: 1 })).toBe("1 version");
    expect(t("sidebar.versions", { n: 0 })).toBe("0 versions");
    expect(formatNumber(1.5, 1)).toBe("1.5");
    expect(document.documentElement.lang).toBe("en");
    setLocale("pt-BR");
    expect(formatNumber(1.5, 1)).toBe("1,5");
    expect(t("sidebar.versions", { n: 0 })).toBe("0 versão");
  });

  it("formats dates with the month names of the language in use", () => {
    const ts = new Date(2026, 1, 4, 17, 42).toISOString();
    expect(formatWhen(ts)).toBe("04 fev · 17:42");
    setLocale("en");
    expect(formatWhen(ts)).toBe("04 Feb · 17:42");
  });
});

describe("catalogs (DEB-01)", () => {
  it("en has exactly the keys of pt-BR, with the same parameters", () => {
    const pt = leaves(ptBR);
    const english = leaves(en);
    expect([...english.keys()].sort()).toEqual([...pt.keys()].sort());
    for (const [key, params] of pt) expect([key, english.get(key)]).toEqual([key, params]);
  });

  it("en has no empty string", () => {
    for (const [key] of leaves(en)) expect(key.length).toBeGreaterThan(0);
    const empty = [...leaves(en).keys()].filter((k) => {
      const parts = k.split(".");
      let node: unknown = en;
      for (const p of parts) node = (node as Tree)[p];
      return node === "";
    });
    expect(empty).toEqual([]);
  });
});

describe("backend errors (DEB-02)", () => {
  it("are translated from the code, never shown as the backend text", () => {
    const error = { code: "path_not_found", message: "path does not exist: /x" };
    expect(errorText(error)).toBe("a pasta não existe");
    setLocale("en");
    expect(errorText(error)).toBe("the folder does not exist");
  });

  it("unknown codes fall back to a generic message", () => {
    expect(errorText({ code: "weird" })).toBe("Erro inesperado");
  });
});
