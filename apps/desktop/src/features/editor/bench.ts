// Typing latency measurement (T16), loaded only by the development hooks.
import type { EditorView } from "@codemirror/view";

/** `?bench=type` (measurement only, T16): types 200 characters in the middle of the document,
 *  one transaction each, and reports the time to the next painted frame in the window title,
 *  where `hyprctl clients` can read it. */
export async function typingBenchmark(view: EditorView, report: (text: string) => void): Promise<void> {
  const nextPaint = () =>
    new Promise<void>((resolve) =>
      requestAnimationFrame(() => {
        const ch = new MessageChannel();
        ch.port1.onmessage = () => resolve();
        ch.port2.postMessage(null);
      }),
    );
  const line = view.state.doc.line(Math.max(1, Math.floor(view.state.doc.lines / 2)));
  let pos = line.to;
  view.dispatch({ selection: { anchor: pos }, scrollIntoView: true });
  await nextPaint();
  const sample = "Revisão ação çãõ é ü ";
  const times: number[] = [];
  for (let k = 0; k < 200; k++) {
    const ch = sample[k % sample.length] ?? " ";
    const t0 = performance.now();
    view.dispatch({ changes: { from: pos, insert: ch }, selection: { anchor: pos + ch.length }, userEvent: "input.type" });
    await nextPaint();
    times.push(performance.now() - t0);
    pos += ch.length;
  }
  times.sort((a, b) => a - b);
  const p = (q: number) => (times[Math.min(times.length - 1, Math.floor(q * times.length))] ?? 0).toFixed(1);
  report(`bench bytes=${view.state.doc.length} p50=${p(0.5)} p95=${p(0.95)} max=${(times.at(-1) ?? 0).toFixed(1)}`);
}
