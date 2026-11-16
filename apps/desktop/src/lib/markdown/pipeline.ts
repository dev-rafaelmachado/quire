// Markdown -> hast (TD-04): unified + remark-gfm + remark-rehype. Raw HTML is dropped by
// remark-rehype, so requirement-id comments (`<!-- TOK-01 -->`) never reach the page.
import type { Element, ElementContent, Root, RootContent } from "hast";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { unified } from "unified";

const processor = unified().use(remarkParse).use(remarkGfm).use(remarkRehype);

export function toHast(md: string): Root {
  return processor.runSync(processor.parse(md));
}

export function textOf(node: RootContent | ElementContent | Root): string {
  if (node.type === "text") return node.value;
  if (node.type === "element" || node.type === "root") return node.children.map((c) => textOf(c)).join("");
  return "";
}

/** A heading with its content and nested sub-sections (VIW-05, VIW-06). */
export interface Section {
  id: string;
  depth: number;
  title: string;
  heading: Element;
  body: RootContent[];
  children: Section[];
}

export interface Doc {
  /** Content before the first heading. */
  intro: RootContent[];
  sections: Section[];
}

export interface OutlineEntry {
  id: string;
  depth: number;
  title: string;
}

function headingDepth(node: RootContent): number | null {
  if (node.type !== "element") return null;
  const m = /^h([1-6])$/.exec(node.tagName);
  return m ? Number(m[1]) : null;
}

function slug(text: string): string {
  return (
    text
      .normalize("NFD")
      .replace(/[̀-ͯ]/g, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "") || "secao"
  );
}

/** Groups the root children into nested sections by heading level. Ids are unique slugs. */
export function buildDoc(root: Root): Doc {
  const doc: Doc = { intro: [], sections: [] };
  const stack: Section[] = [];
  const used = new Map<string, number>();
  for (const node of root.children) {
    const depth = headingDepth(node);
    if (depth !== null && node.type === "element") {
      const title = textOf(node).trim();
      const base = `h-${slug(title)}`;
      const n = used.get(base) ?? 0;
      used.set(base, n + 1);
      const section: Section = { id: n === 0 ? base : `${base}-${n}`, depth, title, heading: node, body: [], children: [] };
      while (stack.length > 0 && (stack[stack.length - 1]?.depth ?? 0) >= depth) stack.pop();
      const parent = stack[stack.length - 1];
      if (parent) parent.children.push(section);
      else doc.sections.push(section);
      stack.push(section);
    } else {
      const top = stack[stack.length - 1];
      if (top) top.body.push(node);
      else doc.intro.push(node);
    }
  }
  return doc;
}

/** Headings for the outline, in document order, from `minDepth` to `maxDepth`. */
export function outline(doc: Doc, minDepth = 2, maxDepth = 3): OutlineEntry[] {
  const out: OutlineEntry[] = [];
  const walk = (sections: Section[]) => {
    for (const s of sections) {
      if (s.depth >= minDepth && s.depth <= maxDepth) out.push({ id: s.id, depth: s.depth, title: s.title });
      walk(s.children);
    }
  };
  walk(doc.sections);
  return out;
}

/** Ids of the section `id` and of all its ancestors (to expand before scrolling to it). */
export function ancestry(doc: Doc, id: string): string[] {
  const path: string[] = [];
  const find = (sections: Section[]): boolean => {
    for (const s of sections) {
      path.push(s.id);
      if (s.id === id || find(s.children)) return true;
      path.pop();
    }
    return false;
  };
  return find(doc.sections) ? path : [];
}

/** Language of a fenced block (`language-ts` -> `ts`) and its text. */
export function codeOf(pre: Element): { lang: string; code: string } | null {
  const code = pre.children.find((c): c is Element => c.type === "element" && c.tagName === "code");
  if (!code) return null;
  const classes = Array.isArray(code.properties.className) ? code.properties.className.map(String) : [];
  const lang = classes.find((c) => c.startsWith("language-"))?.slice("language-".length) ?? "";
  return { lang: lang.toLowerCase(), code: textOf(code).replace(/\n$/, "") };
}
