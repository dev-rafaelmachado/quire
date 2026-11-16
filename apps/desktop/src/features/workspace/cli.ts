import { useEffect } from "react";
import { t } from "../../lib/i18n";
import { type CliOpen, commands, events } from "../../lib/ipc";
import { useSpecView } from "../spec-view/store";
import { useWorkspace } from "./store";

/** Applies navigation asked on the command line (CLI-01..04). */
export function applyCliOpen(open: CliOpen): void {
  const ws = useWorkspace.getState();
  const view = useSpecView.getState();
  switch (open.kind) {
    case "spec": {
      const instanceId = open.instanceId ?? null;
      const file = open.file ?? null;
      if (instanceId !== null) {
        useSpecView.setState({ chosen: { ...view.chosen, [open.specId]: instanceId } });
      }
      ws.setHighlight(null);
      if (ws.selectedSpecId === open.specId) {
        const current = view.detail?.spec.instances[view.detail.instanceIndex]?.id;
        const switching = instanceId !== null && instanceId !== current ? view.switchInstance(instanceId) : Promise.resolve();
        if (file !== null) void switching.then(() => useSpecView.getState().showFile(file));
        else void switching;
      } else {
        useSpecView.setState({ requestedTab: file });
        ws.select(open.specId);
      }
      break;
    }
    case "highlight":
      ws.select(null);
      ws.setText("");
      ws.setStatus("all");
      ws.setHighlight({ query: open.query, specIds: open.specIds });
      break;
    case "pick":
      ws.setPick({ query: open.query, specIds: open.specIds });
      break;
    case "notFound":
      view.setNotice(t("cli.notFound", { query: open.query }));
      break;
  }
}

/** Listens for `cli://open` and takes a navigation that arrived before the UI was ready. */
export function useCliNavigation(): void {
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    void events.cliOpen
      .listen((e) => {
        applyCliOpen(e.payload.open);
        void commands.takeCliOpen();
      })
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      });
    void commands.takeCliOpen().then((pending) => {
      if (pending && !cancelled) applyCliOpen(pending);
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
}
