// The editor chunk: everything CodeMirror the app would load on demand (EDT-02).
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { leanMarkdown } from "./lean";
import { defaultHighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { highlightSelectionMatches, search, searchKeymap } from "@codemirror/search";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";

export function mount(parent: HTMLElement, doc: string): EditorView {
  return new EditorView({
    parent,
    state: EditorState.create({
      doc,
      extensions: [
        lineNumbers(),
        history(),
        search({ top: true }),
        highlightSelectionMatches(),
        leanMarkdown,
        syntaxHighlighting(defaultHighlightStyle),
        EditorView.lineWrapping,
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
        EditorView.theme({ "&": { color: "#e6e8ee", backgroundColor: "#111318" }, ".cm-content": { caretColor: "#8ab4ff" } }, { dark: true }),
      ],
    }),
  });
}
