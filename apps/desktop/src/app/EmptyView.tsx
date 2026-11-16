import { FileText } from "lucide-react";
import { t } from "../lib/i18n";

/** Center column when no spec is open. */
export function EmptyView() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-2 text-center">
      <FileText aria-hidden="true" className="size-8 text-faint" />
      <p className="m-0 text-sm font-medium">{t("main.emptyTitle")}</p>
      <p className="m-0 text-[12.5px] text-muted">{t("main.emptyHint")}</p>
    </div>
  );
}
