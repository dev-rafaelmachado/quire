import { useEffect, useRef } from "react";
import { t } from "../../lib/i18n";
import { Markdown } from "../../lib/markdown/Markdown";
import { Outline } from "../../lib/markdown/Outline";
import { useSpecView } from "./store";

/** Center column: the rendered file of the open spec. */
export function SpecView() {
  const doc = useSpecView((s) => s.doc);
  const content = useSpecView((s) => s.content);
  const collapsed = useSpecView((s) => s.collapsed);
  const toggle = useSpecView((s) => s.toggle);
  const active = useSpecView((s) => s.active);
  const setActive = useSpecView((s) => s.setActive);
  const scrollRequest = useSpecView((s) => s.scrollRequest);
  const error = useSpecView((s) => s.error);
  const scroller = useRef<HTMLDivElement>(null);

  // Active section: the last heading whose top has passed the top of the scroll area (VIW-05).
  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    let frame = 0;
    const update = () => {
      frame = 0;
      const top = el.getBoundingClientRect().top + 24;
      let current: string | null = null;
      for (const h of el.querySelectorAll<HTMLElement>(".md-heading[id]")) {
        if (h.getBoundingClientRect().top <= top) current = h.id;
        else break;
      }
      if (current !== useSpecView.getState().active) setActive(current);
    };
    const onScroll = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    update();
    return () => {
      el.removeEventListener("scroll", onScroll);
      if (frame) cancelAnimationFrame(frame);
    };
  }, [doc, collapsed, setActive]);

  useEffect(() => {
    if (!scrollRequest || !active) return;
    document.getElementById(active)?.scrollIntoView({ block: "start" });
  }, [scrollRequest, active]);

  return (
    <div ref={scroller} className="min-h-0 flex-1 overflow-y-auto">
      <div className="px-10 pt-9 pb-24">
        {error ? (
          <p role="alert" className="m-0 text-warn">
            {error}
          </p>
        ) : null}
        {doc ? <Markdown doc={doc} collapsed={collapsed} onToggle={toggle} plain={content?.large ?? false} /> : null}
        {!doc && content?.text ? (
          <pre className="m-0 font-code text-[12.5px] whitespace-pre-wrap">{content.text}</pre>
        ) : null}
        {!doc && content && content.text === null ? <p className="m-0 text-muted">{t("main.placeholder")}</p> : null}
      </div>
    </div>
  );
}

/** Right column: outline of the file. */
export function SpecOutline() {
  const entries = useSpecView((s) => s.outline);
  const active = useSpecView((s) => s.active);
  const navigate = useSpecView((s) => s.navigate);
  if (entries.length === 0) return null;
  return <Outline entries={entries} active={active} onNavigate={navigate} />;
}
