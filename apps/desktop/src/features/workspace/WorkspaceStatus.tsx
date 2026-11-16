import { t } from "../../lib/i18n";
import { StatusDot } from "../../ui";
import { counts } from "./model";
import { useWorkspace } from "./store";

/** Status bar: watcher (T21) on the left, counts and index state on the right. */
export function WorkspaceStatus() {
  const projects = useWorkspace((s) => s.projects);
  const scan = useWorkspace((s) => s.scan);
  const watch = useWorkspace((s) => s.watch);
  const c = counts(projects);
  const updating = scan?.phase === "scanning" || scan?.phase === "indexing";
  const watching = watch !== null && watch.repos > 0;
  return (
    <>
      <div className="flex items-center gap-2">
        <StatusDot tone={watching ? (watch.fallbacks.length > 0 ? "warn" : "ok") : "faint"} />
        <span>
          {watching ? t("statusbar.watching", { n: watch.repos }) : t("statusbar.watcherOff")}
          {watching && watch.polled > 0
            ? ` · ${t("statusbar.polling", { seconds: Math.round(watch.pollIntervalMs / 100) / 10, n: watch.polled })}`
            : null}
          {watching && watch.fallbacks.length > 0 ? ` · ${t("statusbar.fallbacks", { n: watch.fallbacks.length })}` : null}
        </span>
      </div>
      <div className="flex-1" />
      <span>
        {t("statusbar.counts", { specs: t("scan.specs", { n: c.specs }), projects: t("scan.projects", { n: c.projects }) })}
        {" · "}
        {updating ? t("statusbar.indexUpdating") : t("statusbar.indexFresh")}
      </span>
    </>
  );
}
