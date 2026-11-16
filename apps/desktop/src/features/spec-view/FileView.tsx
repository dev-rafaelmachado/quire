import { AlertTriangle, FileWarning } from "lucide-react";
import { formatNumber, t } from "../../lib/i18n";
import type { FileContent } from "../../lib/ipc";
import { CodeBlock } from "../../lib/markdown/CodeBlock";
import { Markdown } from "../../lib/markdown/Markdown";
import type { Doc } from "../../lib/markdown/pipeline";

interface FileViewProps {
  content: FileContent;
  doc: Doc | null;
  collapsed: ReadonlySet<string>;
  onToggle: (id: string) => void;
}

/** Any file of the folder by kind (VIW-08): markdown, highlighted JSON, plain text or a notice
 *  for binaries. Files above 2 MB show a warning and skip highlighting. */
export function FileView({ content, doc, collapsed, onToggle }: FileViewProps) {
  const banner = content.large ? (
    <p role="status" className="mb-4 flex items-center gap-2 rounded-[var(--radius-md)] bg-warn-bg px-3 py-2 text-[12.5px] text-warn">
      <AlertTriangle aria-hidden="true" className="size-4" />
      {t("specView.large", { size: formatNumber(content.size / (1024 * 1024), 1) })}
    </p>
  ) : null;
  if (content.text === null) {
    return (
      <p className="flex items-center gap-2 text-muted">
        <FileWarning aria-hidden="true" className="size-4" />
        {t("specView.binary", { name: content.name })}
      </p>
    );
  }
  if (content.text.trim() === "") return <p className="text-muted">{t("specView.emptyFile")}</p>;
  if (content.kind === "markdown" && doc) {
    return (
      <>
        {banner}
        <Markdown doc={doc} collapsed={collapsed} onToggle={onToggle} plain={content.large} />
      </>
    );
  }
  const lang = content.kind === "json" ? "json" : content.kind === "jsonl" ? "jsonl" : "";
  return (
    <div className="max-w-[960px]">
      {banner}
      <CodeBlock code={content.text} lang={lang} plain={content.large || lang === ""} />
    </div>
  );
}
