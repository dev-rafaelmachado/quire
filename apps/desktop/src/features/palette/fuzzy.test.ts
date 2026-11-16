import { describe, expect, it } from "vitest";
import { fuzzyScore, rank } from "./fuzzy";

describe("palette matching (TD-17)", () => {
  it("matches a subsequence, ignoring case and accents", () => {
    expect(fuzzyScore("acm142", "ACME-142 — Tokens")).not.toBeNull();
    expect(fuzzyScore("integ", "Integrações")).not.toBeNull();
    expect(fuzzyScore("acao", "Ação")).not.toBeNull();
    expect(fuzzyScore("xyz", "ACME-142")).toBeNull();
    expect(fuzzyScore("241", "ACME-142")).toBeNull();
  });

  it("word starts and runs beat scattered letters", () => {
    const starts = fuzzyScore("tst", "Tokens de serviço teste") ?? 0;
    const run = fuzzyScore("tok", "Tokens") ?? 0;
    const scattered = fuzzyScore("tok", "attack of robots") ?? 0;
    expect(run).toBeGreaterThan(scattered);
    expect(starts).toBeGreaterThan(0);
    // Abbreviation by initials: "lt" finds "Limite de taxa" before "volta".
    expect(fuzzyScore("ldt", "Limite de taxa") ?? 0).toBeGreaterThan(fuzzyScore("ldt", "valid data") ?? 0);
  });

  it("orders items by the best weighted field (issue key counts more)", () => {
    const byKey = { fields: [{ text: "ACME-170", weight: 2 }, { text: "Limite de taxa por cliente", weight: 1 }] };
    const byTitle = { fields: [{ text: "BILL-40", weight: 2 }, { text: "Acme estornos 170", weight: 1 }] };
    expect(rank("acme170", byKey) ?? 0).toBeGreaterThan(rank("acme170", byTitle) ?? 0);
    expect(rank("zzz", byKey)).toBeNull();
    expect(rank("", byKey)).toBe(0);
  });
});
