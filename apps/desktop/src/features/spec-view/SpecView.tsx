import { useEffect, useRef } from "react";
import { t } from "../../lib/i18n";
import { Outline } from "../../lib/markdown/Outline";
import { cx, ErrorText } from "../../ui";
import { GatesPanel } from "../logs-view/GatesPanel";
import { LessonsAside, LessonsPanel } from "../logs-view/Lessons";
import { useLogs } from "../logs-view/store";
import { TaskDetailPanel } from "../tasks-view/TaskDetailPanel";
import { TasksPanel } from "../tasks-view/TasksPanel";
import { useTasks } from "../tasks-view/store";
import { FileView } from "./FileView";
import { displayed } from "./labels";
import { MetaStrip } from "./MetaStrip";
import { OTHERS, useSpecView } from "./store";
import { Tabs } from "./Tabs";

/** File chooser of the "outros" tab. */
function OthersBar() {
  const detail = useSpecView((s) => s.detail);
  const file = useSpecView((s) => s.file);
  const show = useSpecView((s) => s.show);
  if (!detail) return null;
  return (
    <div role="group" aria-label={t("specView.othersFiles")} className="flex flex-none flex-wrap gap-1.5 border-b border-line px-6 py-2">
      {detail.others.map((name) => (
        <button
          key={name}
          type="button"
          aria-pressed={name === file}
          onClick={() => void show(name)}
          className={cx(
            "h-6 rounded-[var(--radius-md)] px-2 font-code text-xs",
            name === file ? "bg-acc-bg text-acc" : "bg-raise text-muted hover:text-text",
          )}
        >
          {name}
        </button>
      ))}
    </div>
  );
}

/** Center column: the rendered file of the open spec. */
export function SpecView() {
  const detail = useSpecView((s) => s.detail);
  const tab = useSpecView((s) => s.tab);
  const counts = useSpecView((s) => s.counts);
  const selectTab = useSpecView((s) => s.selectTab);
  const doc = useSpecView((s) => s.doc);
  const content = useSpecView((s) => s.content);
  const collapsed = useSpecView((s) => s.collapsed);
  const toggle = useSpecView((s) => s.toggle);
  const active = useSpecView((s) => s.active);
  const setActive = useSpecView((s) => s.setActive);
  const scrollRequest = useSpecView((s) => s.scrollRequest);
  const error = useSpecView((s) => s.error);
  const scroller = useRef<HTMLDivElement>(null);

  // Active section: the last heading whose top has passed the top of the scroll area (VIW-05).
  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    let frame = 0;
    const update = () => {
      frame = 0;
      const top = el.getBoundingClientRect().top + 24;
      let current: string | null = null;
      for (const h of el.querySelectorAll<HTMLElement>(".md-heading[id]")) {
        if (h.getBoundingClientRect().top <= top) current = h.id;
        else break;
      }
      if (current !== useSpecView.getState().active) setActive(current);
    };
    const onScroll = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    update();
    return () => {
      el.removeEventListener("scroll", onScroll);
      if (frame) cancelAnimationFrame(frame);
    };
  }, [doc, collapsed, setActive]);

  useEffect(() => {
    if (!scrollRequest || !active) return;
    document.getElementById(active)?.scrollIntoView({ block: "start" });
  }, [scrollRequest, active]);

  const tasksView = useTasks((s) => s.view);
  const loadTasks = useTasks((s) => s.load);
  const instanceId = detail ? displayed(detail)?.id : undefined;
  const onTasks = tab === "tasks.md";
  useEffect(() => {
    if (onTasks && instanceId !== undefined) void loadTasks(instanceId);
  }, [onTasks, instanceId, loadTasks]);

  const logsGates = useLogs((s) => s.gates);
  const logsLessons = useLogs((s) => s.lessons);
  const loadLogs = useLogs((s) => s.load);
  const onLogs = tab === "gates.jsonl" || tab === "lessons.jsonl";
  const hasGates = detail?.tabs.includes("gates.jsonl") ?? false;
  const hasLessons = detail?.tabs.includes("lessons.jsonl") ?? false;
  useEffect(() => {
    if (onLogs && instanceId !== undefined) void loadLogs(instanceId, { gates: hasGates, lessons: hasLessons });
  }, [onLogs, instanceId, hasGates, hasLessons, loadLogs]);

  if (detail && onLogs && (tab === "gates.jsonl" ? logsGates : logsLessons)) {
    return (
      <>
        <Tabs detail={detail} counts={counts} tab={tab} onSelect={(next) => void selectTab(next)} />
        <div role="tabpanel" className="flex min-h-0 flex-1 flex-col">
          {tab === "gates.jsonl" && logsGates ? <GatesPanel gates={logsGates} /> : null}
          {tab === "lessons.jsonl" && logsLessons ? <LessonsPanel lessons={logsLessons} /> : null}
        </div>
      </>
    );
  }

  if (detail && onTasks && tasksView?.structured) {
    return (
      <>
        <Tabs detail={detail} counts={counts} tab={tab} onSelect={(next) => void selectTab(next)} />
        <div role="tabpanel" className="flex min-h-0 flex-1 flex-col">
          <TasksPanel view={tasksView} />
        </div>
      </>
    );
  }

  return (
    <>
      {detail ? <Tabs detail={detail} counts={counts} tab={tab} onSelect={(next) => void selectTab(next)} /> : null}
      {detail && tab !== OTHERS ? <MetaStrip detail={detail} /> : null}
      {tab === OTHERS ? <OthersBar /> : null}
      <div ref={scroller} role="tabpanel" className="min-h-0 flex-1 overflow-y-auto">
        <div className="px-10 pt-9 pb-24">
          {error ? (
            <p role="alert" className="m-0 text-warn">
              <ErrorText error={error} />
            </p>
          ) : null}
          {content ? <FileView content={content} doc={doc} collapsed={collapsed} onToggle={toggle} /> : null}
          {detail && detail.tabs.length === 0 && detail.others.length === 0 ? (
            <p className="m-0 text-muted">{t("specView.noFiles")}</p>
          ) : null}
        </div>
      </div>
    </>
  );
}

/** Which right column the open tab uses (design README: 248-320 px, contextual). */
export function useRightColumn(): { kind: "tasks" | "lessons" | "outline" | null; width: 248 | 320 } {
  const tab = useSpecView((s) => s.tab);
  const hasOutline = useSpecView((s) => s.outline.length > 0);
  const structured = useTasks((s) => s.view?.structured ?? false);
  const lessons = useLogs((s) => s.lessons);
  if (tab === "tasks.md") return { kind: structured ? "tasks" : hasOutline ? "outline" : null, width: structured ? 320 : 248 };
  if (tab === "gates.jsonl") return { kind: lessons ? "lessons" : null, width: 320 };
  if (tab === "lessons.jsonl") return { kind: null, width: 248 };
  return { kind: hasOutline ? "outline" : null, width: 248 };
}

/** Right column: task detail, lessons beside the gates, or the outline of the file. */
export function SpecRight() {
  const { kind } = useRightColumn();
  const lessons = useLogs((s) => s.lessons);
  const selectTab = useSpecView((s) => s.selectTab);
  if (kind === "tasks") return <TaskDetailPanel />;
  if (kind === "lessons" && lessons) return <LessonsAside lessons={lessons} onOpen={() => void selectTab("lessons.jsonl")} />;
  return <SpecOutline />;
}

/** Right column: outline of the file. */
export function SpecOutline() {
  const entries = useSpecView((s) => s.outline);
  const active = useSpecView((s) => s.active);
  const navigate = useSpecView((s) => s.navigate);
  if (entries.length === 0) return null;
  return <Outline entries={entries} active={active} onNavigate={navigate} />;
}
