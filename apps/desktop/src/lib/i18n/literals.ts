/** Finds user-visible string literals in TSX source: JSX text and text attributes. */
export interface Literal {
  line: number;
  text: string;
}

const TEXT_ATTRS = /\b(aria-label|placeholder|title|alt|label)="([^"]*)"/g;
const JSX_TEXT = />([^<>{}]*)</g;
const LETTER = /\p{L}/u;
// JSX text never looks like code; these mark a `a > b ... c < d` expression instead.
const CODE_HINT = /&&|\|\||=|;|\(|\)/;

function stripComments(src: string): string {
  return src.replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, " ")).replace(/\/\/[^\n]*/g, "");
}

function lineOf(src: string, index: number): number {
  return src.slice(0, index).split("\n").length;
}

export function findLiterals(source: string): Literal[] {
  const src = stripComments(source);
  const out: Literal[] = [];
  for (const m of src.matchAll(JSX_TEXT)) {
    const text = (m[1] ?? "").trim();
    if (text && LETTER.test(text) && !CODE_HINT.test(text)) out.push({ line: lineOf(src, m.index), text });
  }
  for (const m of src.matchAll(TEXT_ATTRS)) {
    const text = m[2] ?? "";
    if (LETTER.test(text)) out.push({ line: lineOf(src, m.index), text });
  }
  return out;
}
