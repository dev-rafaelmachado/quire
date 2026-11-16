import { cx } from "./cx";

export interface SegmentOption<V extends string> {
  value: V;
  label: string;
  /** Shown as disabled (with the group's reason nearby, EDT-08). */
  disabled?: boolean;
}

interface SegmentedProps<V extends string> {
  label: string;
  options: ReadonlyArray<SegmentOption<V>>;
  value: V;
  onChange: (value: V) => void;
}

/** Segmented control (wireframes: "Todas / Em andamento / Concluídas", "Ler / Dividir / Editar"). */
export function Segmented<V extends string>({ label, options, value, onChange }: SegmentedProps<V>) {
  return (
    <div
      role="group"
      aria-label={label}
      className="flex flex-none gap-0.5 rounded-[7px] border border-line bg-panel p-0.5"
    >
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          aria-pressed={o.value === value}
          disabled={o.disabled}
          onClick={() => onChange(o.value)}
          className={cx(
            "h-6 rounded-[5px] px-2.5 text-xs font-medium whitespace-nowrap disabled:cursor-not-allowed disabled:opacity-50",
            o.value === value ? "bg-raise text-text" : "text-muted hover:text-text disabled:hover:text-muted",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
