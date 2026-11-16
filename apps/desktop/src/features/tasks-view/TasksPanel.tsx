import { Check, ChevronDown, ChevronRight, CircleCheck, CircleDot, Circle, Hourglass, Lock, X } from "lucide-react";
import { t } from "../../lib/i18n";
import { errorText } from "../../lib/i18n/errors";
import { useEditor } from "../editor/store";
import type { PhaseView, Task, TasksView } from "../../lib/ipc";
import { Chip, ChipList, cx, ErrorText, IconButton, ProgressBar, Segmented, type BarTone, type Tone } from "../../ui";
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

/** Short commit hash of a task line. */
export function CommitChip({ hash }: { hash: string }) {
  return (
    <span title={t("tasks.commit", { hash })}>
      <Chip tone="acc" mono>
        {hash}
      </Chip>
    </span>
  );
}

/** Annotation of a task line, such as "aguardando revisão visual"; never part of the title. */
export function NoteBadge({ note }: { note: string }) {
  return (
    <Chip tone="warn" icon={<Hourglass aria-hidden="true" className="size-3 flex-none" />} className="max-w-full min-w-0 overflow-hidden">
      <span className="sr-only">{t("tasks.note")} </span>
      <span className="min-w-0 truncate">{note}</span>
    </Chip>
  );
}

/** Why the checkboxes of this view cannot be used now, if they are on at all. */
function useCheckboxBlock(view: TasksView | null): string | null {
  const editingTasks = useEditor((s) => s.dirty && s.buffer?.name === "tasks.md");
  if (!view?.checkbox) return null;
  if (view.locked) return errorText(view.locked);
  if (editingTasks) return t("tasks.checkboxDirty");
  return null;
}

function TaskRow({ task, mark, selected, gate }: { task: Task; mark: TaskMark; selected: boolean; gate: string | null }) {
  const select = useTasks((s) => s.select);
  const view = useTasks((s) => s.view);
  const toggle = useTasks((s) => s.toggle);
  const toggling = useTasks((s) => s.toggling);
  const block = useCheckboxBlock(view);
  const checkbox = view?.checkbox ?? false;
  return (
    <div
      className={cx(
        "flex min-h-10 w-full min-w-0 items-center rounded-[var(--radius-md)] border",
        selected ? "border-acc bg-acc-bg/40" : "border-transparent hover:bg-raise",
      )}
    >
      {checkbox ? (
        // A real checkbox when the project opted in (CHK-01); it writes only the box (CHK-02).
        <input
          type="checkbox"
          checked={task.done}
          disabled={block !== null || toggling !== null}
          title={block ?? undefined}
          aria-label={t("tasks.toggle", { id: task.id, title: task.title })}
          onChange={() => void toggle(task)}
          className="ml-2.5 size-4 flex-none accent-[var(--acc)] @[560px]:ml-3"
        />
      ) : null}
      <button
        type="button"
        aria-current={selected ? "true" : undefined}
        onClick={() => select(task.id)}
        className="flex min-h-10 w-full min-w-0 items-center gap-2 px-2.5 py-1.5 text-left @[560px]:gap-3 @[560px]:px-3"
      >
        {checkbox ? null : <MarkIcon mark={mark} />}
        <span className="w-10 flex-none font-code text-xs text-muted">{task.id}</span>
        <span className="min-w-24 flex-1 truncate @[640px]:min-w-44">{task.title}</span>
        {task.note ? (
          // Gives way before the title does: shrinks down to the icon in narrow windows.
          <span className="flex min-w-7 max-w-56 shrink">
            <NoteBadge note={task.note} />
          </span>
        ) : null}
        {task.requirements.length > 0 ? (
          // Gives way before the title: shows the chips that fit and "+N" (DEB-04).
          <ChipList items={task.requirements} label={t("tasks.requirementsList", { list: task.requirements.join(", ") })} />
        ) : null}
        <span className="flex flex-none justify-center gap-1 font-code text-xs @[520px]:min-w-16">
          {task.commits.length > 0 ? (
            task.commits.map((hash) => <CommitChip key={hash} hash={hash} />)
          ) : (
            <span className="text-faint" aria-label={t("tasks.noHash")}>
              —
            </span>
          )}
        </span>
        {/* The gate level is also in the task panel; narrow cards drop it before anything overflows. */}
        <span className="hidden w-12 flex-none text-right @[520px]:block">{gate ? <Chip>{gate}</Chip> : null}</span>
      </button>
    </div>
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
        <div className="@container flex flex-col gap-1 px-3 pb-3">
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

/** Whether the checkboxes write, and why not when they cannot (CHK-01, CHK-04, SAV-11). */
function CheckboxState({ view }: { view: TasksView }) {
  const block = useCheckboxBlock(view);
  const changed = useTasks((s) => s.changedNotice);
  const toggleError = useTasks((s) => s.toggleError);
  const dismiss = useTasks((s) => s.dismissChanged);
  return (
    <>
      {!view.checkbox ? (
        <Chip icon={<Lock aria-hidden="true" className="size-3" />}>{t("tasks.readOnly")}</Chip>
      ) : block ? (
        <Chip tone="warn" icon={<Lock aria-hidden="true" className="size-3" />}>
          {block}
        </Chip>
      ) : (
        <Chip tone="acc">{t("tasks.checkboxOn")}</Chip>
      )}
      {changed || toggleError ? (
        <span role="status" className="flex items-center gap-1.5 text-[12px] text-warn">
          {toggleError ? <ErrorText error={toggleError} /> : t("tasks.changedOnDisk")}
          <IconButton label={t("live.dismiss")} icon={<X className="size-3" />} onClick={dismiss} className="size-5 text-warn" />
        </span>
      ) : null}
    </>
  );
}

/** `tasks.md` by phase (TSK-01..03); checkboxes only when the project opted in (CHK-01). */
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
        <CheckboxState view={view} />
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
