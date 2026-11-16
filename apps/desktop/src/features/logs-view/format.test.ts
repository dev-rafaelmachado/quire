import { describe, expect, it } from "vitest";
import { formatDuration, formatWhen, shortPath } from "./format";

describe("log formatting", () => {
  it("formats timestamps in local time", () => {
    const d = new Date(2026, 10, 4, 17, 42);
    expect(formatWhen(d.toISOString())).toBe("04 nov · 17:42");
    expect(formatWhen(null)).toBe("—");
    expect(formatWhen("not a date")).toBe("not a date");
  });

  it("formats durations", () => {
    expect(formatDuration(41.2)).toBe("41s");
    expect(formatDuration(252)).toBe("4m 12s");
    expect(formatDuration(3780)).toBe("1h 03m");
    expect(formatDuration(null)).toBe("—");
  });

  it("shortens long paths", () => {
    expect(shortPath("/home/dev/acme/.worktrees/acme-142/acme_gateway")).toBe("…/.worktrees/acme-142/acme_gateway");
    expect(shortPath("/a/b")).toBe("/a/b");
  });
});
