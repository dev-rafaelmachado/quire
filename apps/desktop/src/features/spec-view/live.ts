import { useEffect } from "react";
import { t } from "../../lib/i18n";
import { events, type InstanceSummary, type ProjectSummary, type SpecDetail } from "../../lib/ipc";
import { useConflict } from "../conflict/store";
import { useEditor } from "../editor/store";
import { useLogs } from "../logs-view/store";
import { useTasks } from "../tasks-view/store";
import { useWorkspace } from "../workspace/store";
import { displayed, worktreeName } from "./labels";
import { useSpecView } from "./store";

/** Comparable form of a path: `/` separators, no trailing one, no Windows `\\?\` prefix. */
const norm = (p: string) =>
  p
    .replace(/\\/g, "/")
    .replace(/^\/\/\?\/UNC\//, "//")
    .replace(/^\/\/\?\//, "")
    .replace(/\/+$/, "");

/** Whether a `spec://changed` batch touches the instance on screen. */
export function affects(dirs: readonly string[], detail: SpecDetail | null): boolean {
  const shown = detail ? displayed(detail) : undefined;
  if (!shown) return false;
  const path = norm(shown.path);
  return dirs.some((d) => norm(d) === path);
}

export type SelectionFix = { kind: "none" } | { kind: "removed" } | { kind: "switch"; to: InstanceSummary };

/** After the list changes: is the open spec, and the copy on screen, still there? (WAT-04) */
export function reconcileSelection(
  projects: readonly ProjectSummary[],
  specId: number | null,
  detail: SpecDetail | null,
): SelectionFix {
  if (specId === null) return { kind: "none" };
  const spec = projects.flatMap((p) => p.specs).find((s) => s.id === specId);
  if (!spec) return { kind: "removed" };
  const shown = detail ? displayed(detail) : undefined;
  if (!shown || spec.instances.some((i) => i.id === shown.id)) return { kind: "none" };
  const to = spec.instances[spec.defaultIndex] ?? spec.instances[0];
  return to ? { kind: "switch", to } : { kind: "removed" };
}

/** Reacts to changes on disk while the app is open (WAT-01, WAT-02, WAT-04). */
export function useLiveUpdates(): void {
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    void events.specChanged
      .listen((e) => {
        const view = useSpecView.getState();
        if (!affects(e.payload.specDirs, view.detail)) return;
        void view.refresh();
        // The buffer follows the TD-14 rule: its own save is ignored, an outside change reloads
        // a clean buffer and only warns over a dirty one (CNF-04..07).
        void useConflict.getState().onDiskChange();
        const id = view.detail ? displayed(view.detail)?.id : undefined;
        if (id !== undefined && useTasks.getState().instanceId === id) void useTasks.getState().reload();
        if (id !== undefined && useLogs.getState().instanceId === id) void useLogs.getState().reload();
      })
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      });
    // After the app's own save the read view and the tasks follow at once (PRG-01).
    const unsubscribeSaves = useEditor.subscribe((state, prev) => {
      if (state.lastSaved === prev.lastSaved) return;
      const view = useSpecView.getState();
      void view.refresh();
      const id = view.detail ? displayed(view.detail)?.id : undefined;
      if (id !== undefined && useTasks.getState().instanceId === id) void useTasks.getState().reload();
    });
    const unsubscribe = useWorkspace.subscribe((state, prev) => {
      if (state.projects === prev.projects || !state.loaded) return;
      const view = useSpecView.getState();
      const fix = reconcileSelection(state.projects, view.specId, view.detail);
      if (fix.kind === "removed") {
        useWorkspace.getState().select(null);
        view.setNotice(t("live.specRemoved"));
      } else if (fix.kind === "switch") {
        void view.switchInstance(fix.to.id).then(() =>
          useSpecView.getState().setNotice(
            t("live.instanceRemoved", { branch: fix.to.branch ?? t("specView.detached"), worktree: worktreeName(fix.to) }),
          ),
        );
      }
    });
    return () => {
      cancelled = true;
      unlisten?.();
      unsubscribe();
      unsubscribeSaves();
    };
  }, []);
}
