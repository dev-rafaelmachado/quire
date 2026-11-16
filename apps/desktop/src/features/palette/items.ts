import { useTheme } from "../../app/theme";
import { t } from "../../lib/i18n";
import { useEditor } from "../editor/store";
import { useSearch } from "../search/store";
import { displayed, tabLabel, worktreeName } from "../spec-view/labels";
import { OTHERS, useSpecView } from "../spec-view/store";
import { guard } from "../unsaved/store";
import { specTitle } from "../workspace/model";
import { useWorkspace } from "../workspace/store";

export interface PaletteItem {
  id: string;
  label: string;
  hint?: string;
  group: "spec" | "command";
  fields: ReadonlyArray<{ text: string; weight: number }>;
  run: () => void;
}

/** Everything the palette can do right now (CMD-02). Navigations go through the unsaved
 *  changes guard (CMD-05). */
export function paletteItems(): PaletteItem[] {
  const items: PaletteItem[] = [];
  const ws = useWorkspace.getState();
  for (const p of ws.projects) {
    for (const s of p.specs) {
      const title = s.issueKey ? specTitle(s) : null;
      const key = s.issueKey ?? s.folder;
      items.push({
        id: `spec:${s.id}`,
        label: title ? `${key} ${title}` : key,
        hint: p.alias ?? p.name,
        group: "spec",
        fields: [
          { text: key, weight: 2 },
          { text: s.folder, weight: 1 },
          { text: title ?? "", weight: 1 },
        ],
        run: () =>
          guard(() => {
            useSearch.getState().setOpen(false);
            ws.setPrefsOpen(false);
            ws.select(s.id);
          }),
      });
    }
  }
  const command = (id: string, label: string, run: () => void, hint?: string): PaletteItem => ({
    id,
    label,
    group: "command",
    fields: [{ text: label, weight: 1 }],
    run,
    ...(hint !== undefined ? { hint } : {}),
  });
  const view = useSpecView.getState();
  const detail = view.detail;
  if (detail) {
    for (const file of detail.tabs) {
      items.push(command(`tab:${file}`, t("palette.tab", { name: tabLabel(file) }), () => guard(() => view.selectTab(file))));
    }
    if (detail.others.length > 0) {
      items.push(command("tab:others", t("palette.tab", { name: t("specView.others") }), () => guard(() => view.selectTab(OTHERS))));
    }
    const shown = displayed(detail);
    for (const i of detail.spec.instances) {
      if (i.id === shown?.id) continue;
      const branch = i.branch ?? t("specView.detached");
      items.push(command(`instance:${i.id}`, t("palette.instance", { branch, worktree: worktreeName(i) }), () => guard(() => view.switchInstance(i.id))));
    }
  }
  const editor = useEditor.getState();
  if (editor.buffer && !editor.buffer.locked) {
    items.push(command("mode:read", t("palette.mode", { mode: t("editor.read") }), () => guard(() => useEditor.getState().setMode("read"))));
    items.push(command("mode:edit", t("palette.mode", { mode: t("editor.edit") }), () => useEditor.getState().setMode("edit")));
    items.push(command("mode:split", t("palette.mode", { mode: t("editor.split") }), () => useEditor.getState().setMode("split")));
    if (editor.mode !== "read") items.push(command("save", t("palette.save"), () => void useEditor.getState().save(), "Ctrl+S"));
  }
  items.push(command("search", t("palette.searchContent"), () => undefined, "/"));
  items.push(command("theme", t("palette.theme"), () => useTheme.getState().toggle()));
  items.push(
    command("prefs", t("palette.preferences"), () =>
      guard(() => {
        useSearch.getState().setOpen(false);
        ws.setPrefsOpen(true);
      }),
    ),
  );
  return items;
}
