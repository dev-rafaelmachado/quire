import { Info, X } from "lucide-react";
import { t } from "../../lib/i18n";
import { IconButton } from "../../ui";
import { useSpecView } from "./store";

/** Message about a change made outside the app, e.g. the open copy was removed (WAT-04). */
export function Notice() {
  const notice = useSpecView((s) => s.notice);
  const setNotice = useSpecView((s) => s.setNotice);
  if (!notice) return null;
  return (
    <div role="status" className="flex flex-none items-center gap-2 border-b border-line bg-acc-bg px-6 py-1.5 text-[12.5px] text-acc">
      <Info aria-hidden="true" className="size-4 flex-none" />
      <span className="flex-1">{notice}</span>
      <IconButton label={t("live.dismiss")} icon={<X className="size-3.5" />} onClick={() => setNotice(null)} className="size-6 text-acc" />
    </div>
  );
}
