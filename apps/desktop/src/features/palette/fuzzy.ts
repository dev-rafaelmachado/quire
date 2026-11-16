// Approximate matching of the palette (TD-17): the query's characters in order (subsequence),
// ignoring case and accents, with bonuses for word starts, runs and the start of the text.

function fold(s: string): string {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

const SEPARATOR = /[\s\-_/.:·—()[\]]/;

/** Score of `query` against `text`, or `null` when it is not a subsequence. Higher is better. */
export function fuzzyScore(query: string, text: string): number | null {
  const q = fold(query).replace(/\s+/g, "");
  if (q === "") return 0;
  const t = fold(text);
  let score = 0;
  let ti = 0;
  let prev = -2;
  for (const ch of q) {
    let found = -1;
    // Prefer a word start for this character when one is ahead; else the next occurrence.
    for (let k = ti; k < t.length; k++) {
      if (t[k] !== ch) continue;
      const start = k === 0 || SEPARATOR.test(t[k - 1] ?? "");
      if (start) {
        found = k;
        break;
      }
      if (found === -1) found = k;
    }
    if (found === -1) return null;
    const atStart = found === 0 || SEPARATOR.test(t[found - 1] ?? "");
    score += 1;
    if (atStart) score += 6;
    if (found === prev + 1) score += 4;
    if (found === 0) score += 3;
    score -= Math.min(3, (found - ti) * 0.05);
    prev = found;
    ti = found + 1;
  }
  // Shorter texts with the same matches rank first.
  return score - t.length * 0.01;
}

export interface Rankable {
  /** Fields matched, with a weight each (e.g. the issue key counts double). */
  fields: ReadonlyArray<{ text: string; weight: number }>;
}

/** Best weighted score over the fields of an item, or `null` when none matches. */
export function rank(query: string, item: Rankable): number | null {
  let best: number | null = null;
  for (const f of item.fields) {
    const s = fuzzyScore(query, f.text);
    if (s !== null && (best === null || s * f.weight > best)) best = s * f.weight;
  }
  return best;
}
