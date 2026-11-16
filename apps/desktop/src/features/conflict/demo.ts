// `?demo=conflict` and `?demo=notice` (screenshots only): once the editor is ready, an edit in
// memory and a pretend change on disk, set in the UI state only. Nothing reaches the backend.
import { currentEditor } from "../editor/EditorPane";
import { useEditor } from "../editor/store";
import { useConflict } from "./store";

export function startConflictDemo(): void {
  if (typeof window === "undefined") return;
  const demo = new URLSearchParams(window.location.search).get("demo");
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
