// The editor chunk (TD-11, EDT-02): everything CodeMirror lives here and is loaded with a
// dynamic import the first time the user enters Editar or Dividir, never in the initial bundle.
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { highlightSelectionMatches, search, searchKeymap } from "@codemirror/search";
import { EditorState, Prec, type Text } from "@codemirror/state";
import { drawSelection, EditorView, highlightActiveLine, keymap } from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { leanMarkdown } from "./markdownLanguage";

export interface EditorHandle {
  /** The current text (LF line endings). */
  getText: () => string;
  /** Replaces the whole text as one undoable transaction (a reload from disk, CNF-04). */
  replaceText: (text: string) => void;
  /** The text the buffer now matches on disk: dirtiness is measured against it. */
  markSaved: (text: string) => void;
  /** Top visible line (1-based) and the line at a vertical offset (sync scroll, T17). */
  view: EditorView;
  focus: () => void;
  destroy: () => void;
}

export interface EditorOptions {
  text: string;
  /** Called on every change with whether the text differs from the saved one. */
  onDirty: (dirty: boolean) => void;
  /** Called after each change, for the live preview (T17). */
  onChange?: () => void;
  onSave: () => void;
  /** Accessible name of the text area. */
  label: string;
}

// Colors come from the design tokens, so the dark and light themes both apply (TD-09).
const theme = EditorView.theme({
  "&": { height: "100%", backgroundColor: "var(--bg)", color: "var(--text)", fontSize: "13.5px" },
  ".cm-scroller": { fontFamily: "var(--font-mono)", lineHeight: "1.65", padding: "24px 0 96px" },
  ".cm-content": { caretColor: "var(--acc)", padding: "0 40px", maxWidth: "860px" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--acc)" },
  "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, ::selection": {
    backgroundColor: "var(--acc-bg) !important",
  },
  ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--raise) 45%, transparent)" },
  ".cm-selectionMatch": { backgroundColor: "color-mix(in srgb, var(--acc-bg) 70%, transparent)" },
  "&.cm-focused": { outline: "none" },
  ".cm-panels": { backgroundColor: "var(--panel)", color: "var(--text)", borderColor: "var(--line)" },
  ".cm-panels.cm-panels-top": { borderBottom: "1px solid var(--line)" },
  ".cm-searchMatch": { backgroundColor: "var(--warn-bg)", outline: "1px solid var(--warn)" },
  ".cm-searchMatch.cm-searchMatch-selected": { backgroundColor: "var(--acc-bg)" },
  ".cm-textfield": {
    backgroundColor: "var(--bg)",
    color: "var(--text)",
    border: "1px solid var(--line)",
    borderRadius: "var(--radius-sm)",
  },
  ".cm-button": {
    backgroundImage: "none",
    backgroundColor: "var(--raise)",
    color: "var(--text)",
    border: "1px solid var(--line)",
    borderRadius: "var(--radius-sm)",
  },
  ".cm-panel button[name=close]": { color: "var(--muted)" },
});

const highlight = HighlightStyle.define([
  { tag: tags.heading1, fontWeight: "700", fontSize: "1.25em", color: "var(--text)" },
  { tag: tags.heading2, fontWeight: "700", fontSize: "1.12em", color: "var(--text)" },
  { tag: [tags.heading3, tags.heading4, tags.heading5, tags.heading6], fontWeight: "700", color: "var(--text)" },
  { tag: tags.strong, fontWeight: "700" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strikethrough, textDecoration: "line-through" },
  { tag: [tags.link, tags.url], color: "var(--acc)" },
  { tag: tags.monospace, color: "var(--ok)" },
  { tag: tags.quote, color: "var(--muted)" },
  // Marks only (#, *, -, >, ```): `tags.list` would grey out the whole list item.
  { tag: [tags.processingInstruction, tags.contentSeparator], color: "var(--faint)" },
  { tag: tags.meta, color: "var(--faint)" },
  { tag: tags.comment, color: "var(--faint)", fontStyle: "italic" },
]);

export function createEditor(parent: HTMLElement, options: EditorOptions): EditorHandle {
  let saved: Text | null = null;
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: options.text,
      extensions: [
        history(),
        drawSelection(),
        highlightActiveLine(),
        search({ top: true }),
        highlightSelectionMatches(),
        leanMarkdown,
        syntaxHighlighting(highlight),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({ "aria-label": options.label, "aria-multiline": "true" }),
        theme,
        // Ctrl+S saves (EDT-05) ahead of every other binding; Ctrl+K is left alone so the
        // command palette opens from inside the editor (CMD-01).
        Prec.highest(
          keymap.of([
            {
              key: "Mod-s",
              preventDefault: true,
              run: () => {
                options.onSave();
                return true;
              },
            },
          ]),
        ),
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, indentWithTab]),
        EditorView.updateListener.of((u) => {
          if (!u.docChanged) return;
          options.onDirty(saved === null ? false : !u.state.doc.eq(saved));
          options.onChange?.();
        }),
      ],
    }),
  });
  saved = view.state.doc;
  return {
    view,
    getText: () => view.state.doc.toString(),
    replaceText: (text) => {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text }, userEvent: "input.reload" });
    },
    markSaved: (text) => {
      saved = text === view.state.doc.toString() ? view.state.doc : EditorState.create({ doc: text }).doc;
      options.onDirty(!view.state.doc.eq(saved));
    },
    focus: () => view.focus(),
    destroy: () => view.destroy(),
  };
}

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
