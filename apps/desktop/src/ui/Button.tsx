import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cx } from "./cx";

type Variant = "outline" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
}

/** Text button (wireframes: 30 px, bordered panel or ghost). */
export function Button({ variant = "outline", icon, className, children, type = "button", ...rest }: ButtonProps) {
  return (
    <button
      type={type}
      className={cx(
        "inline-flex h-[30px] items-center gap-2 rounded-[var(--radius-md)] px-2.5 text-[12.5px] font-medium",
        variant === "outline" ? "border border-line bg-panel text-text hover:bg-raise" : "text-muted hover:bg-raise",
        "disabled:opacity-60",
        className,
      )}
      {...rest}
    >
      {icon}
      {children}
    </button>
  );
}

interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-label" | "children"> {
  /** Required: icon-only buttons are named for assistive technology. */
  label: string;
  icon: ReactNode;
}

/** 28 px icon-only button; `label` becomes its `aria-label` and tooltip. */
export function IconButton({ label, icon, className, type = "button", ...rest }: IconButtonProps) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cx(
        "inline-flex size-7 flex-none items-center justify-center rounded-[var(--radius-md)] text-muted hover:bg-raise hover:text-text",
        className,
      )}
      {...rest}
    >
      {icon}
    </button>
  );
}
