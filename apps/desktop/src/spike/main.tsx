// T2 spike (disposable): renders a synthetic ~40 KB spec and measures render, shiki,
// mermaid, scroll frame times and collapse/expand cost inside the real webview.
// Results are POSTed to the spike preview server (see vite.config.ts) and shown on the page.
import { StrictMode, useLayoutEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { generateMarkdown } from "./generate";
import { countBlocks, renderNodes, SKIP_MERMAID, SKIP_SHIKI, type Section, splitSections, toHast, tracker } from "./markdown";
import "./spike.css";

const T_MODULE = performance.now();

interface PlanStep {
  scale: number;
}
// Repeated runs at 1x (40 KB) plus a 3x (~125 KB) stress case.
const PLAN: readonly PlanStep[] = [{ scale: 1 }, { scale: 1 }, { scale: 1 }, { scale: 3 }, { scale: 3 }, { scale: 1 }];

const params = new URLSearchParams(location.search);
const mode = params.get("mode");
const stepIndex = Number(params.get("i") ?? "0");
const manual = params.has("manual");
const step: PlanStep = manual ? { scale: Number(params.get("scale") ?? "1") } : (PLAN[stepIndex] ?? { scale: 1 });

type Report = Record<string, unknown>;

function post(report: Report): Promise<void> {
  return fetch("/__spike/report", { method: "POST", body: JSON.stringify(report) }).then(
    () => undefined,
    () => undefined,
  );
}

const nextFrame = (): Promise<number> => new Promise((r) => requestAnimationFrame(r));
const sleep = (ms: number): Promise<void> => new Promise((r) => setTimeout(r, ms));

function stats(xs: number[]): Record<string, number> {
  if (xs.length === 0) return { n: 0 };
  const s = [...xs].sort((a, b) => a - b);
  const pick = (q: number): number => s[Math.min(s.length - 1, Math.floor(q * s.length))] ?? 0;
  const round = (x: number): number => Math.round(x * 100) / 100;
  return {
    n: s.length,
    mean: round(s.reduce((a, b) => a + b, 0) / s.length),
    p50: round(pick(0.5)),
    p95: round(pick(0.95)),
    p99: round(pick(0.99)),
    max: round(s.at(-1) ?? 0),
    over20ms: s.filter((x) => x > 20).length,
    over34ms: s.filter((x) => x > 34).length,
  };
}

/** Scrolls `el` from top to bottom moving `px` per frame, recording frame deltas. */
async function scrollTest(el: HTMLElement, px: number, maxFrames: number): Promise<Record<string, number>> {
  el.scrollTop = 0;
  await nextFrame();
  await nextFrame();
  const deltas: number[] = [];
  let last = await nextFrame();
  const t0 = last;
  for (let f = 0; f < maxFrames && el.scrollTop + el.clientHeight < el.scrollHeight - 1; f++) {
    el.scrollTop += px;
    const now = await nextFrame();
    deltas.push(now - last);
    last = now;
  }
  return { pxPerFrame: px, totalMs: Math.round(last - t0), reachedPx: Math.round(el.scrollTop), ...stats(deltas) };
}

function waitFor(cond: () => boolean, timeoutMs: number): Promise<boolean> {
  return new Promise((resolve) => {
    if (cond()) return resolve(true);
    const timer = setTimeout(() => {
      tracker.listeners.delete(check);
      resolve(false);
    }, timeoutMs);
    const check = (): void => {
      if (cond()) {
        clearTimeout(timer);
        tracker.listeners.delete(check);
        resolve(true);
      }
    };
    tracker.listeners.add(check);
  });
}

// ---------- prepare the document (timed) ----------

const tGen0 = performance.now();
const md = mode === "empty" ? "" : generateMarkdown(step.scale);
const tGen = performance.now() - tGen0;
const tParse0 = performance.now();
const hast = toHast(md);
const { preamble, sections } = splitSections(hast);
const counted = countBlocks(hast);
const expected = {
  ...counted,
  highlight: SKIP_SHIKI ? 0 : counted.highlight,
  mermaid: SKIP_MERMAID ? 0 : counted.mermaid,
};
const tParse = performance.now() - tParse0;

const longTasks: number[] = [];
const supported = typeof PerformanceObserver !== "undefined" ? [...PerformanceObserver.supportedEntryTypes] : [];
if (supported.includes("longtask")) {
  new PerformanceObserver((list) => list.getEntries().forEach((e) => longTasks.push(e.duration))).observe({
    type: "longtask",
    buffered: true,
  });
}

function SectionView({ section, collapsed, onToggle }: { section: Section; collapsed: boolean; onToggle: () => void }) {
  return (
    <section>
      <h2>
        <button type="button" aria-expanded={!collapsed} onClick={onToggle}>
          {collapsed ? "▸" : "▾"} {section.title}
        </button>
      </h2>
      <div hidden={collapsed}>{renderNodes(section.nodes)}</div>
    </section>
  );
}

function App() {
  const scroller = useRef<HTMLElement>(null);
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());
  const [results, setResults] = useState<string>("rodando…");
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
      if (!el || mode === "empty") {
        setResults("modo vazio (baseline de memória)");
        await post({ kind: "empty", tFirstPaint });
        return;
      }
      const shikiOk = await waitFor(() => tracker.highlighted >= expected.highlight, 30000);
      const tShiki = performance.now();
      const mermaidOk = await waitFor(() => tracker.mermaidOk + tracker.mermaidErr >= expected.mermaid, 60000);
      const tMermaid = performance.now();
      await sleep(500);

      const scrollFast = await scrollTest(el, 120, 2000);
      const scrollMedium = await scrollTest(el, 40, 900);

      // Collapse and expand every section, measuring time until the next frame.
      const all = new Set(sections.map((s) => s.id));
      let t0 = performance.now();
      setCollapsed(all);
      await nextFrame();
      await nextFrame();
      const collapseAllMs = performance.now() - t0;
      t0 = performance.now();
      setCollapsed(new Set());
      await nextFrame();
      await nextFrame();
      const expandAllMs = performance.now() - t0;

      const nav = performance.getEntriesByType("navigation")[0] as PerformanceNavigationTiming | undefined;
      const paint = performance.getEntriesByType("paint").map((e) => ({ name: e.name, t: Math.round(e.startTime) }));
      const r2 = (x: number): number => Math.round(x * 10) / 10;
      const report: Report = {
        kind: "run",
        step: stepIndex,
        scale: step.scale,
        layout: params.get("layout") ?? "default(elk)",
        engine: params.get("engine") ?? "javascript",
        skip: SKIP_SHIKI ? "shiki+mermaid" : SKIP_MERMAID ? "mermaid" : "none",
        ua: navigator.userAgent,
        visibility: document.visibilityState,
        dpr: devicePixelRatio,
        viewport: `${innerWidth}x${innerHeight}`,
        mdBytes: new TextEncoder().encode(md).length,
        sections: sections.length,
        blocks: expected,
        domNodes: document.getElementsByTagName("*").length,
        scrollHeight: el.scrollHeight,
        // All times in ms since navigation start unless named *Ms.
        navDomContentLoaded: nav ? r2(nav.domContentLoadedEventEnd) : null,
        moduleStart: r2(T_MODULE),
        genMs: r2(tGen),
        parseMs: r2(tParse),
        tCommit: r2(tCommit),
        tFirstPaint: r2(tFirstPaint),
        tShikiDone: r2(tShiki),
        shikiOk,
        shikiInitMs: r2(tracker.shikiInitMs),
        tMermaidDone: r2(tMermaid),
        mermaidOk,
        mermaidRendered: tracker.mermaidOk,
        mermaidErrors: tracker.mermaidErr,
        mermaidImportMs: r2(tracker.mermaidImportMs),
        mermaidPerDiagramMs: stats(tracker.mermaidDurations),
        scrollFast,
        scrollMedium,
        collapseAllMs: r2(collapseAllMs),
        expandAllMs: r2(expandAllMs),
        longTasks: supported.includes("longtask") ? stats(longTasks) : "unsupported",
        paint,
        supportedEntryTypes: supported,
        resources: performance
          .getEntriesByType("resource")
          .map((e) => new URL(e.name).pathname)
          .filter((n) => n.startsWith("/assets/")),
      };
      await post(report);
      setResults(JSON.stringify(report, null, 2));
      if (!manual && stepIndex + 1 < PLAN.length) {
        location.search = `?i=${stepIndex + 1}`;
      } else {
        await post({ kind: "done" });
      }
    })();
  }, []);

  return (
    <main ref={scroller} className="spike-scroll">
      <article className="doc">
        {renderNodes(preamble)}
        <p className="spike-controls">
          <button type="button" onClick={() => setCollapsed(new Set(sections.map((s) => s.id)))}>
            recolher tudo
          </button>{" "}
          <button type="button" onClick={() => setCollapsed(new Set())}>
            expandir tudo
          </button>
        </p>
        {sections.map((s) => (
          <SectionView
            key={s.id}
            section={s}
            collapsed={collapsed.has(s.id)}
            onToggle={() =>
              setCollapsed((prev) => {
                const next = new Set(prev);
                if (next.has(s.id)) next.delete(s.id);
                else next.add(s.id);
                return next;
              })
            }
          />
        ))}
        <h2>Resultados</h2>
        <pre className="code-block plain">
          <code>{results}</code>
        </pre>
      </article>
    </main>
  );
}

const root = document.getElementById("root");
if (!root) throw new Error("#root element not found");
createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
