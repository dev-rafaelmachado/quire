import { Lock, Save, TriangleAlert } from "lucide-react";
import { t } from "../../lib/i18n";
import { errorText } from "../../lib/i18n/errors";
import { Button, Chip, ErrorText, Segmented } from "../../ui";
import { guard } from "../unsaved/store";
import { type EditMode, useEditor } from "./store";

/** Ler / Editar / Dividir, Save, and the state of the buffer (EDT-01, EDT-05, EDT-06, EDT-08,
 *  SAV-03, SAV-04). */
export function EditBar() {
  const mode = useEditor((s) => s.mode);
  const buffer = useEditor((s) => s.buffer);
  const dirty = useEditor((s) => s.dirty);
  const saving = useEditor((s) => s.saving);
  const error = useEditor((s) => s.error);
  const confirming = useEditor((s) => s.confirmingMixed);
  const setMode = useEditor((s) => s.setMode);
  const save = useEditor((s) => s.save);
  const cancelMixed = useEditor((s) => s.cancelMixed);
  if (!buffer) return null;
  const locked = buffer.locked;
  const editing = mode !== "read";
  const eol = buffer.format?.eol === "crlf" ? t("editor.eolCrlf") : t("editor.eolLf");
  return (
    <div className="flex flex-none flex-col border-b border-line">
      <div className="flex min-h-11 flex-wrap items-center gap-2 px-6 py-1.5">
        <Segmented<EditMode>
          label={t("editor.modeLabel")}
          value={mode}
          // Leaving Editar/Dividir for Ler drops the editor: same guard as any navigation (SAV-06).
          onChange={(next) => (next === "read" ? guard(() => setMode(next)) : setMode(next))}
          options={[
            { value: "read", label: t("editor.read") },
            { value: "edit", label: t("editor.edit"), disabled: locked !== null },
            { value: "split", label: t("editor.split"), disabled: locked !== null },
          ]}
        />
        {locked ? (
          <span className="flex min-w-0 items-center gap-1.5 text-[12px] text-muted">
            <Lock aria-hidden="true" className="size-3.5 flex-none" />
            <span className="min-w-0">{t("editor.locked", { reason: errorText(locked) })}</span>
          </span>
        ) : null}
        {editing && dirty ? (
          <Chip tone="warn">{t("editor.dirty")}</Chip>
        ) : null}
        {editing && buffer.format?.mixed ? (
          <Chip tone="warn" icon={<TriangleAlert aria-hidden="true" className="size-3" />}>
            {t("editor.mixed")}
          </Chip>
        ) : null}
        {editing && buffer.readOnly ? <Chip>{t("editor.readOnly")}</Chip> : null}
        <div className="flex-1" />
        {editing ? (
          <Button onClick={() => void save()} disabled={!dirty || saving} title={t("editor.saveShortcut")} icon={<Save aria-hidden="true" className="size-3.5" />}>
            {saving ? t("editor.saving") : t("editor.save")}
          </Button>
        ) : null}
      </div>
      {confirming ? (
        <div role="alertdialog" aria-label={t("editor.mixed")} className="flex flex-wrap items-center gap-3 bg-warn-bg px-6 py-2 text-[12.5px] text-warn">
          <span className="min-w-0 flex-1">{t("editor.mixedConfirm", { eol })}</span>
          <Button onClick={() => void save({ confirmMixed: true })}>{t("editor.mixedSave")}</Button>
          <Button variant="ghost" onClick={cancelMixed}>
            {t("editor.cancel")}
          </Button>
        </div>
      ) : null}
      {error ? (
        <p role="alert" className="m-0 bg-warn-bg px-6 py-2 text-[12.5px] text-warn">
          <ErrorText error={error} />
        </p>
      ) : null}
    </div>
  );
}
