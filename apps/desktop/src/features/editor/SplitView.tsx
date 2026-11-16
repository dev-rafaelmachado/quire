import { useCallback, useEffect, useRef, useState } from "react";
import { setWindowTitle } from "../../lib/window";
import type { EditorHandle } from "./codemirror";
import { EditorPane } from "./EditorPane";
import { PreviewPane } from "./PreviewPane";
import { useEditor } from "./store";
import { type Anchor, cleanAnchors, lineForTop, topForLine } from "./syncScroll";

/** Pause in typing before the preview follows (EDT-03). */
export const PREVIEW_DEBOUNCE_MS = 200;

function anchorsOf(container: HTMLElement): Anchor[] {
  const base = container.getBoundingClientRect().top - container.scrollTop;
  const out: Anchor[] = [];
  for (const el of container.querySelectorAll<HTMLElement>("[data-line]")) {
    if (el.offsetParent === null) continue; // inside a collapsed section
    out.push({ line: Number(el.dataset["line"]), top: el.getBoundingClientRect().top - base });
  }
  return cleanAnchors(out);
}

/** Dividir: the editor and the live preview side by side, scrolling together without echo
 *  (EDT-03, EDT-04). Only the side the user scrolls drives the other. */
export function SplitView({ name, header }: { name: string; header: boolean }) {
  const saved = useEditor((s) => s.buffer?.savedText ?? "");
  const [text, setText] = useState(saved);
  const editor = useRef<EditorHandle | null>(null);
  const preview = useRef<HTMLElement | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastChange = useRef(0);
  const firedAt = useRef(0);
  // Offsets we set ourselves; the scroll event they cause must not drive the other side.
  const expected = useRef<{ editor: number | null; preview: number | null }>({ editor: null, preview: null });

  useEffect(() => setText(saved), [saved]);
  useEffect(() => () => {
    if (timer.current) clearTimeout(timer.current);
  }, []);

  const onChange = useCallback(() => {
    lastChange.current = performance.now();
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      timer.current = null;
      const h = editor.current;
      firedAt.current = performance.now();
      if (h) setText(h.getText());
    }, PREVIEW_DEBOUNCE_MS);
  }, []);

  // `?bench=preview` (measurement only): parse + render time of the preview after the pause;
  // the delay seen by the user is that plus PREVIEW_DEBOUNCE_MS.
  useEffect(() => {
    if (firedAt.current === 0 || new URLSearchParams(window.location.search).get("bench") !== "preview") return;
    setWindowTitle(`bench preview render=${(performance.now() - firedAt.current).toFixed(0)} bytes=${text.length}`);
  }, [text]);

  const syncFromEditor = useCallback(() => {
    const h = editor.current;
    const p = preview.current;
    if (!h || !p) return;
    const scroller = h.view.scrollDOM;
    if (expected.current.editor !== null && Math.abs(scroller.scrollTop - expected.current.editor) < 2) {
      expected.current.editor = null;
      return;
    }
    expected.current.editor = null;
    const pad = h.view.documentPadding.top;
    const y = Math.max(0, scroller.scrollTop - pad);
    const block = h.view.lineBlockAtHeight(y);
    const line = h.view.state.doc.lineAt(block.from).number + (block.height > 0 ? (y - block.top) / block.height : 0);
    const top = topForLine(anchorsOf(p), line);
    if (top === null) return;
    const target = Math.max(0, Math.round(top));
    if (Math.abs(p.scrollTop - target) < 2) return;
    expected.current.preview = target;
    p.scrollTop = target;
  }, []);

  const syncFromPreview = useCallback(() => {
    const h = editor.current;
    const p = preview.current;
    if (!h || !p) return;
    if (expected.current.preview !== null && Math.abs(p.scrollTop - expected.current.preview) < 2) {
      expected.current.preview = null;
      return;
    }
    expected.current.preview = null;
    const line = lineForTop(anchorsOf(p), p.scrollTop);
    if (line === null) return;
    const doc = h.view.state.doc;
    const n = Math.min(doc.lines, Math.max(1, Math.floor(line)));
    const block = h.view.lineBlockAt(doc.line(n).from);
    const target = Math.max(0, Math.round(block.top + (line - n) * block.height + h.view.documentPadding.top));
    const scroller = h.view.scrollDOM;
    if (Math.abs(scroller.scrollTop - target) < 2) return;
    expected.current.editor = target;
    scroller.scrollTop = target;
  }, []);

  const onReady = useCallback(
    (h: EditorHandle | null) => {
      editor.current?.view.scrollDOM.removeEventListener("scroll", syncFromEditor);
      editor.current = h;
      h?.view.scrollDOM.addEventListener("scroll", syncFromEditor, { passive: true });
    },
    [syncFromEditor],
  );

  const previewRef = useCallback(
    (el: HTMLElement | null) => {
      preview.current?.removeEventListener("scroll", syncFromPreview);
      preview.current = el;
      el?.addEventListener("scroll", syncFromPreview, { passive: true });
    },
    [syncFromPreview],
  );

  return (
    <div className="grid h-full grid-cols-2">
      <EditorPane onChange={onChange} onReady={onReady} />
      <PreviewPane name={name} text={text} header={header} scrollRef={previewRef} />
    </div>
  );
}
