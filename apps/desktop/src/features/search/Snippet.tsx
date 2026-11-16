import type { ReactNode } from "react";

/** Splits a snippet marked with U+0002 / U+0003 into text and `<mark>` (SRC-04). Never HTML:
 *  whatever the file contains is shown as text. */
export function snippetParts(snippet: string): Array<{ text: string; mark: boolean }> {
  const out: Array<{ text: string; mark: boolean }> = [];
  let mark = false;
  let buf = "";
  for (const ch of snippet) {
    if (ch === "\u0002" || ch === "\u0003") {
      if (buf) out.push({ text: buf, mark });
      buf = "";
      mark = ch === "\u0002";
    } else {
      buf += ch;
    }
  }
  if (buf) out.push({ text: buf, mark });
  return out;
}

export function Snippet({ snippet }: { snippet: string }): ReactNode {
  return (
    <>
      {snippetParts(snippet).map((p, i) =>
        p.mark ? (
          <mark key={i} className="rounded-sm bg-warn-bg px-0.5 text-text">
            {p.text}
          </mark>
        ) : (
          <span key={i}>{p.text}</span>
        ),
      )}
    </>
  );
}
