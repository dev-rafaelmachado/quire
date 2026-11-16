// Native window title (EDT-06): "• spec.md — Quire" while the buffer has unsaved changes.
// Outside Tauri (tests, plain browser) only document.title changes.
export function setWindowTitle(title: string): void {
  if (typeof document !== "undefined") document.title = title;
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;
  void import("@tauri-apps/api/window")
    .then(({ getCurrentWindow }) => getCurrentWindow().setTitle(title))
    .catch(() => {
      // Not fatal: the tab already shows the unsaved mark.
    });
}

/** Calls `ask(close)` instead of closing when `blocked()` (unsaved changes, SAV-06); `close`
 *  then destroys the window. Returns the cleanup. Outside Tauri, nothing happens. */
export function onWindowClose(blocked: () => boolean, ask: (close: () => void) => void): () => void {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return () => {};
  let unlisten: (() => void) | null = null;
  let disposed = false;
  void import("@tauri-apps/api/window")
    .then(({ getCurrentWindow }) => {
      const win = getCurrentWindow();
      return win.onCloseRequested((event) => {
        if (!blocked()) return;
        event.preventDefault();
        ask(() => void win.destroy());
      });
    })
    .then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    })
    .catch(() => {
      // Without the listener the window closes as usual.
    });
  return () => {
    disposed = true;
    unlisten?.();
  };
}
