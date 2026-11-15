// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

const openExternal = vi.fn(async (_url: string) => null);
vi.mock("../ipc", () => ({ commands: { openExternal: (url: string) => openExternal(url) } }));
vi.mock("./mermaid", () => ({
  renderMermaid: async (code: string) => {
    if (code.includes("quebrado")) throw new Error("Parse error on line 2");
    return "<svg data-testid='diagram'></svg>";
  },
}));
vi.mock("./highlight", () => ({ highlight: async () => null, canHighlight: () => false }));

const { Markdown } = await import("./Markdown");
const { buildDoc, toHast } = await import("./pipeline");

const MD = `# Título

## Tabela

| Coluna | Valor |
| --- | --- |
| a | 1 |

- [x] feita
- [ ] pendente

## Código

\`\`\`
┌──┐   ┌──┐
│a │──▶│b │
└──┘   └──┘
\`\`\`

\`\`\`mermaid
graph TD
  A[quebrado
\`\`\`

\`\`\`mermaid
graph TD
  A-->B
\`\`\`

Veja [o tracker](https://tracker.example/browse/ACME-1) e [o design](./design.md).
`;

function Harness() {
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());
  const doc = buildDoc(toHast(MD));
  return (
    <Markdown
      doc={doc}
      collapsed={collapsed}
      onToggle={(id) =>
        setCollapsed((prev) => {
          const next = new Set(prev);
          if (next.has(id)) next.delete(id);
          else next.add(id);
          return next;
        })
      }
    />
  );
}

afterEach(cleanup);

describe("Markdown", () => {
  it("renders GFM tables and read-only task lists (VIW-02)", () => {
    render(<Harness />);
    expect(screen.getByRole("table")).toBeTruthy();
    const boxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
    expect(boxes.map((b) => [b.checked, b.disabled])).toEqual([
      [true, true],
      [false, true],
    ]);
  });

  it("keeps ASCII diagrams verbatim in a code block (VIW-04)", () => {
    const { container } = render(<Harness />);
    const pre = container.querySelector(".md-code pre");
    expect(pre?.textContent).toContain("│a │──▶│b │");
  });

  it("shows invalid Mermaid as code with the error and still renders the rest (VIW-03)", async () => {
    render(<Harness />);
    expect((await screen.findByRole("alert")).textContent).toContain("Parse error on line 2");
    expect(await screen.findByTestId("diagram")).toBeTruthy();
    expect(screen.getByText(/A\[quebrado/)).toBeTruthy();
  });

  it("collapses and expands a section (VIW-06)", () => {
    render(<Harness />);
    fireEvent.click(screen.getByRole("button", { name: "Recolher seção Tabela" }));
    expect(screen.queryByRole("table")).toBeNull(); // hidden content leaves the accessibility tree
    fireEvent.click(screen.getByRole("button", { name: "Expandir seção Tabela" }));
    expect(screen.getByRole("table")).toBeTruthy();
  });

  it("opens http links in the browser and never navigates the webview", () => {
    render(<Harness />);
    const external = screen.getByRole("link", { name: "o tracker" });
    const ev = new MouseEvent("click", { bubbles: true, cancelable: true });
    external.dispatchEvent(ev);
    expect(ev.defaultPrevented).toBe(true);
    expect(openExternal).toHaveBeenCalledWith("https://tracker.example/browse/ACME-1");
    const relative = new MouseEvent("click", { bubbles: true, cancelable: true });
    screen.getByRole("link", { name: "o design" }).dispatchEvent(relative);
    expect(relative.defaultPrevented).toBe(true);
    expect(openExternal).toHaveBeenCalledTimes(1);
  });
});
