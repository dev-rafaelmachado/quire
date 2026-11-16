// @vitest-environment jsdom
import { undo } from "@codemirror/commands";
import { describe, expect, it } from "vitest";
import { createEditor } from "./codemirror";

describe("CodeMirror buffer (EDT-06, EDT-07)", () => {
  it("tracks dirtiness against the saved text and undoes through a save", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dirty: boolean[] = [];
    let saves = 0;
    const h = createEditor(host, { text: "# Título\n", label: "x", onDirty: (d) => dirty.push(d), onSave: () => saves++ });
    h.view.dispatch({ changes: { from: h.view.state.doc.length, insert: "ação\n" }, userEvent: "input.type" });
    expect(h.getText()).toBe("# Título\nação\n");
    expect(dirty.at(-1)).toBe(true);
    // Saved: clean again, and the history survives the save (P3: memory only).
    h.markSaved(h.getText());
    expect(dirty.at(-1)).toBe(false);
    expect(undo(h.view)).toBe(true);
    expect(h.getText()).toBe("# Título\n");
    expect(dirty.at(-1)).toBe(true);
    // Typing back the saved text makes it clean.
    h.view.dispatch({ changes: { from: h.view.state.doc.length, insert: "ação\n" } });
    expect(dirty.at(-1)).toBe(false);
    // A reload is one undoable transaction.
    h.replaceText("do disco\n");
    h.markSaved("do disco\n");
    expect(dirty.at(-1)).toBe(false);
    expect(undo(h.view)).toBe(true);
    expect(h.getText()).toBe("# Título\nação\n");
    expect(saves).toBe(0);
    h.destroy();
  });
});
