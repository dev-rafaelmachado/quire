// Mermaid, imported only when a document has a diagram (TD-04). Renders are serialized because
// mermaid.render is not re-entrant. Layout is dagre: Mermaid 12 defaults to ELK, which the T2
// spike measured at +1.46 MB and slower renders; diagrams that ask for ELK still get it.
type MermaidApi = (typeof import("mermaid"))["default"];

let mermaid: Promise<MermaidApi> | null = null;
let queue: Promise<unknown> = Promise.resolve();
let seq = 0;

// Rendered diagrams and errors by source and theme (T17): typing outside a diagram must not
// render it again.
const CACHE_LIMIT = 100;
const done = new Map<string, { svg: string } | { error: string }>();

function key(code: string, dark: boolean): string {
  return `${dark ? "d" : "l"}\u0000${code}`;
}

/** The finished render of this diagram, if any. */
export function cachedMermaid(code: string, dark: boolean): { svg: string } | { error: string } | undefined {
  return done.get(key(code, dark));
}

function remember(code: string, dark: boolean, value: { svg: string } | { error: string }): void {
  if (done.size >= CACHE_LIMIT) {
    const oldest = done.keys().next().value;
    if (oldest !== undefined) done.delete(oldest);
  }
  done.set(key(code, dark), value);
}

export function renderMermaid(code: string, dark: boolean): Promise<string> {
  const hit = cachedMermaid(code, dark);
  if (hit) return "svg" in hit ? Promise.resolve(hit.svg) : Promise.reject(new Error(hit.error));
  const job = queue.then(async () => {
    mermaid ??= import("mermaid").then((m) => m.default);
    const m = await mermaid;
    m.initialize({
      startOnLoad: false,
      theme: dark ? "dark" : "default",
      securityLevel: "strict",
      suppressErrorRendering: true,
      layout: "dagre",
    });
    try {
      const { svg } = await m.render(`quire-mmd-${seq++}`, code);
      remember(code, dark, { svg });
      return svg;
    } catch (e) {
      remember(code, dark, { error: e instanceof Error ? e.message : String(e) });
      throw e;
    }
  });
  queue = job.catch(() => undefined);
  return job;
}
