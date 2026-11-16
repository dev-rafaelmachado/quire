import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (rel: string) => readFileSync(new URL(rel, import.meta.url), "utf8");
const designTokens = read("../../../../docs/design/tokens.css");
const appTokens = read("./tokens.css");

type Palette = Record<string, string>;

function palettes(css: string): Record<"qd" | "ql", Palette> {
  const read = (cls: string): Palette => {
    const body = new RegExp(`\\.${cls}\\{([^}]*)\\}`).exec(css)?.[1] ?? "";
    return Object.fromEntries([...body.matchAll(/--([\w-]+):(#[0-9a-fA-F]{6})/g)].map((m) => [m[1] ?? "", m[2] ?? ""]));
  };
  return { qd: read("qd"), ql: read("ql") };
}

function luminance(hex: string): number {
  const channel = (i: number) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
}

export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

// Text/background pairs used by the UI (wireframes 1-4).
const TEXTS = ["text", "muted", "faint", "acc", "ok", "warn"] as const;
const SURFACES = ["bg", "side", "panel", "raise"] as const;
const TINTED: ReadonlyArray<readonly [string, string]> = [
  ["acc", "acc-bg"],
  ["ok", "ok-bg"],
  ["warn", "warn-bg"],
  ["text", "code"],
  ["muted", "code"],
  ["faint", "code"],
];

describe("design tokens", () => {
  it("the app uses an exact copy of docs/design/tokens.css", () => {
    expect(appTokens.length).toBeGreaterThan(500);
    expect(appTokens).toBe(designTokens);
  });

  for (const theme of ["qd", "ql"] as const) {
    it(`text contrast is at least 4.5:1 in ${theme === "qd" ? "dark" : "light"} theme`, () => {
      const p = palettes(appTokens)[theme];
      const pairs = [...TEXTS.flatMap((f) => SURFACES.map((b) => [f, b] as const)), ...TINTED];
      const failures = pairs
        .map(([f, b]) => ({ pair: `${f}/${b}`, ratio: contrast(p[f] ?? "#000000", p[b] ?? "#000000") }))
        .filter((r) => r.ratio < 4.5);
      expect(Object.keys(p).length).toBeGreaterThan(10);
      expect(failures).toEqual([]);
    });
  }

  it("contrast() matches known WCAG values", () => {
    expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrast("#777777", "#ffffff")).toBeCloseTo(4.48, 2);
  });
});
