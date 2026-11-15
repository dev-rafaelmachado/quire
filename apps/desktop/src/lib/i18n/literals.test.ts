import { describe, expect, it } from "vitest";
import { findLiterals } from "./literals";

const sources = import.meta.glob<string>(["/src/**/*.tsx", "!/src/**/*.test.tsx", "!/src/spike/**"], {
  query: "?raw",
  import: "default",
  eager: true,
});

describe("UI strings come from the catalog", () => {
  it("detects literals (self-test)", () => {
    const bad = `<p>Olá mundo</p>\n<input placeholder="Buscar" />\n<button aria-label="Fechar"><X /></button>`;
    expect(findLiterals(bad).map((l) => l.text)).toEqual(["Olá mundo", "Buscar", "Fechar"]);
    const good = `<p>{t("a.b")}</p>\n<span aria-hidden="true">/</span>\n{a > b && c < d ? x : y}\n<i>{n}/{m}</i>`;
    expect(findLiterals(good)).toEqual([]);
  });

  it("finds components to check", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(3);
  });

  for (const [file, source] of Object.entries(sources)) {
    it(`${file} has no visible string literal`, () => {
      expect(findLiterals(source)).toEqual([]);
    });
  }
});
