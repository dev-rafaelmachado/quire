const FIELD = /^\*\*[^*]+?(?::\*\*|\*\*:)/;

/** Removes the `**Key:** value` block after the title (same rule as quire-core's header
 *  parser), so it is shown once, as metadata chips (VIW-07), and not again as body text. */
export function stripHeader(md: string): string {
  const lines = md.split("\n");
  let i = 0;
  let sawTitle = false;
  while (i < lines.length) {
    const t = (lines[i] ?? "").trim();
    if (t === "" || (!sawTitle && t.startsWith("# "))) {
      if (t.startsWith("# ")) sawTitle = true;
      i++;
      continue;
    }
    break;
  }
  const start = i;
  while (i < lines.length && FIELD.test((lines[i] ?? "").trim())) i++;
  if (i === start) return md;
  return [...lines.slice(0, start), ...lines.slice(i)].join("\n");
}
