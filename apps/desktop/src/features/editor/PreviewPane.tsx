import { type Ref, useMemo, useState } from "react";
import { t } from "../../lib/i18n";
import { stripHeaderMapped } from "../../lib/markdown/header";
import { Markdown } from "../../lib/markdown/Markdown";
import { buildDoc, toHast } from "../../lib/markdown/pipeline";

/** The rendered side of Dividir: the same pipeline as Ler (EDT-03). */
export function PreviewPane({ name, text, header, scrollRef }: { name: string; text: string; header: boolean; scrollRef?: Ref<HTMLElement> }) {
  // `data-line` keeps the file's line numbers even without the header block (EDT-04).
  const doc = useMemo(() => {
    if (!header) return buildDoc(toHast(text));
    const { text: shown, mapLine } = stripHeaderMapped(text);
    return buildDoc(toHast(shown, mapLine));
  }, [text, header]);
  const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(new Set());
  const toggle = (id: string) =>
    setCollapsed((c) => {
      const next = new Set(c);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  return (
    <section ref={scrollRef} aria-label={t("editor.preview", { name })} className="h-full min-h-0 overflow-y-auto border-l border-line">
      <div className="px-8 pt-6 pb-24">
        <Markdown doc={doc} collapsed={collapsed} onToggle={toggle} />
      </div>
    </section>
  );
}
