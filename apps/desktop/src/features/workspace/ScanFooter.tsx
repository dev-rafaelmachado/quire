import { RotateCw } from "lucide-react";
import { t } from "../../lib/i18n";
import { errorText } from "../../lib/i18n/errors";
import { ErrorText, IconButton, StatusDot, type DotTone } from "../../ui";
import { counts } from "./model";
import { useWorkspace } from "./store";

/** Sidebar footer: scan state and per-path errors (DSC-04, DSC-05). */
export function ScanFooter() {
  const scan = useWorkspace((s) => s.scan);
  const config = useWorkspace((s) => s.config);
  const projects = useWorkspace((s) => s.projects);
  const error = useWorkspace((s) => s.error);
  const rescan = useWorkspace((s) => s.rescan);
  const paths = config?.scanPaths ?? [];
  const c = counts(projects);

  let tone: DotTone = "faint";
  let label = t("scan.idle");
  const running = scan?.phase === "scanning" || scan?.phase === "indexing";
  if (paths.length === 0 && config) {
    tone = "warn";
    label = t("scan.noScanPaths");
  } else if (scan?.phase === "scanning") {
    tone = "acc";
    label = paths.length === 1 ? t("scan.scanning", { path: paths[0] ?? "" }) : t("scan.scanningMany", { n: paths.length });
  } else if (scan?.phase === "indexing") {
    tone = "acc";
    label = t("scan.indexing", { done: scan.processed, total: scan.total });
  } else if (scan?.phase === "done") {
    tone = scan.errors.length > 0 ? "warn" : "ok";
    label = t("scan.done");
  } else if (scan?.phase === "failed") {
    tone = "warn";
    label = t("scan.failed", { message: errorText({ code: "index" }) });
  }

  return (
    <div className="mt-auto flex flex-col gap-1 border-t border-line px-4 py-2.5" aria-live="polite">
      <div className="flex items-center gap-2">
        <StatusDot tone={tone} pulse={running} />
        <span className="min-w-0 flex-1 truncate text-xs text-muted" title={label}>
          {label}
        </span>
        <IconButton
          label={t("scan.rescan")}
          icon={<RotateCw className="size-3.5" />}
          onClick={() => void rescan()}
          disabled={running}
          className="-my-1 size-6"
        />
      </div>
      <span className="pl-[15px] text-[11px] text-faint">
        {t("scan.summary", {
          projects: t("scan.projects", { n: c.projects }),
          extra: t("scan.extraWorktrees", { n: c.worktrees }),
          specs: t("scan.specs", { n: c.specs }),
        })}
      </span>
      {scan?.errors.map((e) => (
        <span key={e.path} className="pl-[15px] text-[11px] break-all text-warn">
          {t("scan.pathError", { path: e.path, message: errorText(e) })}
        </span>
      ))}
      {config?.error ? (
        <span className="pl-[15px] text-[11px] text-warn">
          {t("scan.configError")}{" "}
          <ErrorText error={config.error} />
        </span>
      ) : null}
      {error ? <ErrorText error={error} className="pl-[15px] text-[11px] text-warn" /> : null}
    </div>
  );
}
