import { devParam } from "../../lib/devParams";
import { GitBranch, Layers } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { t } from "../../lib/i18n";
import type { InstanceSummary, SpecDetail } from "../../lib/ipc";
import { Button, Chip, cx } from "../../ui";
import { ago, displayed, shortInstancePath, worktreeName } from "./labels";
import { guard } from "../unsaved/store";
import { useSpecView } from "./store";

type State = "open" | "diverges" | "identical";

function stateOf(instance: InstanceSummary, shown: InstanceSummary): State {
  if (instance.id === shown.id) return "open";
  return instance.contentHash === shown.contentHash ? "identical" : "diverges";
}

function InstanceOption({ instance, shown, checked, onCheck }: { instance: InstanceSummary; shown: InstanceSummary; checked: boolean; onCheck: () => void }) {
  const state = stateOf(instance, shown);
  const branch = instance.branch ?? t("specView.detached");
  return (
    <label className={cx("flex cursor-pointer gap-3 rounded-[var(--radius-md)] px-3 py-2.5", checked ? "bg-acc-bg/60" : "hover:bg-raise")}>
      <input type="radio" name="quire-instance" checked={checked} onChange={onCheck} className="mt-1 accent-[var(--acc)]" />
      <span className="flex min-w-0 flex-1 flex-col gap-1">
        <span className="flex items-center gap-2">
          <span className="font-code text-[13px] font-semibold">{branch}</span>
          <Chip>{instance.kind === "worktree" ? t("instances.worktree") : t("instances.checkout")}</Chip>
          <span className="flex-1" />
          {state === "open" ? <span className="text-xs font-medium text-acc">{t("instances.stateOpen")}</span> : null}
          {state === "diverges" ? <Chip tone="warn">{t("instances.stateDiverges")}</Chip> : null}
          {state === "identical" ? <Chip>{t("instances.stateIdentical")}</Chip> : null}
        </span>
        <span className="truncate font-code text-[11px] text-faint" title={instance.path}>
          {shortInstancePath(instance.repoRoot)}
        </span>
        <span className="text-xs text-muted">{t("instances.modified", { ago: ago(instance.mtimeMs, Date.now(), t) })}</span>
      </span>
    </label>
  );
}

function Popover({ detail, onClose }: { detail: SpecDetail; onClose: () => void }) {
  const switchInstance = useSpecView((s) => s.switchInstance);
  const shown = displayed(detail);
  const [picked, setPicked] = useState(shown?.id ?? 0);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    ref.current?.querySelector<HTMLInputElement>("input:checked")?.focus();
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    const onDown = (e: MouseEvent) => {
      const el = ref.current;
      if (el && e.target instanceof Node && !el.contains(e.target) && !(e.target as Element).closest?.("[data-instance-trigger]")) onClose();
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("mousedown", onDown);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("mousedown", onDown);
    };
  }, [onClose]);
  if (!shown) return null;
  const n = detail.spec.instances.length;
  const diverges = detail.spec.versionsBadge !== null;
  return (
    <div
      ref={ref}
      role="dialog"
      aria-label={diverges ? t("instances.title", { n, folder: detail.spec.folder }) : t("instances.titleSame", { n, folder: detail.spec.folder })}
      className="absolute top-[42px] right-0 z-20 flex w-[424px] flex-col gap-1 rounded-[var(--radius-xl)] border border-line bg-panel p-2 shadow-2xl"
    >
      <div className="px-3 pt-2 pb-1.5">
        <p className="m-0 text-[13px] font-semibold">
          {diverges ? t("instances.title", { n, folder: detail.spec.folder }) : t("instances.titleSame", { n, folder: detail.spec.folder })}
        </p>
        <p className="m-0 text-xs text-muted">{t("instances.hint")}</p>
      </div>
      <div role="radiogroup" aria-label={t("instances.list")} className="flex flex-col gap-0.5">
        {detail.spec.instances.map((i) => (
          <InstanceOption key={i.id} instance={i} shown={shown} checked={picked === i.id} onCheck={() => setPicked(i.id)} />
        ))}
      </div>
      <div className="mt-1 flex justify-end gap-2 border-t border-line px-2 pt-2.5 pb-1">
        <Button variant="ghost" onClick={onClose}>
          {t("instances.close")}
        </Button>
        <Button
          disabled={picked === shown.id}
          onClick={() => {
            onClose();
            guard(() => switchInstance(picked));
          }}
        >
          {t("instances.openThis")}
        </Button>
      </div>
    </div>
  );
}

/** Branch and worktree of the instance on screen (DUP-06) and, when the spec has more than one
 *  copy, the switcher with the "N versões" badge (DUP-03, DUP-05). */
export function InstanceBadge() {
  const detail = useSpecView((s) => s.detail);
  // `?picker` opens the switcher on first render (screenshots and demos).
  const [open, setOpen] = useState(() => devParam("picker") !== null);
  const instance = detail ? displayed(detail) : undefined;
  if (!detail || !instance) return null;
  const branch = instance.branch ?? t("specView.detached");
  const many = detail.spec.instances.length > 1;
  const content = (
    <>
      <GitBranch aria-hidden="true" className="size-3.5 text-muted" />
      <span className="font-code text-xs">{branch}</span>
      <span className="font-code text-xs text-faint">{worktreeName(instance)}</span>
      {detail.spec.versionsBadge ? (
        <Chip tone="acc" icon={<Layers aria-hidden="true" className="size-3" />}>
          {t("sidebar.versions", { n: detail.spec.versionsBadge })}
        </Chip>
      ) : null}
    </>
  );
  const box = "flex h-[30px] items-center gap-2 rounded-[var(--radius-md)] border bg-panel px-2.5";
  if (!many) {
    return (
      <div role="group" aria-label={t("specView.instance")} title={t("specView.instanceTitle", { branch, path: instance.path })} className={cx(box, "border-line")}>
        {content}
      </div>
    );
  }
  return (
    <div className="relative">
      <button
        type="button"
        data-instance-trigger
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-label={t("instances.open", { n: detail.spec.instances.length })}
        title={t("specView.instanceTitle", { branch, path: instance.path })}
        onClick={() => setOpen((v) => !v)}
        className={cx(box, open ? "border-acc" : "border-line hover:bg-raise")}
      >
        {content}
      </button>
      {open ? <Popover detail={detail} onClose={() => setOpen(false)} /> : null}
    </div>
  );
}
