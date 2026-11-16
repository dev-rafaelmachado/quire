import { describe, expect, it } from "vitest";
import { ancestry, buildDoc, codeOf, outline, toHast } from "./pipeline";

const MD = `# Título

intro

## Problema

texto <!-- TOK-01 -->

### P1: Detalhe

\`\`\`ts
const a = 1;
\`\`\`

## Problema

| a | b |
| - | - |
| 1 | 2 |

- [x] feito
`;

describe("markdown pipeline", () => {
  const doc = buildDoc(toHast(MD));

  it("nests sections by heading level with unique ids", () => {
    expect(doc.intro).toHaveLength(0);
    expect(doc.sections.map((s) => s.id)).toEqual(["h-titulo"]);
    const h1 = doc.sections[0];
    expect(h1?.children.map((s) => s.id)).toEqual(["h-problema", "h-problema-1"]);
    expect(h1?.children[0]?.children[0]?.title).toBe("P1: Detalhe");
  });

  it("builds the outline and the ancestry of a section", () => {
    expect(outline(doc).map((e) => `${e.depth}:${e.title}`)).toEqual(["2:Problema", "3:P1: Detalhe", "2:Problema"]);
    expect(ancestry(doc, "h-p1-detalhe")).toEqual(["h-titulo", "h-problema", "h-p1-detalhe"]);
    expect(ancestry(doc, "nope")).toEqual([]);
  });

  it("drops HTML comments and keeps GFM tables and task lists", () => {
    const html = JSON.stringify(toHast(MD));
    expect(html).not.toContain("TOK-01");
    expect(html).toContain('"tagName":"table"');
    expect(html).toContain('"type":"checkbox"');
  });

  it("reads fenced code blocks", () => {
    const pre = doc.sections[0]?.children[0]?.children[0]?.body.find((n) => n.type === "element" && n.tagName === "pre");
    expect(pre && pre.type === "element" ? codeOf(pre) : null).toEqual({ lang: "ts", code: "const a = 1;" });
  });
});
