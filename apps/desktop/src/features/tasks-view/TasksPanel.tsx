import { Check, ChevronDown, ChevronRight, CircleCheck, CircleDot, Circle, Lock } from "lucide-react";
import { t } from "../../lib/i18n";
import type { PhaseView, Task, TasksView } from "../../lib/ipc";
import { Chip, cx, ProgressBar, Segmented, type BarTone, type Tone } from "../../ui";
import { fieldText, markOf, nextTaskId, phaseKey, type TaskMark } from "./model";
import { type TaskFilter, useTasks } from "./store";

const STATE = {
  complete: { label: "tasks.phaseComplete", tone: "ok", bar: "ok" },
  inProgress: { label: "tasks.phaseInProgress", tone: "acc", bar: "acc" },
  pending: { label: "tasks.phasePending", tone: "neutral", bar: "faint" },
} as const satisfies Record<PhaseView["state"], { label: "tasks.phaseComplete" | "tasks.phaseInProgress" | "tasks.phasePending"; tone: Tone; bar: BarTone }>;

const MARK_LABEL = { done: "tasks.taskDone", next: "tasks.taskNext", pending: "tasks.taskPending" } as const;

function MarkIcon({ mark }: { mark: TaskMark }) {
  const label = t(MARK_LABEL[mark]);
  if (mark === "done") return <CircleCheck role="img" aria-label={label} className="size-4 flex-none text-ok" />;
  if (mark === "next") return <CircleDot role="img" aria-label={label} className="size-4 flex-none text-acc" />;
  return <Circle role="img" aria-label={label} className="size-4 flex-none text-faint" />;
}

function TaskRow({ task, mark, selected, gate }: { task: Task; mark: TaskMark; selected: boolean; gate: string | null }) {
  const select = useTasks((s) => s.select);
  return (
    <button
      type="button"
      aria-current={selected ? "true" : undefined}
      onClick={() => select(task.id)}
      className={cx(
        "flex min-h-10 w-full items-center gap-3 rounded-[var(--radius-md)] border px-3 py-1.5 text-left",
        selected ? "border-acc bg-acc-bg/40" : "border-transparent hover:bg-raise",
      )}
    >
      <MarkIcon mark={mark} />
      <span className="w-10 flex-none font-code text-xs text-muted">{task.id}</span>
      <span className="min-w-0 flex-1 truncate">{task.title}</span>
      <span className="flex flex-none gap-1">
        {task.requirements.map((r) => (
          <Chip key={r} tone="acc" mono>
            {r}
          </Chip>
        ))}
      </span>
      <span className="w-16 flex-none text-center font-code text-xs">
        {task.hash ? (
          <span className="text-acc" title={t("tasks.commit", { hash: task.hash })}>
            {task.hash}
          </span>
        ) : (
          <span className="text-faint" aria-label={t("tasks.noHash")}>
            —
          </span>
        )}
      </span>
      <span className="w-12 flex-none text-right">{gate ? <Chip>{gate}</Chip> : null}</span>
    </button>
  );
}

function PhaseCard({ phase, index, next, filter }: { phase: PhaseView; index: number; next: string | null; filter: TaskFilter }) {
  const view = useTasks((s) => s.view);
  const expanded = useTasks((s) => s.expanded.has(phaseKey(phase, index)));
  const toggle = useTasks((s) => s.togglePhase);
  const selected = useTasks((s) => s.selected);
  const st = STATE[phase.state];
  const title = phase.number === null ? t("tasks.implicitPhase") : t("tasks.phaseTitle", { n: phase.number, title: phase.title });
  const tasks = filter === "pending" ? phase.tasks.filter((x) => !x.done) : phase.tasks;
  const done = new Set(phase.tasks.filter((x) => x.done).map((x) => x.id));
  const gateOf = (id: string) => fieldText(view?.details.find((d) => d.id === id), "Gate");
  return (
    <section className="rounded-[var(--radius-lg)] border border-line bg-panel">
      <h3 className="m-0">
        <button
          type="button"
          aria-expanded={expanded}
          aria-label={expanded ? t("tasks.collapse", { title }) : t("tasks.expand", { title })}
          onClick={() => toggle(phaseKey(phase, index))}
          className="flex h-10 w-full items-center gap-3 px-4 text-left"
        >
          {expanded ? <ChevronDown aria-hidden="true" className="size-3.5 text-faint" /> : <ChevronRight aria-hidden="true" className="size-3.5 text-faint" />}
          <span className="min-w-0 flex-1 truncate text-sm font-semibold" aria-hidden="true">
            {title}
          </span>
          <span className="font-code text-xs text-muted" aria-hidden="true">
            {t("tasks.count", { done: phase.done, total: phase.total })}
          </span>
          <ProgressBar done={phase.done} total={phase.total} tone={st.bar} width={80} label={t("progress.label", { done: phase.done, total: phase.total })} />
          <Chip tone={st.tone} icon={phase.state === "complete" ? <Check aria-hidden="true" className="size-3" /> : undefined}>
            {t(st.label)}
          </Chip>
        </button>
      </h3>
      {expanded ? (
        <div className="flex flex-col gap-1 px-3 pb-3">
          {phase.diagram.length > 0 ? (
            <div role="group" aria-label={t("tasks.diagram")} className="flex flex-col gap-1.5 px-2 pb-2">
              {phase.diagram.map((chain, ci) => (
                <div key={ci} className="flex flex-wrap items-center gap-1.5">
                  {chain.map((id, i) => (
                    <span key={`${id}-${i}`} className="flex items-center gap-1.5">
                      {i > 0 ? <ChevronRight aria-hidden="true" className="size-3 text-faint" /> : null}
                      <Chip mono tone={done.has(id) ? "ok" : id === next ? "acc" : "neutral"}>
                        {id}
                      </Chip>
                    </span>
                  ))}
                </div>
              ))}
            </div>
          ) : null}
          {tasks.map((task) => (
            <TaskRow key={`${task.id}-${task.line}`} task={task} mark={markOf(task, next)} selected={task.id === selected} gate={gateOf(task.id)} />
          ))}
        </div>
      ) : null}
    </section>
  );
}

/** `tasks.md` by phase (TSK-01..03); read-only (TSK-05). */
export function TasksPanel({ view }: { view: TasksView }) {
  const filter = useTasks((s) => s.filter);
  const setFilter = useTasks((s) => s.setFilter);
  const next = nextTaskId(view.phases);
  const phasesDone = view.phases.filter((p) => p.state === "complete").length;
  const visible = filter === "pending" ? view.phases.filter((p) => p.tasks.some((x) => !x.done)) : view.phases;
  return (
    <>
      <div className="flex min-h-12 flex-none flex-wrap items-center gap-2.5 border-b border-line px-6 py-2">
        <span className="font-code text-[13px] font-semibold">
          {t("tasks.count", { done: view.done, total: view.total })}
          <span className="ml-1 font-sans text-xs font-normal text-muted">{t("tasks.word", { n: view.total })}</span>
        </span>
        <ProgressBar done={view.done} total={view.total} tone={view.done === view.total ? "ok" : "acc"} width={180} label={t("progress.label", { done: view.done, total: view.total })} />
        <Chip>{t("tasks.phasesDone", { done: phasesDone, total: view.phases.length })}</Chip>
        <Chip icon={<Lock aria-hidden="true" className="size-3" />}>{t("tasks.readOnly")}</Chip>
        <div className="flex-1" />
        <Segmented
          label={t("tasks.filterLabel")}
          value={filter}
          onChange={setFilter}
          options={[
            { value: "all", label: t("tasks.all") },
            { value: "pending", label: t("tasks.pending") },
          ]}
        />
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="flex max-w-[860px] flex-col gap-3 px-6 pt-5 pb-24">
          {view.phases.length === 0 ? <p className="m-0 text-muted">{t("tasks.noTasks")}</p> : null}
          {visible.map((p) => {
            const index = view.phases.indexOf(p);
            return <PhaseCard key={phaseKey(p, index)} phase={p} index={index} next={next} filter={filter} />;
          })}
        </div>
      </div>
    </>
  );
}
