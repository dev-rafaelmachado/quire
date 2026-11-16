import { Command, FileText, Search } from "lucide-react";
import { type KeyboardEvent, useEffect, useMemo, useRef, useState } from "react";
import { t } from "../../lib/i18n";
import { useEditor } from "../editor/store";
import { Snippet } from "../search/Snippet";
import { useSpecView } from "../spec-view/store";
import { useWorkspace } from "../workspace/store";
import { useSearch } from "../search/store";
import { rank } from "./fuzzy";
import { type PaletteItem, paletteItems } from "./items";
import { SEARCH_PREFIX, usePalette } from "./store";

const SHOWN = 50;

interface Row {
  id: string;
  label: string;
  hint?: string;
  snippet?: string;
  icon: "spec" | "command" | "hit";
  run: () => void;
}

function rows(query: string, items: PaletteItem[], recent: string[]): Row[] {
  const toRow = (i: PaletteItem): Row => ({ id: i.id, label: i.label, ...(i.hint ? { hint: i.hint } : {}), icon: i.group, run: i.run });
  if (query.trim() === "") {
    const byId = new Map(items.map((i) => [i.id, i]));
    const first = recent.flatMap((id) => (byId.has(id) ? [byId.get(id) as PaletteItem] : []));
    const rest = items.filter((i) => !recent.includes(i.id));
    return [...first, ...rest].slice(0, SHOWN).map(toRow);
  }
  return items
    .map((i) => ({ i, s: rank(query, i) }))
    .filter((x): x is { i: PaletteItem; s: number } => x.s !== null)
    .sort((a, b) => b.s - a.s)
    .slice(0, SHOWN)
    .map((x) => toRow(x.i));
}

/** Ctrl+K: specs and commands by keyboard, and the content search after "/" (CMD-01..05). */
export function Palette() {
  const open = usePalette((s) => s.open);
  const query = usePalette((s) => s.query);
  const recent = usePalette((s) => s.recent);
  const { setOpen, setQuery, remember } = usePalette.getState();
  const results = useSearch((s) => s.results);
  // The items follow what is loaded: specs, the open spec, the editor.
  const projects = useWorkspace((s) => s.projects);
  const detail = useSpecView((s) => s.detail);
  const buffer = useEditor((s) => s.buffer);
  const mode = useEditor((s) => s.mode);
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const searching = query.startsWith(SEARCH_PREFIX);

  useEffect(() => {
    if (open) input.current?.focus();
  }, [open]);
  useEffect(() => setActive(0), [query]);
  // Content search in the same panel (CMD-03).
  useEffect(() => {
    if (open && searching) useSearch.getState().setText(query.slice(SEARCH_PREFIX.length));
  }, [open, searching, query]);

  const list: Row[] = useMemo(() => {
    if (!open) return [];
    if (searching) {
      return (results?.groups ?? []).flatMap((g) =>
        g.hits.map((h) => ({
          id: `hit:${h.instanceId}:${h.file}:${h.line}`,
          label: `${g.issueKey ?? g.folder} · ${h.file}${h.heading ? ` · ${h.heading}` : ""}`,
          ...(h.branch ? { hint: h.branch } : {}),
          snippet: h.snippet,
          icon: "hit" as const,
          run: () => useSearch.getState().openHit(g, h),
        })),
      ).slice(0, SHOWN);
    }
    return rows(query, paletteItems(), recent);
    // paletteItems() reads these stores; listing them keeps the items current.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, searching, query, recent, results, projects, detail, buffer, mode]);

  if (!open) return null;
  const close = () => setOpen(false);
  const runRow = (row: Row) => {
    if (row.id === "search") {
      setQuery(SEARCH_PREFIX);
      return;
    }
    if (row.icon !== "hit") remember(row.id);
    close();
    row.run();
  };
  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(list.length - 1, a + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(0, a - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const row = list[active];
      if (row) runRow(row);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    }
  };
  const activeId = list[active] ? `palette-opt-${active}` : undefined;
  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center bg-black/30 px-4 pt-[12vh]" onMouseDown={(e) => e.target === e.currentTarget && close()}>
      <div role="dialog" aria-modal="true" aria-label={t("palette.title")} className="flex max-h-[70vh] w-full max-w-[640px] flex-col overflow-hidden rounded-[var(--radius-lg)] border border-line bg-panel shadow-2xl">
        <div className="flex items-center gap-2 border-b border-line px-3">
          {searching ? <Search aria-hidden="true" className="size-4 text-faint" /> : <Command aria-hidden="true" className="size-4 text-faint" />}
          <input
            ref={input}
            role="combobox"
            aria-expanded="true"
            aria-controls="palette-list"
            aria-autocomplete="list"
            aria-activedescendant={activeId}
            aria-label={t("palette.input")}
            placeholder={t("palette.placeholder")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKey}
            className="h-11 min-w-0 flex-1 bg-transparent text-[14px] outline-none"
          />
        </div>
        <ul id="palette-list" role="listbox" aria-label={searching ? t("palette.results") : t("palette.items")} className="m-0 min-h-0 flex-1 list-none overflow-y-auto p-1.5">
          {list.length === 0 ? (
            <li role="presentation" className="px-3 py-2 text-[12.5px] text-muted">
              {searching ? (query.length > SEARCH_PREFIX.length ? t("search.none") : t("palette.searchHint")) : t("palette.none")}
            </li>
          ) : null}
          {list.map((row, i) => (
            <li
              key={row.id}
              id={`palette-opt-${i}`}
              role="option"
              aria-selected={i === active}
              onMouseEnter={() => setActive(i)}
              onMouseDown={(e) => {
                e.preventDefault();
                runRow(row);
              }}
              className={`flex cursor-pointer flex-col gap-0.5 rounded-[var(--radius-md)] px-3 py-1.5 ${i === active ? "bg-raise" : ""}`}
            >
              <span className="flex items-center gap-2 text-[13px]">
                {row.icon === "spec" ? <FileText aria-hidden="true" className="size-3.5 flex-none text-faint" /> : null}
                <span className="min-w-0 flex-1 truncate">{row.label}</span>
                {row.hint ? <span className="flex-none text-[11.5px] text-faint">{row.hint}</span> : null}
              </span>
              {row.snippet ? (
                <span className="truncate text-[12px] text-muted">
                  <Snippet snippet={row.snippet} />
                </span>
              ) : null}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

/** Ctrl+K anywhere, the editor included (CMD-01): handled in the capture phase, before any
 *  other key binding. */
export function usePaletteShortcut(): void {
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "k") {
        e.preventDefault();
        e.stopPropagation();
        const p = usePalette.getState();
        p.setOpen(!p.open);
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, []);
}
