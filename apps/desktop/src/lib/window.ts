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
