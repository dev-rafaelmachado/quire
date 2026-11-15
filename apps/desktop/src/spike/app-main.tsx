// T16 measurement (disposable, outside the app build): the synthetic ~40 KB spec rendered
// by the app's real pipeline (lib/markdown: Shiki/Oniguruma, Mermaid/dagre, app CSS), with
// the same metrics as the T2 spike. One run per page load; `?scale=N`.
import "../index.css";
import "../app/theme";
import type { Element, Root, RootContent } from "hast";
import { StrictMode, useLayoutEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { canHighlight } from "../lib/markdown/highlight";
import { Markdown } from "../lib/markdown/Markdown";
import { buildDoc, codeOf, type Section, toHast } from "../lib/markdown/pipeline";
import { generateMarkdown } from "./generate";

const params = new URLSearchParams(location.search);
const scale = Number(params.get("scale") ?? "1");
const nextFrame = (): Promise<number> => new Promise((r) => requestAnimationFrame(r));
const r1 = (x: number): number => Math.round(x * 10) / 10;

function stats(xs: number[]): Record<string, number> {
  const s = [...xs].sort((a, b) => a - b);
  const pick = (q: number): number => s[Math.min(s.length - 1, Math.floor(q * s.length))] ?? 0;
  return {
    n: s.length,
    mean: r1(s.reduce((a, b) => a + b, 0) / Math.max(1, s.length)),
    p50: r1(pick(0.5)),
    p95: r1(pick(0.95)),
    max: r1(s.at(-1) ?? 0),
    over20ms: s.filter((x) => x > 20).length,
    over34ms: s.filter((x) => x > 34).length,
  };
}

async function scrollTest(el: HTMLElement, px: number, maxFrames: number): Promise<Record<string, number>> {
  el.scrollTop = 0;
  await nextFrame();
  const deltas: number[] = [];
  let last = await nextFrame();
  for (let f = 0; f < maxFrames && el.scrollTop + el.clientHeight < el.scrollHeight - 1; f++) {
    el.scrollTop += px;
    const now = await nextFrame();
    deltas.push(now - last);
    last = now;
  }
  return { pxPerFrame: px, ...stats(deltas) };
}

const tGen0 = performance.now();
const md = generateMarkdown(scale);
const genMs = performance.now() - tGen0;
const tParse0 = performance.now();
const hast = toHast(md);
const doc = buildDoc(hast);
const parseMs = performance.now() - tParse0;

const expected = { highlight: 0, mermaid: 0 };
const walk = (n: Root | RootContent): void => {
  if (n.type === "element" && n.tagName === "pre") {
    const info = codeOf(n as Element);
    if (info?.lang === "mermaid") expected.mermaid++;
    else if (info && canHighlight(info.lang)) expected.highlight++;
    return;
  }
  if (n.type === "root" || n.type === "element") n.children.forEach(walk);
};
walk(hast);
const allIds: string[] = [];
const collect = (ss: Section[]): void => ss.forEach((s) => (allIds.push(s.id), collect(s.children)));
collect(doc.sections);

function App() {
  const scroller = useRef<HTMLDivElement>(null);
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());
  const started = useRef(false);
  useLayoutEffect(() => {
    if (started.current) return;
    started.current = true;
    const tCommit = performance.now();
    void (async () => {
      await nextFrame();
      await nextFrame();
      const tFirstPaint = performance.now();
      const el = scroller.current;
      if (!el) return;
      const t0 = performance.now();
      let tShiki = 0;
      let tMermaid = 0;
      while (performance.now() - t0 < 30000) {
        const hl = el.querySelectorAll(".md-code .shiki").length;
        const mm = el.querySelectorAll(".md-mermaid svg").length + el.querySelectorAll(".md-mermaid [role=alert]").length;
        if (!tShiki && hl >= expected.highlight) tShiki = performance.now();
        if (!tMermaid && mm >= expected.mermaid) tMermaid = performance.now();
        if (tShiki && tMermaid) break;
        await nextFrame();
      }
      await new Promise((r) => setTimeout(r, 500));
      const scrollFast = await scrollTest(el, 120, 2000);
      const scrollMedium = await scrollTest(el, 40, 900);
      let t = performance.now();
      setCollapsed(new Set(allIds));
      await nextFrame();
      await nextFrame();
      const collapseAllMs = performance.now() - t;
      t = performance.now();
      setCollapsed(new Set());
      await nextFrame();
      await nextFrame();
      const expandAllMs = performance.now() - t;
      const report = {
        kind: "t16",
        scale,
        mdBytes: new TextEncoder().encode(md).length,
        expected,
        domNodes: document.getElementsByTagName("*").length,
        genMs: r1(genMs),
        parseMs: r1(parseMs),
        tCommit: r1(tCommit),
        tFirstPaint: r1(tFirstPaint),
        tShikiDone: tShiki ? r1(tShiki) : null,
        tMermaidDone: tMermaid ? r1(tMermaid) : null,
        tAllDone: tShiki && tMermaid ? r1(Math.max(tShiki, tMermaid)) : null,
        mermaidErrors: el.querySelectorAll(".md-mermaid [role=alert]").length,
        mermaidSvgs: el.querySelectorAll(".md-mermaid svg").length,
        highlighted: el.querySelectorAll(".md-code .shiki").length,
        scrollHeight: el.scrollHeight,
        scrollFast,
        scrollMedium,
        collapseAllMs: r1(collapseAllMs),
        expandAllMs: r1(expandAllMs),
        resources: performance
          .getEntriesByType("resource")
          .map((e) => new URL(e.name).pathname)
          .filter((n) => n.startsWith("/assets/")),
      };
      await fetch("/__spike/report", { method: "POST", body: JSON.stringify(report) }).catch(() => undefined);
      await fetch("/__spike/report", { method: "POST", body: JSON.stringify({ kind: "done" }) }).catch(() => undefined);
    })();
  }, []);
  return (
    // Inline styles: the app CSS does not scan src/spike for Tailwind classes.
    <div ref={scroller} style={{ height: "100vh", overflowY: "auto", background: "var(--bg)" }}>
      <div style={{ padding: "36px 40px 96px" }}>
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
      </div>
    </div>
  );
}

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
