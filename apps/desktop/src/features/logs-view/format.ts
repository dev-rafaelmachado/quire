import { t } from "../../lib/i18n";

/** `2026-11-04T17:42:00Z` -> `04 nov · 17:42` in local time (wireframe 3), month names from
 *  the catalog of the language in use. */
export function formatWhen(ts: string | null): string {
  if (!ts) return "—";
  const d = new Date(ts);
  if (Number.isNaN(d.getTime())) return ts;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getDate())} ${t("format.months").split(",")[d.getMonth()] ?? ""} · ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** Seconds -> `42s`, `4m 12s`, `1h 03m`. */
export function formatDuration(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds) || seconds < 0) return "—";
  const s = Math.round(seconds);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ${String(s % 60).padStart(2, "0")}s`;
  return `${Math.floor(m / 60)}h ${String(m % 60).padStart(2, "0")}m`;
}

/** Long working directories keep their tail: `…/.worktrees/ACME-142/gateway`. */
export function shortPath(path: string, keep = 3): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length <= keep ? path : `…/${parts.slice(-keep).join("/")}`;
}
