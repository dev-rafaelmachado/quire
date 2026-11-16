// Scroll alignment of Dividir (EDT-04): both sides map to source lines through anchors, one per
// top-level block of the preview (`data-line`), interpolating inside a block.

export interface Anchor {
  /** First source line of the block (1-based). */
  line: number;
  /** Offset of the block from the top of the preview's scrollable content. */
  top: number;
}

/** Anchors in document order, keeping only those that move forward on both axes. */
export function cleanAnchors(anchors: readonly Anchor[]): Anchor[] {
  const out: Anchor[] = [];
  for (const a of [...anchors].sort((x, y) => x.line - y.line || x.top - y.top)) {
    const last = out[out.length - 1];
    if (!last || (a.line > last.line && a.top >= last.top)) out.push(a);
  }
  return out;
}

/** Where the preview should be for a (fractional) source line at the top of the editor. */
export function topForLine(anchors: readonly Anchor[], line: number): number | null {
  if (anchors.length === 0) return null;
  let i = 0;
  while (i + 1 < anchors.length && (anchors[i + 1]?.line ?? Infinity) <= line) i++;
  const a = anchors[i];
  if (!a) return null;
  if (line <= a.line) return a.top;
  const b = anchors[i + 1];
  if (!b) return a.top;
  return a.top + ((b.top - a.top) * (line - a.line)) / (b.line - a.line);
}

/** The (fractional) source line shown at a preview offset. */
export function lineForTop(anchors: readonly Anchor[], top: number): number | null {
  if (anchors.length === 0) return null;
  let i = 0;
  while (i + 1 < anchors.length && (anchors[i + 1]?.top ?? Infinity) <= top) i++;
  const a = anchors[i];
  if (!a) return null;
  if (top <= a.top) return a.line;
  const b = anchors[i + 1];
  if (!b || b.top === a.top) return a.line;
  return a.line + ((b.line - a.line) * (top - a.top)) / (b.top - a.top);
}
