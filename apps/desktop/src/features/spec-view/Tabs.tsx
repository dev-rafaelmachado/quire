import { FolderOpen } from "lucide-react";
import { t } from "../../lib/i18n";
import type { SpecDetail } from "../../lib/ipc";
import { cx } from "../../ui";
import { displayed, tabLabel } from "./labels";
import { OTHERS, type TabCounts } from "./store";

function count(file: string, detail: SpecDetail, counts: TabCounts): string | null {
  if (file === "tasks.md") {
    const p = displayed(detail)?.progress;
    return p ? `${p[0]}/${p[1]}` : null;
  }
  if (file === "gates.jsonl" && counts.gates !== undefined) return String(counts.gates);
  if (file === "lessons.jsonl" && counts.lessons !== undefined) return String(counts.lessons);
  return null;
}

interface TabsProps {
  detail: SpecDetail;
  counts: TabCounts;
  tab: string | null;
  onSelect: (tab: string) => void;
}

/** One tab per primary file that exists, then "outros" (VIW-01). */
export function Tabs({ detail, counts, tab, onSelect }: TabsProps) {
  const tabClass = (selected: boolean) =>
    cx(
      "-mb-px flex h-10 flex-none items-center gap-1.5 border-b-2 px-3",
      selected ? "border-acc font-semibold text-text" : "border-transparent font-medium text-muted hover:text-text",
    );
  return (
    <div role="tablist" aria-label={t("specView.tabsLabel")} className="flex h-[41px] flex-none items-center gap-0.5 overflow-x-auto border-b border-line px-4">
      {detail.tabs.map((file) => {
        const n = count(file, detail, counts);
        return (
          <button key={file} type="button" role="tab" aria-selected={tab === file} onClick={() => onSelect(file)} className={tabClass(tab === file)}>
            {tabLabel(file)}
            {n !== null ? <span className="font-code text-[11px] text-faint">{n}</span> : null}
          </button>
        );
      })}
      {detail.others.length > 0 ? (
        <>
          {detail.tabs.length > 0 ? <span aria-hidden="true" className="mx-2 h-4 w-px flex-none bg-line" /> : null}
          <button
            type="button"
            role="tab"
            aria-selected={tab === OTHERS}
            aria-label={`${t("specView.othersLabel")} (${detail.others.length})`}
            onClick={() => onSelect(OTHERS)}
            className={tabClass(tab === OTHERS)}
          >
            <FolderOpen aria-hidden="true" className="size-3.5" />
            {t("specView.others")}
            <span className="font-code text-[11px] text-faint">{detail.others.length}</span>
          </button>
        </>
      ) : null}
    </div>
  );
}
