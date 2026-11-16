import { useEffect, useRef } from "react";
import { t } from "../../lib/i18n";
import { Button, ErrorText } from "../../ui";
import { useEditor } from "../editor/store";
import { useUnsaved } from "./store";

/** Salvar / Descartar / Cancelar before leaving a buffer with unsaved changes (SAV-06). */
export function UnsavedDialog() {
  const pending = useUnsaved((s) => s.pending);
  const busy = useUnsaved((s) => s.busy);
  const error = useUnsaved((s) => s.error);
  const needsMixed = useUnsaved((s) => s.needsMixed);
  const choose = useUnsaved((s) => s.choose);
  const name = useEditor((s) => s.buffer?.name ?? "");
  const first = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!pending) return;
    first.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void choose("cancel");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [pending, choose]);

  if (!pending) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-4">
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="unsaved-title"
        aria-describedby="unsaved-body"
        className="flex w-full max-w-[460px] flex-col gap-4 rounded-[var(--radius-lg)] border border-line bg-panel p-5 shadow-xl"
      >
        <h2 id="unsaved-title" className="m-0 text-[15px] font-semibold">
          {t("unsaved.title", { name })}
        </h2>
        <p id="unsaved-body" className="m-0 text-[13px] text-muted">
          {needsMixed ? t("unsaved.mixed") : t("unsaved.body")}
        </p>
        {error ? (
          <p role="alert" className="m-0 rounded-[var(--radius-md)] bg-warn-bg px-3 py-2 text-[12.5px] text-warn">
            <ErrorText error={error} />
          </p>
        ) : null}
        <div className="flex flex-wrap justify-end gap-2">
          <Button ref={first} variant="ghost" onClick={() => void choose("cancel")} disabled={busy}>
            {t("unsaved.cancel")}
          </Button>
          <Button onClick={() => void choose("discard")} disabled={busy}>
            {t("unsaved.discard")}
          </Button>
          <Button onClick={() => void choose("save", { confirmMixed: needsMixed })} disabled={busy}>
            {busy ? t("editor.saving") : needsMixed ? t("editor.mixedSave") : t("unsaved.save")}
          </Button>
        </div>
      </div>
    </div>
  );
}
