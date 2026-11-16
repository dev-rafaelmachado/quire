import { describe, expect, it } from "vitest";
import { cleanAnchors, lineForTop, topForLine } from "./syncScroll";

// A document: title (line 1), paragraph (3), heading (8), table (10..14), code (16..30).
const anchors = cleanAnchors([
  { line: 1, top: 0 },
  { line: 3, top: 60 },
  { line: 8, top: 160 },
  { line: 10, top: 210 },
  { line: 16, top: 400 },
]);

describe("split view scroll mapping (EDT-04)", () => {
  it("an editor scrolled to the table puts the preview on the table, and back", () => {
    expect(topForLine(anchors, 10)).toBe(210);
    expect(lineForTop(anchors, 210)).toBe(10);
  });

  it("interpolates inside a block in both directions", () => {
    expect(topForLine(anchors, 13)).toBeCloseTo(210 + (190 * 3) / 6);
    const top = topForLine(anchors, 12.5) ?? 0;
    expect(lineForTop(anchors, top)).toBeCloseTo(12.5);
  });

  it("clamps before the first and after the last block", () => {
    expect(topForLine(anchors, 0)).toBe(0);
    expect(topForLine(anchors, 99)).toBe(400);
    expect(lineForTop(anchors, -5)).toBe(1);
    expect(lineForTop(anchors, 9999)).toBe(16);
    expect(topForLine([], 3)).toBeNull();
    expect(lineForTop([], 3)).toBeNull();
  });

  it("drops anchors that go backwards (collapsed or hidden blocks)", () => {
    expect(cleanAnchors([{ line: 5, top: 100 }, { line: 3, top: 300 }, { line: 9, top: 200 }])).toEqual([
      { line: 3, top: 300 },
    ]);
  });
});
