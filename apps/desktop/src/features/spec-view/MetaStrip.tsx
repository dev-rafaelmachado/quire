import { Check, ExternalLink } from "lucide-react";
import { t } from "../../lib/i18n";
import { commands, type SpecDetail } from "../../lib/ipc";
import { Chip, type Tone } from "../../ui";
import { issueUrl } from "./labels";

function statusTone(part: string): Tone {
  if (/conclu|aprovad|pass|done/i.test(part)) return "ok";
  if (/fail|bloque|erro|block/i.test(part)) return "warn";
  return "neutral";
}

/** Header fields as chips and the issue link (VIW-07). */
export function MetaStrip({ detail }: { detail: SpecDetail }) {
  const url = issueUrl(detail);
  const key = detail.spec.issueKey;
  const fields = detail.header.filter((f) => !["status", "issue"].includes(f.key.toLowerCase()));
  const status = detail.header.find((f) => f.key.toLowerCase() === "status");
  const parts = status ? status.text.split(/\s+·\s+/).filter(Boolean) : [];
  if (parts.length === 0 && fields.length === 0 && !url) return null;
  return (
    <div aria-label={t("specView.metaLabel")} role="group" className="flex min-h-12 flex-none flex-wrap items-center gap-2.5 border-b border-line px-6 py-2">
      {parts.map((p, i) => {
        const tone = i === 0 ? statusTone(p) : "neutral";
        return (
          <Chip key={p} tone={tone} icon={tone === "ok" ? <Check aria-hidden="true" className="size-3" /> : undefined}>
            {p}
          </Chip>
        );
      })}
      {fields.map((f) => (
        <Chip key={f.key}>{t("specView.field", { key: f.key, value: f.text })}</Chip>
      ))}
      {url ? (
        <a
          href={url}
          onClick={(e) => {
            e.preventDefault();
            void commands.openExternal(url);
          }}
          aria-label={t("specView.openIssue", { key: key ?? url })}
          className="inline-flex items-center gap-1 font-code text-acc hover:underline"
        >
          {key ?? url}
          <ExternalLink aria-hidden="true" className="size-3" />
        </a>
      ) : null}
    </div>
  );
}
