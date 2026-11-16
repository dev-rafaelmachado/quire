// T9 spike (disposable): opens CodeMirror 6 on the synthetic spec of the v0.1 spike and
// measures load, mount and per-keystroke latency (transaction to the next painted frame).
// `?scale=N` (1 ≈ 40 KB), `?plan` runs scales 1, 1, 1, 12, 48 reloading itself, `?manual`
// stays open for the author's dead-key and IME script.
import { generateMarkdown } from "../spike/generate";

const params = new URLSearchParams(location.search);
const bar = document.getElementById("bar") as HTMLDivElement;
const host = document.getElementById("host") as HTMLDivElement;
const PLAN = [1, 1, 1, 12, 48];

function nextPaint(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => {
      const channel = new MessageChannel();
      channel.port1.onmessage = () => resolve();
      channel.port2.postMessage(null);
    });
  });
}

function percentile(values: number[], p: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))] ?? 0;
}

async function run(): Promise<void> {
  const step = Number(params.get("i") ?? "-1");
  const scale = step >= 0 ? (PLAN[step] ?? 1) : Number(params.get("scale") ?? "1");
  const doc = generateMarkdown(scale);
  const t0 = performance.now();
  const { mount } = await import("./editor");
  const tLoaded = performance.now();
  const view = mount(host, doc);
  await nextPaint();
  const tMounted = performance.now();

  // Continuous typing in the middle of the document, one transaction per character.
  const line = view.state.doc.line(Math.max(1, Math.floor(view.state.doc.lines / 2)));
  let pos = line.to;
  view.dispatch({ selection: { anchor: pos }, scrollIntoView: true });
  await nextPaint();
  const text = "Revisão da tarefa ação çãõ é ü ";
  const latencies: number[] = [];
  for (let k = 0; k < 200; k++) {
    const ch = text[k % text.length] ?? " ";
    const t = performance.now();
    view.dispatch({ changes: { from: pos, insert: ch }, selection: { anchor: pos + ch.length }, userEvent: "input.type" });
    await nextPaint();
    latencies.push(performance.now() - t);
    pos += ch.length;
  }
  const report = {
    spike: "T9-codemirror",
    scale,
    bytes: new TextEncoder().encode(doc).length,
    lines: view.state.doc.lines,
    loadMs: Math.round(tLoaded - t0),
    mountMs: Math.round(tMounted - tLoaded),
    typeP50: Number(percentile(latencies, 50).toFixed(1)),
    typeP95: Number(percentile(latencies, 95).toFixed(1)),
    typeMax: Number(Math.max(...latencies).toFixed(1)),
    ua: navigator.userAgent,
  };
  bar.textContent = JSON.stringify(report);
  await fetch("/__spike/report", { method: "POST", body: JSON.stringify(report) });
  if (step >= 0 && step + 1 < PLAN.length) location.search = `?i=${step + 1}`;
  if (params.has("manual")) view.focus();
}

void run();
