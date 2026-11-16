import { useMemo, useState } from "react";
import { t } from "../../lib/i18n";
import { stripHeader } from "../../lib/markdown/header";
import { Markdown } from "../../lib/markdown/Markdown";
import { buildDoc, toHast } from "../../lib/markdown/pipeline";

/** The rendered side of Dividir: the same pipeline as Ler (EDT-03). */
export function PreviewPane({ name, text, header }: { name: string; text: string; header: boolean }) {
  const doc = useMemo(() => buildDoc(toHast(header ? stripHeader(text) : text)), [text, header]);
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());
  const toggle = (id: string) =>
    setCollapsed((c) => {
      const next = new Set(c);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  return (
    <section aria-label={t("editor.preview", { name })} className="h-full min-h-0 overflow-y-auto border-l border-line">
      <div className="px-8 pt-6 pb-24">
        <Markdown doc={doc} collapsed={collapsed} onToggle={toggle} />
      </div>
    </section>
  );
}
