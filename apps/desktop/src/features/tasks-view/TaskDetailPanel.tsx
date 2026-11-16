import { CircleCheck, Lock } from "lucide-react";
import type { ReactNode } from "react";
import { t } from "../../lib/i18n";
import { Chip } from "../../ui";
import { CommitChip, NoteBadge } from "./TasksPanel";
import { dependsOn, fieldItems, fieldList, fieldText, markOf, nextTaskId } from "./model";
import { useTasks } from "./store";

/** Renders `code` spans of a plain detail line. */
function InlineCode({ text }: { text: string }) {
  return (
    <>
      {text.split("`").map((part, i) =>
        i % 2 === 1 ? (
          <code key={i} className="rounded-[var(--radius-sm)] bg-raise px-1 font-code text-[11.5px]">
            {part}
          </code>
        ) : (
          part
        ),
      )}
    </>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <h4 className="m-0 text-[11px] font-semibold tracking-[.06em] text-faint uppercase">{label}</h4>
      {children}
    </div>
  );
}

/** Right column of the tasks tab: the selected task's detail section (TSK-04). */
export function TaskDetailPanel() {
  const view = useTasks((s) => s.view);
  const selected = useTasks((s) => s.selected);
  const gates = useTasks((s) => s.gates);
  if (!view || !selected) return null;
  const checkbox = view.checkbox;
  const task = view.phases.flatMap((p) => p.tasks).find((x) => x.id === selected);
  const detail = view.details.find((d) => d.id === selected);
  const next = nextTaskId(view.phases);
  const allTasks = new Map(view.phases.flatMap((p) => p.tasks).map((x) => [x.id, x]));
  const mark = task ? markOf(task, next) : "pending";
  const where = fieldText(detail, "Onde");
  const deps = dependsOn(detail);
  const reqs = task?.requirements ?? [];
  const ready = fieldItems(detail, "Pronto quando");
  const tests = fieldList(detail, "Testes");
  const what = fieldText(detail, "What");
  const commits = task?.commits ?? [];
  // The last commit of the task is the one its gate ran on; earlier ones are looked up too.
  const gated = [...commits].reverse().find((c) => gates.some((g) => g.head?.startsWith(c)));
  const gate = gated ? gates.find((g) => g.head?.startsWith(gated)) : undefined;
  return (
    <section aria-label={t("tasks.detail")} className="flex min-h-0 flex-1 flex-col">
      <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto px-4 pt-4 pb-4">
        <div className="flex items-center gap-2">
          <Chip mono>{selected}</Chip>
          <Chip tone={mark === "done" ? "ok" : mark === "next" ? "acc" : "neutral"}>
            {t(mark === "done" ? "tasks.taskDone" : mark === "next" ? "tasks.taskNext" : "tasks.taskPending")}
          </Chip>
          {commits.map((hash) => (
            <CommitChip key={hash} hash={hash} />
          ))}
        </div>
        {task?.note ? (
          <div>
            <NoteBadge note={task.note} />
          </div>
        ) : null}
        <h3 className="m-0 text-base leading-snug font-semibold">{task?.title ?? detail?.title}</h3>
        {!detail ? <p className="m-0 text-[12.5px] text-muted">{t("tasks.noDetail", { id: selected })}</p> : null}
        {what ? (
          <Field label={t("tasks.what")}>
            <p className="m-0 text-[12.5px] text-muted">{what}</p>
          </Field>
        ) : null}
        {where ? (
          <Field label={t("tasks.where")}>
            <span className="font-code text-xs break-all text-acc">{where}</span>
          </Field>
        ) : null}
        {deps.length > 0 ? (
          <Field label={t("tasks.dependsOn")}>
            <div className="flex flex-wrap gap-1">
              {deps.map((d) => (
                <Chip key={d} mono tone={allTasks.get(d)?.done ? "ok" : "neutral"} icon={allTasks.get(d)?.done ? <CircleCheck aria-hidden="true" className="size-3" /> : undefined}>
                  {d}
                </Chip>
              ))}
            </div>
          </Field>
        ) : null}
        {reqs.length > 0 ? (
          <Field label={t("tasks.requirement")}>
            <div className="flex flex-wrap gap-1">
              {reqs.map((r) => (
                <Chip key={r} tone="acc" mono>
                  {r}
                </Chip>
              ))}
            </div>
          </Field>
        ) : null}
        {ready.length > 0 ? (
          <Field label={t("tasks.readyWhen")}>
            <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
              {ready.map((item) => (
                <li key={item} className="flex items-start gap-2 text-[12.5px]">
                  <input type="checkbox" disabled checked={task?.done ?? false} aria-label={item.replace(/`/g, "")} className="mt-0.5 accent-[var(--acc)]" />
                  <span>
                    <InlineCode text={item} />
                  </span>
                </li>
              ))}
            </ul>
          </Field>
        ) : null}
        {tests.length > 0 ? (
          <Field label={t("tasks.tests")}>
            <div className="flex flex-wrap gap-1">
              {tests.map((x) => (
                <Chip key={x}>{x}</Chip>
              ))}
            </div>
          </Field>
        ) : null}
        {fieldText(detail, "Gate") ? (
          <Field label={t("tasks.gate")}>
            <div className="flex flex-wrap items-center gap-2">
              <Chip>{fieldText(detail, "Gate")}</Chip>
              {gate ? (
                <span className="text-xs text-muted">
                  {t("tasks.lastGate", {
                    level: gate.level ?? "",
                    hash: gated ?? "",
                    status: gate.status === "fail" ? t("tasks.gateFail") : t("tasks.gatePass"),
                  })}
                </span>
              ) : null}
            </div>
          </Field>
        ) : null}
      </div>
      <p className="m-0 flex flex-none items-start gap-2 border-t border-line px-4 py-3 text-[11.5px] text-faint">
        <Lock aria-hidden="true" className="mt-0.5 size-3 flex-none" />
        {checkbox ? t("tasks.checkboxNote") : t("tasks.readOnlyNote")}
      </p>
    </section>
  );
}
