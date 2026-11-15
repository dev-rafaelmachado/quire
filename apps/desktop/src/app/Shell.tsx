import type { ReactNode } from "react";

interface ShellProps {
  top: ReactNode;
  side: ReactNode;
  main: ReactNode;
  /** Contextual right column (outline, task detail, lessons); omitted when empty. */
  right?: ReactNode;
  status: ReactNode;
}

/** Window layout from docs/design/README.md: 48 px top bar, 272 px sidebar, center,
 *  248 px contextual column and 28 px status bar. */
export function Shell({ top, side, main, right, status }: ShellProps) {
  return (
    <div className="flex h-screen flex-col overflow-hidden bg-bg text-[13px] text-text">
      <header className="flex h-12 flex-none items-center gap-3 border-b border-line bg-side pr-3 pl-4">{top}</header>
      <div className="flex min-h-0 flex-1">
        <nav className="flex w-[272px] flex-none flex-col overflow-hidden border-r border-line bg-side">{side}</nav>
        <main className="flex min-w-0 flex-1 flex-col">{main}</main>
        {right ? (
          <aside className="flex w-[248px] flex-none flex-col overflow-hidden border-l border-line bg-side">{right}</aside>
        ) : null}
      </div>
      <footer className="flex h-7 flex-none items-center gap-3 border-t border-line bg-side px-4 text-xs text-muted">
        {status}
      </footer>
    </div>
  );
}
