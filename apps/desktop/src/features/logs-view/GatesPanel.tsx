import { AlertTriangle, Check, ChevronDown, ChevronRight, Lock, X } from "lucide-react";
import { Fragment } from "react";
import { t } from "../../lib/i18n";
import type { GateRun, GatesView } from "../../lib/ipc";
import { Chip, Segmented } from "../../ui";
import { formatDuration, formatWhen, shortPath } from "./format";
import { isFail, type LevelFilter, type StatusFilter, useLogs } from "./store";

export function SkippedNotice({ file, skipped, lines }: { file: string; skipped: number; lines: readonly number[] }) {
  if (skipped === 0) return null;
  return (
    <p role="status" className="m-0 flex items-center gap-2 border-b border-line bg-warn-bg px-6 py-2 text-[12.5px] text-warn">
      <AlertTriangle aria-hidden="true" className="size-4 flex-none" />
      {t("logs.skipped", { n: skipped, file, lines: lines.join(", ") })}
    </p>
  );
}

function StatusChip({ run }: { run: GateRun }) {
  return isFail(run) ? (
    <Chip tone="warn" icon={<X aria-hidden="true" className="size-3" />}>
      {run.status ?? t("logs.fail")}
    </Chip>
  ) : (
    <Chip tone="ok" icon={<Check aria-hidden="true" className="size-3" />}>
      {run.status ?? t("logs.pass")}
    </Chip>
  );
}

function FailureDetail({ run }: { run: GateRun }) {
  const code = run.exit ?? "?";
  return (
    <div className="mx-6 my-2 rounded-[var(--radius-lg)] border border-line bg-code px-3.5 py-3 font-code text-xs leading-relaxed">
      {run.tail.length > 0 ? (
        <pre className="m-0 whitespace-pre-wrap text-text">
          {run.tail.map((line, i) => (
            <span key={i} className={/fail|error|panick/i.test(line) ? "text-warn" : undefined}>
              {line}
              {"\n"}
            </span>
          ))}
        </pre>
      ) : (
        <p className="m-0 text-muted">{t("logs.noTail")}</p>
      )}
      <p className="m-0 mt-1 text-faint">
        {run.cwd ? t("logs.exitCwd", { code, cwd: shortPath(run.cwd) }) : t("logs.exitOnly", { code })}
      </p>
    </div>
  );
}

/** Gate history, newest first, with level/status filters and counts (LOG-01, LOG-02). */
export function GatesPanel({ gates }: { gates: GatesView }) {
  const level = useLogs((s) => s.level);
  const status = useLogs((s) => s.status);
  const open = useLogs((s) => s.open);
  const setLevel = useLogs((s) => s.setLevel);
  const setStatus = useLogs((s) => s.setStatus);
  const toggleRun = useLogs((s) => s.toggleRun);
  const rows = gates.runs
    .map((run, index) => ({ run, index }))
    .filter(({ run }) => level === "all" || run.level === level)
    .filter(({ run }) => status === "all" || (status === "fail") === isFail(run));
  const c = gates.counts;
  return (
    <>
      <div role="group" aria-label={t("logs.filtersLabel")} className="flex min-h-12 flex-none flex-wrap items-center gap-3 border-b border-line px-6 py-2">
        <span className="flex items-center gap-2">
          <span className="text-xs text-muted">{t("logs.level")}</span>
          <Segmented<LevelFilter>
          label={t("logs.levelLabel")}
          value={level}
          onChange={setLevel}
          options={[
            { value: "all", label: t("logs.all") },
            { value: "fase", label: t("logs.levelFase") },
            { value: "final", label: t("logs.levelFinal") },
            { value: "e2e", label: t("logs.levelE2e") },
          ]}
        />
        </span>
        <span className="flex items-center gap-2">
          <span className="text-xs text-muted">{t("logs.status")}</span>
          <Segmented<StatusFilter>
          label={t("logs.statusLabel")}
          value={status}
          onChange={setStatus}
          options={[
            { value: "all", label: t("logs.all") },
            { value: "pass", label: t("logs.pass") },
            { value: "fail", label: t("logs.fail") },
          ]}
        />
        </span>
        <div className="flex-1" />
        <span className="flex items-center gap-3">
          <span className="text-xs whitespace-nowrap text-muted">
            {t("logs.counts", { runs: t("logs.runs", { n: c.runs }), passes: c.passes, fails: c.fails })}
          </span>
          <span title={t("logs.readOnlyHint")}>
            <Chip icon={<Lock aria-hidden="true" className="size-3" />}>{t("logs.readOnly")}</Chip>
          </span>
        </span>
      </div>
      <SkippedNotice file="gates.jsonl" skipped={gates.skipped} lines={gates.skippedLines} />
      <div className="min-h-0 flex-1 overflow-auto">
        {gates.runs.length === 0 ? <p className="m-0 px-6 py-4 text-muted">{t("logs.noRuns")}</p> : null}
        {gates.runs.length > 0 && rows.length === 0 ? <p className="m-0 px-6 py-4 text-muted">{t("logs.noMatch")}</p> : null}
        {rows.length > 0 ? (
          <table className="w-full border-collapse text-[13px]">
            <thead>
              <tr className="text-left text-[11px] font-semibold tracking-[.06em] text-faint uppercase">
                <th className="h-9 border-b border-line pl-6 font-semibold">{t("logs.when")}</th>
                <th className="border-b border-line font-semibold">{t("logs.level")}</th>
                <th className="border-b border-line font-semibold">{t("logs.status")}</th>
                <th className="border-b border-line font-semibold">{t("logs.head")}</th>
                <th className="border-b border-line font-semibold">{t("logs.duration")}</th>
                <th className="border-b border-line pr-6 font-semibold">{t("logs.command")}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map(({ run, index }) => {
                const fail = isFail(run);
                const expanded = fail && open.has(index);
                const when = formatWhen(run.ts);
                return (
                  <Fragment key={index}>
                    <tr className="h-10 border-b border-line">
                      <td className="pl-6 whitespace-nowrap">
                        {fail ? (
                          <button
                            type="button"
                            aria-expanded={expanded}
                            aria-label={expanded ? t("logs.collapseRun", { when }) : t("logs.expandRun", { when })}
                            onClick={() => toggleRun(index)}
                            className="-ml-5 inline-flex items-center gap-1"
                          >
                            {expanded ? <ChevronDown aria-hidden="true" className="size-3.5 text-faint" /> : <ChevronRight aria-hidden="true" className="size-3.5 text-faint" />}
                            <span aria-hidden="true">{when}</span>
                          </button>
                        ) : (
                          when
                        )}
                      </td>
                      <td>{run.level ? <Chip>{run.level}</Chip> : null}</td>
                      <td>
                        <StatusChip run={run} />
                      </td>
                      <td className="font-code text-xs text-acc">{run.head?.slice(0, 7) ?? "—"}</td>
                      <td className="font-code text-xs whitespace-nowrap text-muted">{formatDuration(run.duration_s)}</td>
                      <td className="max-w-0 truncate pr-6 font-code text-xs" title={run.cmd ?? undefined}>
                        {run.cmd}
                      </td>
                    </tr>
                    {expanded ? (
                      <tr className="border-b border-line">
                        <td colSpan={6}>
                          <FailureDetail run={run} />
                        </td>
                      </tr>
                    ) : null}
                  </Fragment>
                );
              })}
            </tbody>
          </table>
        ) : null}
      </div>
    </>
  );
}
