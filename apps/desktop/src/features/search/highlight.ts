// Occurrences of the searched terms in the rendered file (SRC-06), with the CSS Custom
// Highlight API: nothing is written into the DOM React owns. Matching ignores case and accents,
// like the index (SRC-03).

const NAME = "quire-search";

/** Lowercase, accents removed, with the index in the original string of each character. */
export function fold(text: string): { folded: string; map: number[] } {
  let folded = "";
  const map: number[] = [];
  for (let i = 0; i < text.length; i++) {
    const base = (text[i] ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
    for (let k = 0; k < base.length; k++) {
      folded += base[k];
      map.push(i);
    }
  }
  return { folded, map };
}

/** `[start, end)` ranges of `terms` in `text`, ignoring case and accents; the last term also
 *  matches as a prefix, like the query. */
export function findTerms(text: string, terms: readonly string[]): Array<[number, number]> {
  const { folded, map } = fold(text);
  const out: Array<[number, number]> = [];
  for (const term of terms) {
    const t = fold(term).folded.replace(/[^\p{L}\p{N}]/gu, "");
    if (!t) continue;
    let at = folded.indexOf(t);
    while (at >= 0) {
      const start = map[at] ?? 0;
      const end = (map[at + t.length - 1] ?? start) + 1;
      out.push([start, end]);
      at = folded.indexOf(t, at + t.length);
    }
  }
  return out.sort((a, b) => a[0] - b[0]);
}

interface HighlightRegistry {
  set(name: string, value: unknown): void;
  delete(name: string): void;
}

function registry(): HighlightRegistry | null {
  const css = (globalThis as { CSS?: { highlights?: HighlightRegistry } }).CSS;
  return css?.highlights ?? null;
}

/** Highlights the terms inside `root`; returns how many occurrences were found. */
export function highlightTerms(root: Element, terms: readonly string[]): number {
  const reg = registry();
  const HighlightCtor = (globalThis as { Highlight?: new (...ranges: Range[]) => unknown }).Highlight;
  const ranges: Range[] = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = node.textContent ?? "";
    for (const [start, end] of findTerms(text, terms)) {
      const r = document.createRange();
      r.setStart(node, start);
      r.setEnd(node, end);
      ranges.push(r);
    }
  }
  if (reg && HighlightCtor) {
    if (ranges.length > 0) reg.set(NAME, new HighlightCtor(...ranges));
    else reg.delete(NAME);
  }
  return ranges.length;
}

export function clearHighlights(): void {
  registry()?.delete(NAME);
}
