import { TriangleAlert } from "lucide-react";
import { t } from "../../lib/i18n";
import { Button } from "../../ui";
import { useEditor } from "../editor/store";
import { worktreeName } from "../spec-view/labels";
import { useSpecView } from "../spec-view/store";
import { useConflict } from "./store";

/** The copy being edited was removed while the buffer has unsaved changes (INS-04): the buffer
 *  stays, nothing switches on its own, and the edits can go to another copy, against its
 *  content (the conflict dialog decides). */
export function GoneNotice() {
  const gone = useConflict((s) => s.gone);
  if (!gone) return null;
  const to = gone.to;
  const carry = () => {
    if (!to) return;
    useEditor.getState().carryTo(to.id);
    useConflict.getState().setGone(null);
    void useSpecView.getState().switchInstance(to.id);
  };
  return (
    <div role="status" className="flex flex-none flex-wrap items-center gap-2 border-b border-line bg-warn-bg px-6 py-1.5 text-[12.5px] text-warn">
      <TriangleAlert aria-hidden="true" className="size-4 flex-none" />
      <span className="min-w-0 flex-1">{t("instances.goneDirty")}</span>
      {to ? (
        <Button onClick={carry}>{t("instances.carryTo", { branch: to.branch ?? t("specView.detached"), worktree: worktreeName(to) })}</Button>
      ) : null}
    </div>
  );
}
