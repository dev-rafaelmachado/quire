import { useEffect, useRef } from "react";
import { t } from "../../lib/i18n";
import { Button, ErrorText } from "../../ui";
import { useEditor } from "../editor/store";
import { useConflict } from "./store";

function DiffView({ buffer, disk }: { buffer: string; disk: string }) {
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = host.current;
    if (!el) return;
    let destroy: (() => void) | null = null;
    let cancelled = false;
    void import("./diff").then(({ createDiff }) => {
      if (!cancelled) destroy = createDiff(el, buffer, disk, t("conflict.diffLabel"));
    });
    return () => {
      cancelled = true;
      destroy?.();
    };
  }, [buffer, disk]);
  return <div ref={host} className="overflow-hidden rounded-[var(--radius-md)] border border-line" />;
}

/** The file changed (or vanished) on disk under unsaved edits: Recarregar, Sobrescrever, Salvar
 *  como cópia or Ver diferenças, and nothing is written before the choice (CNF-02, CNF-07..09). */
export function ConflictDialog() {
  const open = useConflict((s) => s.open);
  const disk = useConflict((s) => s.disk);
  const busy = useConflict((s) => s.busy);
  const error = useConflict((s) => s.error);
  const copied = useConflict((s) => s.copied);
  const showDiff = useConflict((s) => s.showDiff);
  const { close, reload, overwrite, saveCopy, toggleDiff } = useConflict.getState();
  const buffer = useEditor((s) => s.buffer);
  const getText = useEditor((s) => s.getText);
  const first = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    first.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        close();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, close]);

  if (!open || !buffer) return null;
  const deleted = disk !== null && disk.text === null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-4">
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="conflict-title"
        aria-describedby="conflict-body"
        className="flex max-h-[90vh] w-full max-w-[720px] flex-col gap-4 overflow-y-auto rounded-[var(--radius-lg)] border border-line bg-panel p-5 shadow-xl"
      >
        <h2 id="conflict-title" className="m-0 text-[15px] font-semibold">
          {deleted ? t("conflict.deletedTitle", { name: buffer.name }) : t("conflict.title", { name: buffer.name })}
        </h2>
        <div id="conflict-body" className="flex flex-col gap-2 text-[13px] text-muted">
          <p className="m-0">{deleted ? t("conflict.deletedBody") : t("conflict.body")}</p>
          {buffer.format?.mixed ? <p className="m-0">{t("conflict.mixed")}</p> : null}
        </div>
        {copied ? (
          <p role="status" className="m-0 rounded-[var(--radius-md)] bg-ok-bg px-3 py-2 text-[12.5px] text-ok">
            {t("conflict.copied", { name: copied })}
          </p>
        ) : null}
        {error ? (
          <p role="alert" className="m-0 rounded-[var(--radius-md)] bg-warn-bg px-3 py-2 text-[12.5px] text-warn">
            <ErrorText error={error} />
          </p>
        ) : null}
        {showDiff && disk && disk.text !== null && getText ? (
          <div className="flex flex-col gap-1.5">
            <p className="m-0 flex flex-wrap gap-3 text-[11.5px] text-muted">
              <span className="flex items-center gap-1.5">
                <span aria-hidden="true" className="size-2.5 rounded-sm bg-warn-bg ring-1 ring-warn" />
                {t("conflict.legendDisk")}
              </span>
              <span className="flex items-center gap-1.5">
                <span aria-hidden="true" className="size-2.5 rounded-sm bg-ok-bg ring-1 ring-ok" />
                {t("conflict.legendEditor")}
              </span>
            </p>
            <DiffView buffer={getText()} disk={disk.text} />
          </div>
        ) : null}
        <div className="flex flex-wrap justify-end gap-2">
          <Button ref={first} variant="ghost" onClick={close} disabled={busy}>
            {t("conflict.keepEditing")}
          </Button>
          {!deleted ? (
            <Button onClick={toggleDiff} disabled={busy || disk === null} aria-pressed={showDiff}>
              {showDiff ? t("conflict.hideDiff") : t("conflict.showDiff")}
            </Button>
          ) : null}
          <Button onClick={() => void saveCopy()} disabled={busy}>
            {t("conflict.saveCopy")}
          </Button>
          {!deleted ? (
            <Button onClick={() => void reload()} disabled={busy}>
              {t("conflict.reload")}
            </Button>
          ) : null}
          <Button onClick={() => void overwrite()} disabled={busy}>
            {deleted ? t("conflict.recreate") : t("conflict.overwrite")}
          </Button>
        </div>
      </div>
    </div>
  );
}
