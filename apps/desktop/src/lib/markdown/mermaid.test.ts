import { describe, expect, it, vi } from "vitest";

const render = vi.fn(async (_id: string, code: string) => {
  if (code.includes("broken")) throw new Error("Parse error");
  return { svg: `<svg>${code.length}</svg>` };
});
vi.mock("mermaid", () => ({ default: { initialize: () => undefined, render: (id: string, code: string) => render(id, code) } }));

const { cachedMermaid, renderMermaid } = await import("./mermaid");

describe("mermaid cache (T17)", () => {
  it("renders a diagram once per source and theme", async () => {
    const code = "flowchart TD\n  A --> B";
    expect(cachedMermaid(code, true)).toBeUndefined();
    const a = await renderMermaid(code, true);
    const b = await renderMermaid(code, true);
    expect(a).toBe(b);
    expect(render).toHaveBeenCalledTimes(1);
    expect(cachedMermaid(code, true)).toEqual({ svg: a });
    await renderMermaid(code, false);
    expect(render).toHaveBeenCalledTimes(2);
  });

  it("remembers errors too, so typing elsewhere does not retry a broken diagram", async () => {
    render.mockClear();
    await expect(renderMermaid("broken", true)).rejects.toThrow("Parse error");
    await expect(renderMermaid("broken", true)).rejects.toThrow("Parse error");
    expect(render).toHaveBeenCalledTimes(1);
  });
});
