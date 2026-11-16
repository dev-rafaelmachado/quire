// Line diff between the buffer and the disk (CNF-09), loaded on demand next to the editor chunk.
import { unifiedMergeView } from "@codemirror/merge";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

const theme = EditorView.theme({
  "&": { maxHeight: "50vh", backgroundColor: "var(--bg)", color: "var(--text)", fontSize: "12.5px" },
  ".cm-scroller": { fontFamily: "var(--font-mono)", lineHeight: "1.55" },
  ".cm-content": { padding: "8px 12px" },
  ".cm-changedLine": { backgroundColor: "color-mix(in srgb, var(--ok-bg) 70%, transparent)" },
  ".cm-deletedChunk": { backgroundColor: "color-mix(in srgb, var(--warn-bg) 80%, transparent)" },
  ".cm-insertedLine, .cm-changedText": { backgroundColor: "var(--ok-bg)" },
  ".cm-deletedLine, .cm-deletedText": { backgroundColor: "var(--warn-bg)" },
});

/** A read-only view of the buffer with the lines that differ from the disk marked. */
export function createDiff(parent: HTMLElement, buffer: string, disk: string, label: string): () => void {
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: buffer,
      extensions: [
        EditorState.readOnly.of(true),
        EditorView.editable.of(false),
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({ "aria-label": label }),
        unifiedMergeView({ original: disk, mergeControls: false, highlightChanges: true, gutter: true }),
        theme,
      ],
    }),
  });
  return () => view.destroy();
}
