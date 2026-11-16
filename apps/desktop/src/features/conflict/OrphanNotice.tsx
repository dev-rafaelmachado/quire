import { FileWarning } from "lucide-react";
import { useEffect, useState } from "react";
import { t } from "../../lib/i18n";
import { asCmdError, type CmdError, commands } from "../../lib/ipc";
import { Button, ErrorText } from "../../ui";

/** Temporary files left by an interrupted save (SAV-10): listed for the instance on screen,
 *  never deleted on their own; each goes only on an explicit click. */
export function OrphanNotice({ instanceId, version }: { instanceId: number; version: unknown }) {
  const [names, setNames] = useState<string[]>([]);
  const [error, setError] = useState<CmdError | null>(null);
  useEffect(() => {
    let alive = true;
    commands.orphanTemps(instanceId).then(
      (list) => alive && setNames(list),
      () => alive && setNames([]),
    );
    return () => {
      alive = false;
    };
  }, [instanceId, version]);
  if (names.length === 0) return null;
  const remove = async (name: string) => {
    try {
      await commands.removeOrphanTemp(instanceId, name);
      setNames((n) => n.filter((x) => x !== name));
      setError(null);
    } catch (e) {
      setError(asCmdError(e));
    }
  };
  return (
    <div role="status" className="flex flex-none flex-col gap-1 border-b border-line bg-warn-bg px-6 py-2 text-[12.5px] text-warn">
      <span className="flex items-center gap-2">
        <FileWarning aria-hidden="true" className="size-4 flex-none" />
        {t("conflict.orphans", { n: names.length })}
      </span>
      {names.map((name) => (
        <span key={name} className="flex items-center gap-2 pl-6">
          <code className="min-w-0 flex-1 truncate font-code text-[11.5px]">{name}</code>
          <Button onClick={() => void remove(name)}>{t("conflict.removeOrphan", { name })}</Button>
        </span>
      ))}
      {error ? <ErrorText error={error} /> : null}
    </div>
  );
}
