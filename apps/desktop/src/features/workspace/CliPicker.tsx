import { useEffect, useRef } from "react";
import { t } from "../../lib/i18n";
import { Button } from "../../ui";
import { specTitle } from "./model";
import { useWorkspace } from "./store";

/** Chooser for a command-line key found in more than one project (CLI-02). */
export function CliPicker() {
  const pick = useWorkspace((s) => s.pick);
  const projects = useWorkspace((s) => s.projects);
  const setPick = useWorkspace((s) => s.setPick);
  const select = useWorkspace((s) => s.select);
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!pick) return;
    first.current?.focus();
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setPick(null);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [pick, setPick]);
  if (!pick) return null;
  const options = projects.flatMap((p) =>
    p.specs.filter((s) => pick.specIds.includes(s.id)).map((s) => ({ project: p.alias ?? p.name, spec: s })),
  );
  return (
    <div className="fixed inset-0 z-30 flex items-start justify-center bg-bg/60 pt-24">
      <div role="dialog" aria-modal="true" aria-label={t("cli.pickTitle", { query: pick.query })} className="flex w-[460px] flex-col gap-2 rounded-[var(--radius-xl)] border border-line bg-panel p-3 shadow-2xl">
        <div className="px-1">
          <p className="m-0 text-sm font-semibold">{t("cli.pickTitle", { query: pick.query })}</p>
          <p className="m-0 text-xs text-muted">{t("cli.pickHint")}</p>
        </div>
        <ul aria-label={t("cli.pickList")} className="m-0 flex list-none flex-col gap-1 p-0">
          {options.map(({ project, spec }, i) => (
            <li key={spec.id}>
              <button
                ref={i === 0 ? first : undefined}
                type="button"
                onClick={() => {
                  setPick(null);
                  select(spec.id);
                }}
                className="flex w-full flex-col items-start gap-0.5 rounded-[var(--radius-md)] px-3 py-2 text-left hover:bg-raise"
              >
                <span className="text-xs text-muted">{project}</span>
                <span className="flex items-center gap-2">
                  <span className="font-code text-xs text-acc">{spec.issueKey ?? spec.folder}</span>
                  <span className="font-medium">{specTitle(spec)}</span>
                </span>
              </button>
            </li>
          ))}
        </ul>
        <div className="flex justify-end px-1">
          <Button variant="ghost" onClick={() => setPick(null)}>
            {t("cli.cancel")}
          </Button>
        </div>
      </div>
    </div>
  );
}
