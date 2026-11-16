import type { Element, RootContent } from "hast";
import { type Components, toJsxRuntime } from "hast-util-to-jsx-runtime";
import { ChevronDown, ChevronRight } from "lucide-react";
import { type MouseEvent, type ReactNode, memo, useMemo } from "react";
import { Fragment, jsx, jsxs } from "react/jsx-runtime";
import { t, useLocale } from "../i18n";
import { commands } from "../ipc";
import { CodeBlock, MermaidBlock } from "./CodeBlock";
import { codeOf, type Doc, type Section } from "./pipeline";

function openLink(e: MouseEvent<HTMLAnchorElement>, href: string | undefined): void {
  // The webview must never navigate away from the app.
  e.preventDefault();
  if (href && /^https?:\/\//i.test(href)) void commands.openExternal(href);
}

function makeComponents(plain: boolean): Partial<Components> {
  return {
    pre: (props) => {
      const node = (props as { node?: Element }).node;
      const info = node ? codeOf(node) : null;
      if (info?.lang === "mermaid") return <MermaidBlock code={info.code} />;
      if (info) return <CodeBlock code={info.code} lang={info.lang} plain={plain} />;
      return <pre>{props.children}</pre>;
    },
    a: ({ href, children }) => {
      const external = typeof href === "string" && /^https?:\/\//i.test(href);
      return (
        <a
          href={href}
          onClick={(e) => openLink(e, href)}
          title={external ? t("markdown.externalLink", { href }) : href}
        >
          {children}
        </a>
      );
    },
    table: ({ children }) => (
      <div className="md-table">
        <table>{children}</table>
      </div>
    ),
  };
}

// Stable identities: new component functions on every render would make React remount every
// code block and diagram (re-highlight, re-render) whenever a section is toggled.
const RICH = makeComponents(false);
const PLAIN = makeComponents(true);

function render(nodes: RootContent[], plain: boolean): ReactNode {
  return toJsxRuntime({ type: "root", children: nodes }, { Fragment, jsx, jsxs, components: plain ? PLAIN : RICH, passNode: true });
}

interface MarkdownProps {
  doc: Doc;
  /** Ids of collapsed sections (VIW-06). */
  collapsed: ReadonlySet<string>;
  onToggle: (id: string) => void;
  /** Disables highlighting (very large files). */
  plain?: boolean;
}

const SectionView = memo(function SectionView({
  section,
  collapsed,
  onToggle,
  plain,
}: {
  section: Section;
  collapsed: ReadonlySet<string>;
  onToggle: (id: string) => void;
  plain: boolean;
}) {
  useLocale((s) => s.locale); // memo would otherwise keep the labels of the old language
  const closed = collapsed.has(section.id);
  const heading = useMemo(() => render(section.heading.children as RootContent[], plain), [section, plain]);
  const body = useMemo(() => render(section.body, plain), [section, plain]);
  const Tag = `h${section.depth}` as "h1";
  const toggle =
    section.depth >= 2 ? (
      <button
        type="button"
        className="md-toggle"
        aria-expanded={!closed}
        aria-controls={`${section.id}-body`}
        aria-label={closed ? t("markdown.expand", { title: section.title }) : t("markdown.collapse", { title: section.title })}
        onClick={() => onToggle(section.id)}
      >
        {closed ? <ChevronRight className="size-3.5" /> : <ChevronDown className="size-3.5" />}
      </button>
    ) : null;
  return (
    <section className="md-section" data-collapsed={closed || undefined}>
      <Tag id={section.id} className="md-heading" data-line={section.heading.properties["dataLine"] as number | undefined}>
        {toggle}
        {heading}
      </Tag>
      {/* Hidden, not unmounted: expanding must not re-highlight code or re-render diagrams. */}
      <div id={`${section.id}-body`} hidden={closed}>
        {body}
        {section.children.map((c) => (
          <SectionView key={c.id} section={c} collapsed={collapsed} onToggle={onToggle} plain={plain} />
        ))}
      </div>
    </section>
  );
});

/** Rendered markdown (VIW-02..06) inside the ~736 px reading column. */
export function Markdown({ doc, collapsed, onToggle, plain = false }: MarkdownProps) {
  const intro = useMemo(() => render(doc.intro, plain), [doc, plain]);
  return (
    <article className="md">
      {intro}
      {doc.sections.map((s) => (
        <SectionView key={s.id} section={s} collapsed={collapsed} onToggle={onToggle} plain={plain} />
      ))}
    </article>
  );
}
