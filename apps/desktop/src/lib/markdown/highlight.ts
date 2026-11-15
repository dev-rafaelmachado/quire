// Syntax highlighting with Shiki, loaded on demand (TD-04). The Oniguruma (WASM) engine is
// used instead of the JS regex engine: the T2 spike measured ~190 MB less memory and ~35%
// faster highlighting. Both themes are emitted as CSS variables and picked by .qd / .ql.
import type { HighlighterCore, LanguageInput } from "shiki/core";

type LangModule = { default: LanguageInput };

/** Fence alias -> [Shiki language id, grammar loader]. */
const LANGS: Record<string, readonly [string, () => Promise<LangModule>]> = {
  ts: ["typescript", () => import("shiki/langs/typescript.mjs")],
  typescript: ["typescript", () => import("shiki/langs/typescript.mjs")],
  tsx: ["tsx", () => import("shiki/langs/tsx.mjs")],
  js: ["javascript", () => import("shiki/langs/javascript.mjs")],
  javascript: ["javascript", () => import("shiki/langs/javascript.mjs")],
  jsx: ["jsx", () => import("shiki/langs/jsx.mjs")],
  json: ["json", () => import("shiki/langs/json.mjs")],
  jsonc: ["jsonc", () => import("shiki/langs/jsonc.mjs")],
  jsonl: ["jsonl", () => import("shiki/langs/jsonl.mjs")],
  rust: ["rust", () => import("shiki/langs/rust.mjs")],
  rs: ["rust", () => import("shiki/langs/rust.mjs")],
  bash: ["bash", () => import("shiki/langs/bash.mjs")],
  sh: ["bash", () => import("shiki/langs/bash.mjs")],
  shell: ["bash", () => import("shiki/langs/bash.mjs")],
  sql: ["sql", () => import("shiki/langs/sql.mjs")],
  toml: ["toml", () => import("shiki/langs/toml.mjs")],
  yaml: ["yaml", () => import("shiki/langs/yaml.mjs")],
  yml: ["yaml", () => import("shiki/langs/yaml.mjs")],
  python: ["python", () => import("shiki/langs/python.mjs")],
  py: ["python", () => import("shiki/langs/python.mjs")],
  go: ["go", () => import("shiki/langs/go.mjs")],
  java: ["java", () => import("shiki/langs/java.mjs")],
  kotlin: ["kotlin", () => import("shiki/langs/kotlin.mjs")],
  csharp: ["csharp", () => import("shiki/langs/csharp.mjs")],
  diff: ["diff", () => import("shiki/langs/diff.mjs")],
  html: ["html", () => import("shiki/langs/html.mjs")],
  css: ["css", () => import("shiki/langs/css.mjs")],
  markdown: ["markdown", () => import("shiki/langs/markdown.mjs")],
  md: ["markdown", () => import("shiki/langs/markdown.mjs")],
};

let highlighter: Promise<HighlighterCore> | null = null;
const loaded = new Map<string, Promise<string | null>>();

function getHighlighter(): Promise<HighlighterCore> {
  highlighter ??= (async () => {
    const [{ createHighlighterCore }, { createOnigurumaEngine }] = await Promise.all([
      import("shiki/core"),
      import("shiki/engine/oniguruma"),
    ]);
    return createHighlighterCore({
      themes: [import("shiki/themes/github-dark.mjs"), import("shiki/themes/github-light.mjs")],
      langs: [],
      engine: createOnigurumaEngine(import("shiki/wasm")),
    });
  })();
  return highlighter;
}

export function canHighlight(lang: string): boolean {
  return lang in LANGS;
}

/** Loads a grammar once; resolves to its Shiki id, or null if unknown. */
function loadLang(hl: HighlighterCore, lang: string): Promise<string | null> {
  const entry = LANGS[lang];
  if (!entry) return Promise.resolve(null);
  const [id, loader] = entry;
  let p = loaded.get(id);
  if (!p) {
    p = loader().then(async (mod) => {
      await hl.loadLanguage(mod.default);
      return id;
    });
    loaded.set(id, p);
  }
  return p;
}

/** HTML for a code block, or null when the language is unknown. Output is escaped by Shiki. */
export async function highlight(code: string, lang: string): Promise<string | null> {
  if (!canHighlight(lang)) return null;
  const hl = await getHighlighter();
  const id = await loadLang(hl, lang);
  if (!id) return null;
  return hl.codeToHtml(code, {
    lang: id,
    themes: { dark: "github-dark", light: "github-light" },
    defaultColor: false,
  });
}
