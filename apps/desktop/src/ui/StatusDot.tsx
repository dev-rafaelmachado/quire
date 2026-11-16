import { cx } from "./cx";

export type DotTone = "ok" | "acc" | "warn" | "faint";

const DOT: Record<DotTone, string> = { ok: "bg-ok", acc: "bg-acc", warn: "bg-warn", faint: "bg-faint" };

/** 7 px status dot; always paired with text (color is never the only signal). */
export function StatusDot({ tone, pulse = false }: { tone: DotTone; pulse?: boolean }) {
  return <span aria-hidden="true" className={cx("size-[7px] flex-none rounded-full", DOT[tone], pulse && "animate-pulse")} />;
}
