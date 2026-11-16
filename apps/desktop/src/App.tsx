import { useEffect } from "react";
import { EmptyView } from "./app/EmptyView";
import { Shell } from "./app/Shell";
import { TopBar } from "./app/TopBar";
import { InstanceBadge } from "./features/spec-view/InstanceBadge";
import { useLiveUpdates } from "./features/spec-view/live";
import { Notice } from "./features/spec-view/Notice";
import { SpecRight, SpecView, useRightColumn } from "./features/spec-view/SpecView";
import { useSpecView } from "./features/spec-view/store";
import { Preferences } from "./features/settings/Preferences";
import { useCliNavigation } from "./features/workspace/cli";
import { CliPicker } from "./features/workspace/CliPicker";
import { Sidebar } from "./features/workspace/Sidebar";
import { findSelected } from "./features/workspace/selection";
import { useWorkspace } from "./features/workspace/store";
import { WorkspaceStatus } from "./features/workspace/WorkspaceStatus";

export function App() {
  const init = useWorkspace((s) => s.init);
  const projects = useWorkspace((s) => s.projects);
  const selectedSpecId = useWorkspace((s) => s.selectedSpecId);
  const open = useSpecView((s) => s.open);
  const right = useRightColumn();
  const prefsOpen = useWorkspace((s) => s.prefsOpen);
  const setPrefsOpen = useWorkspace((s) => s.setPrefsOpen);
  const config = useWorkspace((s) => s.config);
  const showPrefs = prefsOpen && config !== null;
  const selected = findSelected(projects, selectedSpecId);

  useEffect(() => {
    void init();
  }, [init]);
  useLiveUpdates();
  useCliNavigation();

  useEffect(() => {
    void open(selectedSpecId);
  }, [open, selectedSpecId]);

  return (
    <Shell
      top={
        <TopBar
          crumbs={selected ? { project: selected.project.alias ?? selected.project.name, spec: selected.spec.folder } : undefined}
          actions={selected && !showPrefs ? <InstanceBadge /> : null}
          onPreferences={() => setPrefsOpen(!prefsOpen)}
        />
      }
      side={<Sidebar />}
      main={
        <>
          <Notice />
          <CliPicker />
          {showPrefs ? <Preferences config={config} /> : selected ? <SpecView /> : <EmptyView />}
        </>
      }
      right={selected && right.kind && !showPrefs ? <SpecRight /> : undefined}
      rightWidth={right.width}
      status={<WorkspaceStatus />}
    />
  );
}
