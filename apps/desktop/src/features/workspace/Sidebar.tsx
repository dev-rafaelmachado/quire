import { AlertTriangle, ChevronDown, ChevronRight, Folder, Layers, Search } from "lucide-react";
import { t } from "../../lib/i18n";
import type { ProjectSummary, SpecSummary } from "../../lib/ipc";
import { cx, ProgressBar, Segmented, type BarTone, type SegmentOption } from "../../ui";
import { filterProjects, specState, specTitle, type SpecState, type StatusFilter } from "./model";
import { ScanFooter } from "./ScanFooter";
import { useWorkspace } from "./store";

const STATE_LABEL = {
  complete: "specState.complete",
  inProgress: "specState.inProgress",
  pending: "specState.pending",
  noTasks: "specState.noTasks",
} as const;
const STATE_TEXT: Record<SpecState, string> = {
  complete: "text-ok",
  inProgress: "text-acc",
  pending: "text-faint",
  noTasks: "text-faint",
};
const STATE_BAR: Record<SpecState, BarTone> = { complete: "ok", inProgress: "acc", pending: "faint", noTasks: "faint" };

function SpecRow({ spec, selected, onSelect }: { spec: SpecSummary; selected: boolean; onSelect: () => void }) {
  const state = specState(spec);
  const progress = spec.instances[spec.defaultIndex]?.progress ?? null;
  return (
    <button
      type="button"
      aria-current={selected ? "true" : undefined}
      onClick={onSelect}
      className={cx("flex w-full flex-col gap-[5px] py-2 pr-3 pl-9 text-left", selected ? "bg-raise" : "hover:bg-panel")}
    >
      <span className="flex w-full items-center gap-2">
        <span className={cx("font-code text-xs", selected ? "text-acc" : "text-muted")}>{spec.issueKey ?? spec.folder}</span>
        {spec.issueKey ? (
          <span className={cx("min-w-0 flex-1 truncate", selected ? "font-semibold" : "font-medium")}>{specTitle(spec)}</span>
        ) : (
          <span className="flex-1" />
        )}
        {spec.versionsBadge ? (
          <span className="flex items-center gap-[3px] text-[11px] text-faint" aria-label={t("sidebar.versions", { n: spec.versionsBadge })}>
            <Layers aria-hidden="true" className="size-3" />
            <span aria-hidden="true">{spec.versionsBadge}</span>
          </span>
        ) : null}
      </span>
      <span className="flex items-center gap-2">
        <span className={cx("w-[74px] text-[11px]", STATE_TEXT[state])}>{t(STATE_LABEL[state])}</span>
        {progress ? (
          <>
            <ProgressBar
              done={progress[0]}
              total={progress[1]}
              tone={STATE_BAR[state]}
              label={t("progress.label", { done: progress[0], total: progress[1] })}
            />
            <span className="font-code text-[11px] text-faint" aria-hidden="true">
              {progress[0]}/{progress[1]}
            </span>
          </>
        ) : null}
      </span>
    </button>
  );
}

function ProjectGroup({ project }: { project: ProjectSummary }) {
  const collapsed = useWorkspace((s) => Boolean(s.collapsed[project.id]));
  const toggle = useWorkspace((s) => s.toggleProject);
  const selectedSpecId = useWorkspace((s) => s.selectedSpecId);
  const select = useWorkspace((s) => s.select);
  const name = project.alias ?? project.name;
  return (
    <div role="group" aria-label={name}>
      <button
        type="button"
        aria-expanded={!collapsed}
        aria-label={collapsed ? t("sidebar.expand", { name }) : t("sidebar.collapse", { name })}
        onClick={() => toggle(project.id)}
        className="flex h-8 w-full items-center gap-2 px-3 text-[13px] font-semibold hover:bg-panel"
      >
        {collapsed ? (
          <ChevronRight aria-hidden="true" className="size-3.5 text-faint" />
        ) : (
          <ChevronDown aria-hidden="true" className="size-3.5 text-faint" />
        )}
        <Folder aria-hidden="true" className="size-4 text-muted" />
        <span className="min-w-0 flex-1 truncate text-left" aria-hidden="true">
          {name}
        </span>
        {project.gitWarning ? (
          <AlertTriangle
            role="img"
            aria-label={t("sidebar.gitWarning", { message: project.gitWarning })}
            className="size-3.5 text-warn"
          />
        ) : null}
        <span className="text-[11px] font-medium text-faint" aria-hidden="true">
          {project.specs.length}
        </span>
      </button>
      {collapsed
        ? null
        : project.specs.map((spec) => (
            <SpecRow key={spec.id} spec={spec} selected={spec.id === selectedSpecId} onSelect={() => select(spec.id)} />
          ))}
    </div>
  );
}

export function Sidebar() {
  const projects = useWorkspace((s) => s.projects);
  const loaded = useWorkspace((s) => s.loaded);
  const text = useWorkspace((s) => s.text);
  const status = useWorkspace((s) => s.status);
  const setText = useWorkspace((s) => s.setText);
  const setStatus = useWorkspace((s) => s.setStatus);
  const visible = filterProjects(projects, text, status);
  const options: ReadonlyArray<SegmentOption<StatusFilter>> = [
    { value: "all", label: t("sidebar.statusAll") },
    { value: "inProgress", label: t("sidebar.statusInProgress") },
    { value: "done", label: t("sidebar.statusDone") },
  ];
  return (
    <>
      <div className="flex items-center gap-2 pt-2.5 pr-2.5 pb-1.5 pl-4">
        <h2 className="m-0 flex-1 text-[11px] font-semibold tracking-[.06em] text-faint uppercase">{t("sidebar.title")}</h2>
      </div>
      <label className="mx-3 flex h-[30px] items-center gap-2 rounded-[var(--radius-md)] border border-line bg-bg px-2.5 text-faint">
        <Search aria-hidden="true" className="size-3.5" />
        <input
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={t("sidebar.filterPlaceholder")}
          aria-label={t("sidebar.filterLabel")}
          className="min-w-0 flex-1 border-0 bg-transparent text-[12.5px] text-text outline-none"
        />
      </label>
      <div className="px-3 pt-2.5 pb-1.5">
        <Segmented label={t("sidebar.statusFilterLabel")} options={options} value={status} onChange={setStatus} />
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto pt-1">
        {visible.map((p) => (
          <ProjectGroup key={p.id} project={p} />
        ))}
        {loaded && visible.length === 0 ? (
          <p className="m-0 px-4 py-3 text-xs text-muted">
            {projects.length === 0 ? t("sidebar.empty") : t("sidebar.noMatch")}
          </p>
        ) : null}
      </div>
      <ScanFooter />
    </>
  );
}
