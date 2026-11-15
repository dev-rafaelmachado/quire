import { t } from "../../lib/i18n";
import { StatusDot } from "../../ui";
import { counts } from "./model";
import { useWorkspace } from "./store";

/** Status bar: watcher (T21) on the left, counts and index state on the right. */
export function WorkspaceStatus() {
  const projects = useWorkspace((s) => s.projects);
  const scan = useWorkspace((s) => s.scan);
  const c = counts(projects);
  const updating = scan?.phase === "scanning" || scan?.phase === "indexing";
  return (
    <>
      <div className="flex items-center gap-2">
        <StatusDot tone="faint" />
        <span>{t("statusbar.watcherOff")}</span>
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
