import { t } from "../i18n";
import { cx } from "../../ui";
import type { OutlineEntry } from "./pipeline";

interface OutlineProps {
  entries: readonly OutlineEntry[];
  active: string | null;
  onNavigate: (id: string) => void;
}

/** "Neste arquivo": headings of the file, the visible one highlighted (VIW-05). */
export function Outline({ entries, active, onNavigate }: OutlineProps) {
  return (
    <nav aria-label={t("markdown.outlineLabel")} className="flex min-h-0 flex-col gap-px overflow-y-auto px-2 pt-3.5 pb-2">
      <h2 className="m-0 px-3 pb-2 text-[11px] font-semibold tracking-[.06em] text-faint uppercase">{t("markdown.outline")}</h2>
      {entries.map((e) => (
        <button
          key={e.id}
          type="button"
          aria-current={e.id === active ? "location" : undefined}
          onClick={() => onNavigate(e.id)}
          className={cx(
            "block w-full truncate rounded-[var(--radius-md)] py-[5px] pr-3 text-left text-[12.5px]",
            e.depth >= 3 ? "pl-7" : "pl-3.5",
            e.id === active ? "bg-acc-bg font-semibold text-acc" : "text-muted hover:bg-raise hover:text-text",
          )}
        >
          {e.title}
        </button>
      ))}
    </nav>
  );
}
