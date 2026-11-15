import { t } from "../../lib/i18n";
import type { SpecSummary } from "../../lib/ipc";
import { Chip } from "../../ui";
import { specTitle } from "./model";

/** Center column for a selected spec until the spec view lands (T17). */
export function SpecPlaceholder({ spec }: { spec: SpecSummary }) {
  return (
    <div className="mx-auto flex w-full max-w-[736px] flex-col gap-3 px-10 pt-9">
      <div className="flex items-center gap-2">
        {spec.issueKey ? (
          <Chip tone="acc" mono>
            {spec.issueKey}
          </Chip>
        ) : null}
        <Chip>{t("main.instances", { n: spec.instances.length })}</Chip>
      </div>
      <h1 className="m-0 text-[28px] leading-tight font-semibold tracking-tight">{specTitle(spec)}</h1>
      <p className="m-0 text-muted">{t("main.placeholder")}</p>
    </div>
  );
}
