// Screenshot demos and measurements, for development builds only. Every import of this module
// sits behind `import.meta.env.DEV`, so production bundles never contain it. Demos change the
// UI state in memory only: nothing reaches the backend and nothing is written.
import { currentEditor } from "../features/editor/EditorPane";
import type { EditorHandle } from "../features/editor/codemirror";
import { useEditor } from "../features/editor/store";
import { useConflict } from "../features/conflict/store";
import { guard } from "../features/unsaved/store";
import { devParam } from "../lib/devParams";
import { setWindowTitle } from "../lib/window";

/** Called once the editor is mounted: `?demo=unsaved` and `?bench=type|preview`. */
export function onEditorReady(h: EditorHandle, mountedAt: number): void {
  if (devParam("demo") === "unsaved") {
    h.view.dispatch({ changes: { from: h.view.state.doc.length, insert: "\nRascunho ainda não salvo.\n" } });
    guard(() => undefined);
  }
  const bench = devParam("bench");
  if (bench === "type" || bench === "preview") {
    const t0 = performance.now();
    void import("../features/editor/bench").then(({ typingBenchmark }) =>
      typingBenchmark(h.view, (r) => setWindowTitle(`${r} open=${(t0 - mountedAt).toFixed(0)}`)),
    );
  }
}

/** `?demo=conflict` and `?demo=notice`: an edit in memory and a pretend change on disk. */
export function startDevHooks(): void {
  const demo = devParam("demo");
  if (demo !== "conflict" && demo !== "notice") return;
  const unsubscribe = useEditor.subscribe((s) => {
    const h = currentEditor();
    if (!s.getText || !h || !s.buffer?.format) return;
    unsubscribe();
    const saved = s.buffer.savedText;
    h.view.dispatch({ changes: { from: h.view.state.doc.length, insert: "\nMinha nota ainda não salva.\n" } });
    const agent = saved.replace("## Problema", "## Problema\n\nParágrafo que o agente acrescentou.");
    if (demo === "notice") useConflict.setState({ notice: "changed" });
    else useConflict.setState({ open: true, showDiff: true, disk: { text: agent, fingerprint: "0-demo", format: s.buffer.format } });
  });
}
