import { GitBranch } from "lucide-react";
import { t } from "../../lib/i18n";
import { displayed, worktreeName } from "./labels";
import { useSpecView } from "./store";

/** Branch and worktree of the instance on screen (DUP-06). The switcher arrives in T20. */
export function InstanceBadge() {
  const detail = useSpecView((s) => s.detail);
  const instance = detail ? displayed(detail) : undefined;
  if (!instance) return null;
  const branch = instance.branch ?? t("specView.detached");
  return (
    <div
      role="group"
      aria-label={t("specView.instance")}
      title={t("specView.instanceTitle", { branch, path: instance.path })}
      className="flex h-[30px] items-center gap-2 rounded-[var(--radius-md)] border border-line bg-panel px-2.5"
    >
      <GitBranch aria-hidden="true" className="size-3.5 text-muted" />
      <span className="font-code text-xs">{branch}</span>
      <span className="font-code text-xs text-faint">{worktreeName(instance)}</span>
    </div>
  );
}
