import { useEffect, useRef, useState } from "react";
import { t } from "../../lib/i18n";
import { setWindowTitle } from "../../lib/window";
import { guard } from "../unsaved/store";
import type { EditorHandle } from "./codemirror";
import { useEditor } from "./store";

let current: EditorHandle | null = null;

/** The mounted editor, if any (reload from disk and scroll sync use it). */
export function currentEditor(): EditorHandle | null {
  return current;
}

/** CodeMirror for the buffer of the center column, loaded on first use (EDT-02). One editor
 *  per file: switching files mounts a new one with its own in-memory history (EDT-07). */
export function EditorPane({ onChange, onReady }: { onChange?: () => void; onReady?: (h: EditorHandle | null) => void }) {
  const buffer = useEditor((s) => s.buffer);
  const savedVersion = useEditor((s) => s.savedVersion);
  const host = useRef<HTMLDivElement>(null);
  const handle = useRef<EditorHandle | null>(null);
  const [ready, setReady] = useState(false);
  const loaded = buffer !== null && buffer.format !== null && buffer.locked === null;
  const key = buffer ? `${buffer.instanceId}\u0000${buffer.name}` : "";
  const changeRef = useRef(onChange);
  changeRef.current = onChange;
  const readyRef = useRef(onReady);
  readyRef.current = onReady;

  useEffect(() => {
    const el = host.current;
    const b = useEditor.getState().buffer;
    if (!el || !loaded || !b) return;
    let cancelled = false;
    const mountedAt = performance.now();
    void import("./codemirror").then(({ createEditor, typingBenchmark }) => {
      if (cancelled) return;
      const h = createEditor(el, {
        text: b.initialText ?? b.savedText,
        saved: b.savedText,
        label: t("editor.textOf", { name: b.name }),
        onDirty: (dirty) => useEditor.getState().setDirty(dirty),
        onChange: () => changeRef.current?.(),
        onSave: () => void useEditor.getState().save(),
      });
      handle.current = h;
      current = h;
      useEditor.getState().setGetText(h.getText);
      setReady(true);
      readyRef.current?.(h);
      h.focus();
      const params = new URLSearchParams(window.location.search);
      // `?demo=unsaved` (screenshots only): an edit in memory, then the guard. Nothing is saved.
      if (params.get("demo") === "unsaved") {
        h.view.dispatch({ changes: { from: h.view.state.doc.length, insert: "\nRascunho ainda não salvo.\n" } });
        guard(() => undefined);
      }
      const bench = params.get("bench");
      if (bench === "type" || bench === "preview") {
        const t0 = performance.now();
        void typingBenchmark(h.view, (r) => setWindowTitle(`${r} open=${(t0 - mountedAt).toFixed(0)}`));
      }
    });
    return () => {
      cancelled = true;
      readyRef.current?.(null);
      handle.current?.destroy();
      if (current === handle.current) current = null;
      handle.current = null;
      useEditor.getState().setGetText(null);
      setReady(false);
    };
  }, [key, loaded]);

  // After a save (or a reload), "clean" means the new saved text.
  useEffect(() => {
    const b = useEditor.getState().buffer;
    if (b && handle.current) handle.current.markSaved(b.savedText);
  }, [savedVersion]);

  return (
    <div className="relative h-full min-h-0">
      {!ready ? <p className="absolute top-6 left-10 m-0 text-muted">{t("editor.loading")}</p> : null}
      <div ref={host} className="h-full" data-testid="editor-host" />
    </div>
  );
}
