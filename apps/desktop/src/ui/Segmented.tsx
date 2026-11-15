import { cx } from "./cx";

export interface SegmentOption<V extends string> {
  value: V;
  label: string;
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
          onClick={() => onChange(o.value)}
          className={cx(
            "h-6 rounded-[5px] px-2.5 text-xs font-medium whitespace-nowrap",
            o.value === value ? "bg-raise text-text" : "text-muted hover:text-text",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}
