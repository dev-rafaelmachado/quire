import { describe, expect, it } from "vitest";
import { resolveLocale } from "./locale";

describe("resolveLocale (DEB-10, TD-23)", () => {
  const cases: Array<[string | null, string[], "pt-BR" | "en"]> = [
    [null, ["pt-BR"], "pt-BR"],
    [null, ["pt-PT"], "pt-BR"],
    [null, ["pt"], "pt-BR"],
    [null, ["en-US"], "en"],
    [null, ["en_GB"], "en"],
    [null, ["es-ES"], "en"],
    [null, ["es-ES", "pt-BR"], "pt-BR"],
    [null, ["de-DE", "en-US", "pt-BR"], "en"],
    [null, [], "en"],
    ["en", ["pt-BR"], "en"],
    ["pt-BR", ["en-US"], "pt-BR"],
    ["fr", ["pt-BR"], "pt-BR"],
    ["fr", ["es-ES"], "en"],
    ["", ["pt-BR"], "pt-BR"],
  ];
  for (const [configured, system, expected] of cases) {
    it(`${JSON.stringify(configured)} with ${JSON.stringify(system)} -> ${expected}`, () => {
      expect(resolveLocale(configured, system)).toBe(expected);
    });
  }
});
