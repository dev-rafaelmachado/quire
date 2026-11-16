import { describe, expect, it } from "vitest";
import { chipsThatFit } from "./ChipList";

describe("chipsThatFit (DEB-04)", () => {
  it("keeps every chip when they fit", () => {
    expect(chipsThatFit([50, 50, 50], 158)).toBe(3);
  });
  it("leaves room for +N when some do not fit", () => {
    // 50 + 4 + 50 = 104, plus 34 for "+1" = 138 <= 140; a third chip would need 158.
    expect(chipsThatFit([50, 50, 50], 140)).toBe(2);
    expect(chipsThatFit([50, 50, 50], 100)).toBe(1);
  });
  it("shows only +N when not even one fits", () => {
    expect(chipsThatFit([80, 80], 60)).toBe(0);
  });
  it("a single chip needs no room for +N", () => {
    expect(chipsThatFit([50], 50)).toBe(1);
  });
});
