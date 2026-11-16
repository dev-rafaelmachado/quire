import { useEffect, useState } from "react";
import { useTheme } from "../../app/theme";
import { t } from "../i18n";
import { cachedHighlight, highlight } from "./highlight";
import { cachedMermaid, renderMermaid } from "./mermaid";

/** Fenced code: monospace, alignment preserved (VIW-04), highlighted when possible (VIW-02). */
export function CodeBlock({ code, lang, plain }: { code: string; lang: string; plain: boolean }) {
  const [html, setHtml] = useState<string | null>(() => (plain ? null : (cachedHighlight(code, lang) ?? null)));
  useEffect(() => {
    if (plain) return;
    const hit = cachedHighlight(code, lang);
    if (hit !== undefined) {
      setHtml(hit);
      return;
    }
    let alive = true;
    void highlight(code, lang).then((h) => {
      if (alive) setHtml(h);
    });
    return () => {
      alive = false;
    };
  }, [code, lang, plain]);
  if (html !== null) {
    // Shiki escapes the code it renders.
    return <div className="md-code" data-lang={lang || undefined} dangerouslySetInnerHTML={{ __html: html }} />;
  }
  return (
    <div className="md-code" data-lang={lang || undefined}>
      <pre>
        <code>{code}</code>
      </pre>
    </div>
  );
}

/** Mermaid diagram; invalid syntax shows the code and the error, page intact (VIW-03). */
export function MermaidBlock({ code }: { code: string }) {
  const dark = useTheme((s) => s.theme === "dark");
  const [state, setState] = useState<{ svg: string } | { error: string } | null>(() => cachedMermaid(code, dark) ?? null);
  useEffect(() => {
    const hit = cachedMermaid(code, dark);
    if (hit) {
      setState(hit);
      return;
    }
    let alive = true;
    setState(null);
    renderMermaid(code, dark).then(
      (svg) => alive && setState({ svg }),
      (e: unknown) => alive && setState({ error: e instanceof Error ? e.message : String(e) }),
    );
    return () => {
      alive = false;
    };
  }, [code, dark]);
  if (state && "svg" in state) {
    // securityLevel "strict": Mermaid sanitizes labels and disables scripts in the SVG.
    return <div className="md-mermaid" dangerouslySetInnerHTML={{ __html: state.svg }} />;
  }
  return (
    <div className="md-mermaid md-mermaid-pending" aria-busy={state === null}>
      {state && "error" in state ? (
        <p role="alert" className="m-0 mb-2 text-xs text-warn">
          {t("markdown.mermaidError", { message: state.error })}
        </p>
      ) : (
        <p className="m-0 mb-2 text-xs text-faint">{t("markdown.rendering")}</p>
      )}
      <div className="md-code">
        <pre>
          <code>{code}</code>
        </pre>
      </div>
    </div>
  );
}
