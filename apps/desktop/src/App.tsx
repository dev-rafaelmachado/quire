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
import { useEditor } from "./features/editor/store";
import { t, useLocale } from "./lib/i18n";
import { setWindowTitle, onWindowClose } from "./lib/window";
import { guard, hasUnsaved } from "./features/unsaved/store";
import { UnsavedDialog } from "./features/unsaved/UnsavedDialog";
import { ConflictDialog } from "./features/conflict/ConflictDialog";
import { startConflictDemo } from "./features/conflict/demo";
import { SearchPanel } from "./features/search/SearchPanel";
import { Palette, usePaletteShortcut } from "./features/palette/Palette";
import { useSearch } from "./features/search/store";

export function App() {
  // Subscribing here re-renders the whole tree when the language changes (no restart).
  useLocale((s) => s.locale);
  const init = useWorkspace((s) => s.init);
  const projects = useWorkspace((s) => s.projects);
  const selectedSpecId = useWorkspace((s) => s.selectedSpecId);
  const open = useSpecView((s) => s.open);
  const right = useRightColumn();
  const prefsOpen = useWorkspace((s) => s.prefsOpen);
  const setPrefsOpen = useWorkspace((s) => s.setPrefsOpen);
  const config = useWorkspace((s) => s.config);
  const showPrefs = prefsOpen && config !== null;
  const searchOpen = useSearch((s) => s.open);
  const selected = findSelected(projects, selectedSpecId);

  useEffect(() => {
    void init();
  }, [init]);
  useLiveUpdates();

  // EDT-06: the window title shows the unsaved mark too.
  const dirtyName = useEditor((s) => (s.dirty && s.buffer ? s.buffer.name : null));
  useEffect(() => {
    setWindowTitle(dirtyName ? `• ${dirtyName} — ${t("app.name")}` : t("app.name"));
  }, [dirtyName]);

  useEffect(() => startConflictDemo(), []);
  usePaletteShortcut();

  // Ctrl+Shift+F opens the search (SRC); the palette (Ctrl+K) reaches it too.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "f") {
        e.preventDefault();
        // Opening the search replaces the center column: same guard as any navigation.
        guard(() => useSearch.getState().setOpen(true));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Closing the window with unsaved changes asks first (SAV-06).
  useEffect(() => onWindowClose(hasUnsaved, (close) => guard(close)), []);
  useCliNavigation();

  useEffect(() => {
    void open(selectedSpecId);
  }, [open, selectedSpecId]);

  return (
    <Shell
      top={
        <TopBar
          crumbs={selected ? { project: selected.project.alias ?? selected.project.name, spec: selected.spec.folder } : undefined}
          actions={selected && !showPrefs && !searchOpen ? <InstanceBadge /> : null}
          onPreferences={() => guard(() => setPrefsOpen(!prefsOpen))}
          onSearch={() => (searchOpen ? useSearch.getState().setOpen(false) : guard(() => useSearch.getState().setOpen(true)))}
        />
      }
      side={<Sidebar />}
      main={
        <>
          <Palette />
          <UnsavedDialog />
          <ConflictDialog />
          <Notice />
          <CliPicker />
          {searchOpen ? <SearchPanel /> : showPrefs ? <Preferences config={config} /> : selected ? <SpecView /> : <EmptyView />}
        </>
      }
      right={selected && right.kind && !showPrefs && !searchOpen ? <SpecRight /> : undefined}
      rightWidth={right.width}
      status={<WorkspaceStatus />}
    />
  );
}
