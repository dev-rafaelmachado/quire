// T2 spike (disposable): unified + remark-gfm + rehype -> React, shiki and mermaid on demand.
import type { Element, ElementContent, Root, RootContent } from "hast";
import { type Components, toJsxRuntime } from "hast-util-to-jsx-runtime";
import { type ReactNode, useEffect, useState } from "react";
import { Fragment, jsx, jsxs } from "react/jsx-runtime";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import type { HighlighterCore } from "shiki/core";
import { unified } from "unified";

// ---------- parse ----------

const processor = unified().use(remarkParse).use(remarkGfm).use(remarkRehype);

export function toHast(md: string): Root {
  return processor.runSync(processor.parse(md));
}

export interface Section {
  id: string;
  title: string;
  nodes: RootContent[];
}

export function textOf(node: RootContent | ElementContent): string {
  if (node.type === "text") return node.value;
  if (node.type === "element") return node.children.map(textOf).join("");
  return "";
}

/** Groups the root children into sections that start at each h2. */
export function splitSections(root: Root): { preamble: RootContent[]; sections: Section[] } {
  const preamble: RootContent[] = [];
  const sections: Section[] = [];
  for (const node of root.children) {
    if (node.type === "element" && node.tagName === "h2") {
      sections.push({ id: `s${sections.length}`, title: textOf(node), nodes: [] });
    } else {
      const current = sections.at(-1);
      (current ? current.nodes : preamble).push(node);
    }
  }
  return { preamble, sections };
}

interface CodeInfo {
  lang: string;
  code: string;
}

function codeInfo(pre: Element): CodeInfo | null {
  const codeEl = pre.children.find((c): c is Element => c.type === "element" && c.tagName === "code");
  if (!codeEl) return null;
  const classes = codeEl.properties.className;
  const list = Array.isArray(classes) ? classes.map(String) : [];
  const langClass = list.find((c) => c.startsWith("language-"));
  return { lang: langClass ? langClass.slice("language-".length) : "", code: textOf(codeEl).replace(/\n$/, "") };
}

export function countBlocks(root: Root): { highlight: number; mermaid: number; plain: number } {
  const counts = { highlight: 0, mermaid: 0, plain: 0 };
  const walk = (n: Root | RootContent | ElementContent): void => {
    if (n.type === "element" && n.tagName === "pre") {
      const info = codeInfo(n);
      if (info?.lang === "mermaid") counts.mermaid++;
      else if (info && info.lang in LANG_LOADERS) counts.highlight++;
      else counts.plain++;
      return;
    }
    if (n.type === "root" || n.type === "element") n.children.forEach(walk);
  };
  walk(root);
  return counts;
}

// ---------- progress tracking (for the metrics) ----------

type Listener = () => void;

export const tracker = {
  highlighted: 0,
  mermaidOk: 0,
  mermaidErr: 0,
  shikiInitMs: 0,
  mermaidImportMs: 0,
  mermaidDurations: [] as number[],
  listeners: new Set<Listener>(),
  bump(): void {
    this.listeners.forEach((l) => l());
  },
};

// ---------- shiki (on demand) ----------

const LANG_LOADERS: Record<string, () => Promise<unknown>> = {
  ts: () => import("shiki/langs/typescript.mjs"),
  rust: () => import("shiki/langs/rust.mjs"),
  json: () => import("shiki/langs/json.mjs"),
  bash: () => import("shiki/langs/bash.mjs"),
  sql: () => import("shiki/langs/sql.mjs"),
  toml: () => import("shiki/langs/toml.mjs"),
};

let highlighterPromise: Promise<HighlighterCore> | null = null;
const loadedLangs = new Map<string, Promise<void>>();

function getHighlighter(): Promise<HighlighterCore> {
  if (!highlighterPromise) {
    const t0 = performance.now();
    highlighterPromise = (async () => {
      // `?engine=oniguruma` compares the WASM engine against the default JS regex engine.
      const useOnig = new URLSearchParams(location.search).get("engine") === "oniguruma";
      const { createHighlighterCore } = await import("shiki/core");
      const engine = useOnig
        ? (await import("shiki/engine/oniguruma")).createOnigurumaEngine(import("shiki/wasm"))
        : (await import("shiki/engine/javascript")).createJavaScriptRegexEngine();
      const hl = await createHighlighterCore({
        themes: [import("shiki/themes/github-dark.mjs")],
        langs: [],
        engine,
      });
      tracker.shikiInitMs = performance.now() - t0;
      return hl;
    })();
  }
  return highlighterPromise;
}

async function highlight(lang: string, code: string): Promise<string | null> {
  const loader = LANG_LOADERS[lang];
  if (!loader) return null;
  const hl = await getHighlighter();
  let langReady = loadedLangs.get(lang);
  if (!langReady) {
    langReady = loader().then((mod) => hl.loadLanguage((mod as { default: Parameters<HighlighterCore["loadLanguage"]>[0] }).default));
    loadedLangs.set(lang, langReady);
  }
  await langReady;
  const id = lang === "ts" ? "typescript" : lang;
  return hl.codeToHtml(code, { lang: id, theme: "github-dark" });
}

function CodeBlock({ lang, code }: CodeInfo) {
  const [html, setHtml] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    void highlight(lang, code).then((h) => {
      if (!alive) return;
      setHtml(h);
      if (h !== null) {
        tracker.highlighted++;
        tracker.bump();
      }
    });
    return () => {
      alive = false;
    };
  }, [lang, code]);
  if (html !== null) {
    // Shiki escapes the code; output is trusted in this spike.
    return <div className="code-block" dangerouslySetInnerHTML={{ __html: html }} />;
  }
  return (
    <pre className="code-block plain">
      <code>{code}</code>
    </pre>
  );
}

// ---------- mermaid (dynamic import, serialized renders) ----------

type MermaidApi = (typeof import("mermaid"))["default"];
let mermaidPromise: Promise<MermaidApi> | null = null;
let renderQueue: Promise<unknown> = Promise.resolve();
let diagramSeq = 0;

function getMermaid(): Promise<MermaidApi> {
  if (!mermaidPromise) {
    const t0 = performance.now();
    mermaidPromise = import("mermaid").then(({ default: m }) => {
      // Mermaid 12 defaults to the ELK layout (~1.5 MB); `?layout=dagre` measures the lighter one.
      const layout = new URLSearchParams(location.search).get("layout") ?? undefined;
      m.initialize({
        startOnLoad: false,
        theme: "dark",
        securityLevel: "strict",
        suppressErrorRendering: true,
        ...(layout ? { layout } : {}),
      });
      tracker.mermaidImportMs = performance.now() - t0;
      return m;
    });
  }
  return mermaidPromise;
}

function renderDiagram(code: string): Promise<string> {
  const job = renderQueue.then(async () => {
    const m = await getMermaid();
    const t0 = performance.now();
    try {
      const { svg } = await m.render(`mmd-${diagramSeq++}`, code);
      return svg;
    } finally {
      tracker.mermaidDurations.push(performance.now() - t0);
    }
  });
  renderQueue = job.catch(() => undefined);
  return job;
}

function MermaidBlock({ code }: { code: string }) {
  const [state, setState] = useState<{ svg: string } | { error: string } | null>(null);
  useEffect(() => {
    let alive = true;
    renderDiagram(code).then(
      (svg) => {
        if (!alive) return;
        setState({ svg });
        tracker.mermaidOk++;
        tracker.bump();
      },
      (err: unknown) => {
        if (!alive) return;
        setState({ error: err instanceof Error ? err.message : String(err) });
        tracker.mermaidErr++;
        tracker.bump();
      },
    );
    return () => {
      alive = false;
    };
  }, [code]);
  if (state && "svg" in state) {
    return <div className="mermaid-block" dangerouslySetInnerHTML={{ __html: state.svg }} />;
  }
  return (
    <div className="mermaid-block pending">
      {state && "error" in state ? <p className="mermaid-error">Mermaid: {state.error}</p> : null}
      <pre className="code-block plain">
        <code>{code}</code>
      </pre>
    </div>
  );
}

// ---------- hast -> React ----------

// Memory attribution switches: `?nomermaid` skips Mermaid, `?plain` skips Mermaid and Shiki.
const flags = new URLSearchParams(location.search);
export const SKIP_MERMAID = flags.has("nomermaid") || flags.has("plain");
export const SKIP_SHIKI = flags.has("plain");

const components: Partial<Components> = {
  pre: (props) => {
    const node = (props as { node?: Element }).node;
    const info = node ? codeInfo(node) : null;
    const plain = (code: string) => (
      <pre className="code-block plain">
        <code>{code}</code>
      </pre>
    );
    if (info?.lang === "mermaid") return SKIP_MERMAID ? plain(info.code) : <MermaidBlock code={info.code} />;
    if (info) return SKIP_SHIKI ? plain(info.code) : <CodeBlock lang={info.lang} code={info.code} />;
    return <pre>{props.children}</pre>;
  },
};

export function renderNodes(nodes: RootContent[]): ReactNode {
  return toJsxRuntime({ type: "root", children: nodes }, { Fragment, jsx, jsxs, components, passNode: true });
}
