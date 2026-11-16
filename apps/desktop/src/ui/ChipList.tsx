import { useLayoutEffect, useRef, useState } from "react";
import { Chip, type Tone } from "./Chip";

const GAP = 4;
const MORE = 30; // room kept for the "+N" chip

/** How many chips of these widths fit in `available`, keeping room for "+N" when some are left
 *  out. At least one is hidden behind "+N" before none fits at all. */
export function chipsThatFit(widths: readonly number[], available: number): number {
  let used = 0;
  for (let i = 0; i < widths.length; i++) {
    const w = (widths[i] ?? 0) + (i > 0 ? GAP : 0);
    const reserve = i < widths.length - 1 ? MORE + GAP : 0;
    if (used + w + reserve > available) return i;
    used += w;
  }
  return widths.length;
}

interface ChipListProps {
  items: readonly string[];
  tone?: Tone;
  mono?: boolean;
  /** Read in full by screen readers and shown on hover (DEB-04). */
  label: string;
}

/** A row of chips that gives way in narrow containers: shows the chips that fit and a "+N"
 *  chip for the rest, never painting past its box (DEB-04). */
export function ChipList({ items, tone = "acc", mono = true, label }: ChipListProps) {
  const box = useRef<HTMLSpanElement>(null);
  const ruler = useRef<HTMLSpanElement>(null);
  const [fit, setFit] = useState(items.length);
  const [natural, setNatural] = useState<number | undefined>(undefined);

  useLayoutEffect(() => {
    const el = box.current;
    const measure = () => {
      const kids = ruler.current ? [...ruler.current.children].map((c) => (c as HTMLElement).offsetWidth) : [];
      const full = kids.reduce((a, w, i) => a + w + (i > 0 ? GAP : 0), 0);
      setNatural(full > 0 ? full : undefined);
      setFit(el ? chipsThatFit(kids, el.clientWidth || full) : items.length);
    };
    measure();
    if (!el || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [items]);

  if (items.length === 0) return null;
  const hidden = items.length - fit;
  return (
    <span
      ref={box}
      title={label}
      className="relative flex min-w-[30px] shrink gap-1 overflow-hidden"
      style={{ flexBasis: natural, maxWidth: natural }}
    >
      <span className="sr-only">{label}</span>
      <span ref={ruler} aria-hidden="true" className="invisible absolute top-0 left-0 flex gap-1 whitespace-nowrap">
        {items.map((r) => (
          <Chip key={r} tone={tone} mono={mono}>
            {r}
          </Chip>
        ))}
      </span>
      <span aria-hidden="true" className="flex gap-1">
        {items.slice(0, fit).map((r) => (
          <Chip key={r} tone={tone} mono={mono}>
            {r}
          </Chip>
        ))}
        {hidden > 0 ? (
          <Chip tone="neutral" mono>
            +{hidden}
          </Chip>
        ) : null}
      </span>
    </span>
  );
}
