import { FileText, Loader2, Search, X } from "lucide-react";
import { useEffect, useRef } from "react";
import { t } from "../../lib/i18n";
import { ErrorText, IconButton } from "../../ui";
import { useSpecView } from "../spec-view/store";
import { useWorkspace } from "../workspace/store";
import { Snippet } from "./Snippet";
import { useSearch } from "./store";

/** Files offered in the file filter (SRC-05). */
export const SEARCH_FILES = ["spec.md", "design.md", "tasks.md", "summary.md", "validation.md"] as const;

const control =
  "h-[30px] min-w-0 rounded-[var(--radius-md)] border border-line bg-bg px-2.5 text-[12.5px] text-text outline-none focus-visible:border-acc";

/** Search in every spec, or in the open one (SRC-03..06), in the center column. */
export function SearchPanel() {
  const text = useSearch((s) => s.text);
  const projectId = useSearch((s) => s.projectId);
  const file = useSearch((s) => s.file);
  const inSpec = useSearch((s) => s.inSpec);
  const results = useSearch((s) => s.results);
  const loading = useSearch((s) => s.loading);
  const error = useSearch((s) => s.error);
  const { setText, setFilters, setOpen, openHit } = useSearch.getState();
  const projects = useWorkspace((s) => s.projects);
  const specOpen = useSpecView((s) => s.specId !== null);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus();
    // Text already there (reopened panel or `?search=`): show its results.
    if (useSearch.getState().text.trim() !== "" && useSearch.getState().results === null) void useSearch.getState().run();
  }, []);

  // The index changed (a scan, a save): the results follow it.
  useEffect(() => {
    if (useSearch.getState().text.trim() !== "") void useSearch.getState().run();
  }, [projects]);

  const hits = results ? results.groups.reduce((n, g) => n + g.hits.length, 0) : 0;
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="flex max-w-[880px] flex-col gap-4 px-10 pt-8 pb-24">
        <div className="flex items-center gap-3">
          <h1 className="m-0 flex-1 text-[22px] font-semibold tracking-tight">{t("search.title")}</h1>
          <IconButton label={t("search.close")} icon={<X className="size-4" />} onClick={() => setOpen(false)} />
        </div>
        <label className="flex items-center gap-2 rounded-[var(--radius-md)] border border-line bg-panel px-3 focus-within:border-acc">
          <Search aria-hidden="true" className="size-4 text-faint" />
          <input
            ref={input}
            type="search"
            value={text}
            onChange={(e) => setText(e.target.value)}
            aria-label={t("search.input")}
            placeholder={t("search.placeholder")}
            className="h-10 min-w-0 flex-1 bg-transparent text-[14px] outline-none"
          />
          {loading ? <Loader2 aria-hidden="true" className="size-4 animate-spin text-faint" /> : null}
        </label>
        <div role="group" aria-label={t("search.filters")} className="flex flex-wrap items-center gap-3 text-[12.5px]">
          <select aria-label={t("search.project")} value={projectId ?? ""} onChange={(e) => setFilters({ projectId: e.target.value === "" ? null : Number(e.target.value) })} className={control}>
            <option value="">{t("search.allProjects")}</option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.alias ?? p.name}
              </option>
            ))}
          </select>
          <select aria-label={t("search.file")} value={file ?? ""} onChange={(e) => setFilters({ file: e.target.value || null })} className={control}>
            <option value="">{t("search.allFiles")}</option>
            {SEARCH_FILES.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
          <label className="flex items-center gap-2 text-muted">
            <input type="checkbox" checked={inSpec && specOpen} disabled={!specOpen} onChange={(e) => setFilters({ inSpec: e.target.checked })} />
            {t("search.inSpec")}
          </label>
        </div>
        {error ? (
          <p role="alert" className="m-0 text-warn">
            <ErrorText error={error} />
          </p>
        ) : null}
        <p role="status" className="m-0 text-[12.5px] text-muted">
          {results === null
            ? t("search.hint")
            : hits === 0
              ? t("search.none")
              : `${t("search.count", { n: hits })} · ${t("search.specs", { n: results.groups.length })}${results.truncated ? ` · ${t("search.truncated")}` : ""}`}
        </p>
        {results?.groups.map((g) => (
          <section key={g.specId} aria-label={g.issueKey ?? g.folder} className="flex flex-col gap-1">
            <h2 className="m-0 flex items-baseline gap-2 text-[13px]">
              <span className="font-code text-acc">{g.issueKey ?? g.folder}</span>
              <span className="min-w-0 truncate font-semibold">{g.title ?? g.folder}</span>
              <span className="text-[11.5px] text-faint">{g.project}</span>
            </h2>
            <ul className="m-0 flex list-none flex-col gap-0.5 p-0">
              {g.hits.map((h) => (
                <li key={`${h.instanceId}-${h.file}-${h.line}`}>
                  <button
                    type="button"
                    onClick={() => openHit(g, h)}
                    className="flex w-full flex-col gap-0.5 rounded-[var(--radius-md)] px-3 py-2 text-left hover:bg-raise focus-visible:bg-raise"
                  >
                    <span className="flex items-center gap-2 text-[12px] text-muted">
                      <FileText aria-hidden="true" className="size-3.5" />
                      <span className="font-code">{h.file}</span>
                      {h.heading ? <span className="min-w-0 truncate">{h.heading}</span> : null}
                      {h.branch ? <span className="ml-auto flex-none font-code text-[11px] text-faint">{h.branch}</span> : null}
                    </span>
                    <span className="text-[13px] leading-relaxed">
                      <Snippet snippet={h.snippet} />
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </section>
        ))}
      </div>
    </div>
  );
}
