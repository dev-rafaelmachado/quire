import { useEffect } from "react";
import { EmptyView } from "./app/EmptyView";
import { Shell } from "./app/Shell";
import { TopBar } from "./app/TopBar";
import { InstanceBadge } from "./features/spec-view/InstanceBadge";
import { SpecRight, SpecView } from "./features/spec-view/SpecView";
import { useTasks } from "./features/tasks-view/store";
import { useSpecView } from "./features/spec-view/store";
import { Sidebar } from "./features/workspace/Sidebar";
import { findSelected } from "./features/workspace/selection";
import { useWorkspace } from "./features/workspace/store";
import { WorkspaceStatus } from "./features/workspace/WorkspaceStatus";

export function App() {
  const init = useWorkspace((s) => s.init);
  const projects = useWorkspace((s) => s.projects);
  const selectedSpecId = useWorkspace((s) => s.selectedSpecId);
  const open = useSpecView((s) => s.open);
  const hasOutline = useSpecView((s) => s.outline.length > 0);
  const onTasks = useSpecView((s) => s.tab === "tasks.md");
  const tasksStructured = useTasks((s) => s.view?.structured ?? false);
  const hasRight = onTasks ? tasksStructured : hasOutline;
  const selected = findSelected(projects, selectedSpecId);

  useEffect(() => {
    void init();
  }, [init]);

  useEffect(() => {
    void open(selectedSpecId);
  }, [open, selectedSpecId]);

  return (
    <Shell
      top={
        <TopBar
          crumbs={selected ? { project: selected.project.alias ?? selected.project.name, spec: selected.spec.folder } : undefined}
          actions={selected ? <InstanceBadge /> : null}
        />
      }
      side={<Sidebar />}
      main={selected ? <SpecView /> : <EmptyView />}
      right={selected && hasRight ? <SpecRight /> : undefined}
      rightWidth={onTasks ? 320 : 248}
      status={<WorkspaceStatus />}
    />
  );
}
