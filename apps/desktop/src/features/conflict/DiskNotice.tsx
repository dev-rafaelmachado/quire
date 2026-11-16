import { TriangleAlert } from "lucide-react";
import { t } from "../../lib/i18n";
import { Button } from "../../ui";
import { useConflict } from "./store";

/** Fixed notice over the editor when the file changed or was deleted on disk while the buffer
 *  has unsaved changes. It never blocks the editing (CNF-05, CNF-07). */
export function DiskNotice() {
  const notice = useConflict((s) => s.notice);
  const openDialog = useConflict((s) => s.openDialog);
  if (!notice) return null;
  return (
    <div role="status" className="flex flex-none flex-wrap items-center gap-2 border-b border-line bg-warn-bg px-6 py-1.5 text-[12.5px] text-warn">
      <TriangleAlert aria-hidden="true" className="size-4 flex-none" />
      <span className="min-w-0 flex-1">{notice === "deleted" ? t("conflict.noticeDeleted") : t("conflict.noticeChanged")}</span>
      <Button onClick={() => void openDialog()}>{t("conflict.options")}</Button>
    </div>
  );
}
