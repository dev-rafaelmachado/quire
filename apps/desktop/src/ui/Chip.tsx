import type { ReactNode } from "react";
import { cx } from "./cx";

export type Tone = "neutral" | "acc" | "ok" | "warn";

const TONES: Record<Tone, string> = {
  neutral: "bg-raise text-muted",
  acc: "bg-acc-bg text-acc",
  ok: "bg-ok-bg text-ok",
  warn: "bg-warn-bg text-warn",
};

interface ChipProps {
  tone?: Tone;
  mono?: boolean;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
}

/** 20 px pill used for status, ids and the "N versões" badge. */
export function Chip({ tone = "neutral", mono = false, icon, children, className }: ChipProps) {
  return (
    <span
      className={cx(
        "inline-flex h-5 flex-none items-center gap-1 rounded-[var(--radius-xl)] px-2 text-[11px] font-medium whitespace-nowrap",
        mono && "font-code",
        TONES[tone],
        className,
      )}
    >
      {icon}
      {children}
    </span>
  );
}
