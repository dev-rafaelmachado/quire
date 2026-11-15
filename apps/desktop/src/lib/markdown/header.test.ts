import { describe, expect, it } from "vitest";
import { stripHeader } from "./header";

describe("stripHeader", () => {
  it("removes the field block after the title", () => {
    const md = "# ACME-1 — T\n\n**Status:** ok\n**Issue:** [A](u)\n\n## Problema\n**Not:** header\n";
    expect(stripHeader(md)).toBe("# ACME-1 — T\n\n\n## Problema\n**Not:** header\n");
  });

  it("accepts the colon outside the bold and leaves other files intact", () => {
    expect(stripHeader("**Spec**: `x`\ntext")).toBe("text");
    expect(stripHeader("# T\n\nJust prose.\n")).toBe("# T\n\nJust prose.\n");
  });
});
