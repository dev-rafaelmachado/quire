// Mermaid, imported only when a document has a diagram (TD-04). Renders are serialized because
// mermaid.render is not re-entrant. Layout is dagre: Mermaid 12 defaults to ELK, which the T2
// spike measured at +1.46 MB and slower renders; diagrams that ask for ELK still get it.
type MermaidApi = (typeof import("mermaid"))["default"];

let mermaid: Promise<MermaidApi> | null = null;
let queue: Promise<unknown> = Promise.resolve();
let seq = 0;

export function renderMermaid(code: string, dark: boolean): Promise<string> {
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
    const { svg } = await m.render(`quire-mmd-${seq++}`, code);
    return svg;
  });
  queue = job.catch(() => undefined);
  return job;
}
