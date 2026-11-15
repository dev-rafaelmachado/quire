import { cx } from "./cx";

export type BarTone = "ok" | "acc" | "faint";

const FILL: Record<BarTone, string> = { ok: "bg-ok", acc: "bg-acc", faint: "bg-faint" };

interface ProgressBarProps {
  done: number;
  total: number;
  tone: BarTone;
  label: string;
  width?: number;
  className?: string;
}

/** 4 px progress bar; the fill tone follows the state (complete, in progress, pending). */
export function ProgressBar({ done, total, tone, label, width = 56, className }: ProgressBarProps) {
  const pct = total > 0 ? Math.min(100, Math.round((done / total) * 100)) : 0;
  return (
    <span
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={total}
      aria-valuenow={done}
      className={cx("block h-1 flex-none rounded-sm bg-line", className)}
      style={{ width }}
    >
      <span className={cx("block h-1 rounded-sm", FILL[tone])} style={{ width: `${pct}%` }} />
    </span>
  );
}
