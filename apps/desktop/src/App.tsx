import { useEffect } from "react";
import { EmptyView } from "./app/EmptyView";
import { Shell } from "./app/Shell";
import { TopBar } from "./app/TopBar";
import { Sidebar } from "./features/workspace/Sidebar";
import { SpecPlaceholder } from "./features/workspace/SpecPlaceholder";
import { findSelected } from "./features/workspace/selection";
import { useWorkspace } from "./features/workspace/store";
import { WorkspaceStatus } from "./features/workspace/WorkspaceStatus";

export function App() {
  const init = useWorkspace((s) => s.init);
  const projects = useWorkspace((s) => s.projects);
  const selectedSpecId = useWorkspace((s) => s.selectedSpecId);
  const selected = findSelected(projects, selectedSpecId);

  useEffect(() => {
    void init();
  }, [init]);

  return (
    <Shell
      top={
        <TopBar
          crumbs={selected ? { project: selected.project.alias ?? selected.project.name, spec: selected.spec.folder } : undefined}
        />
      }
      side={<Sidebar />}
      main={selected ? <SpecPlaceholder spec={selected.spec} /> : <EmptyView />}
      status={<WorkspaceStatus />}
    />
  );
}
