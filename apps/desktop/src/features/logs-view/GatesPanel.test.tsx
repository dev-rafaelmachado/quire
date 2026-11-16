// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { GateRun, GatesView } from "../../lib/ipc";
import { GatesPanel } from "./GatesPanel";
import { LessonCards, LessonsPanel } from "./Lessons";
import { useLogs } from "./store";

const run = (over: Partial<GateRun>): GateRun => ({
  ts: "2026-01-04T10:00:00Z",
  repo: "acme_billing",
  level: "fase",
  cmd: "cargo test",
  cwd: "/home/dev/acme/.worktrees/bill-31/acme_billing",
  head: "9f8e7d6c5b4a",
  exit: 0,
  status: "pass",
  duration_s: 12.3,
  tail: [],
  ...over,
});

const gates: GatesView = {
  runs: [
    run({ ts: "2026-01-04T15:00:00Z", level: "final", cmd: "cargo test --release", duration_s: 48 }),
    run({ ts: "2026-01-04T12:00:00Z", status: "fail", exit: 101, head: "7d6c5b4a3f2e", tail: ["test export_large ... FAILED", "thread panicked at src/invoice.rs:3"] }),
    run({}),
  ],
  counts: { runs: 3, passes: 2, fails: 1 },
  skipped: 2,
  skippedLines: [2, 5],
};

beforeEach(() => useLogs.setState({ gates, level: "all", status: "all", open: new Set([1]) }));
afterEach(cleanup);

describe("gates view", () => {
  it("shows counts, rows newest first and the skipped-lines notice (LOG-01, LOG-04)", () => {
    render(<GatesPanel gates={gates} />);
    expect(screen.getByText("3 execuções · 2 pass · 1 fail")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("2 linhas ignoradas em gates.jsonl (linhas 2, 5).");
    const commands = screen.getAllByRole("row").slice(1).map((r) => r.querySelector("td:last-child")?.textContent);
    expect(commands[0]).toBe("cargo test --release");
  });

  it("shows tail, exit code and directory of a failed run (LOG-02)", () => {
    render(<GatesPanel gates={gates} />);
    expect(screen.getByText(/test export_large \.\.\. FAILED/)).toBeTruthy();
    expect(screen.getByText("exit 101 · cwd …/.worktrees/bill-31/acme_billing")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /Esconder saída/ }));
    expect(screen.queryByText(/FAILED/)).toBeNull();
  });

  it("combines level and status filters", () => {
    render(<GatesPanel gates={gates} />);
    fireEvent.click(screen.getByRole("button", { name: "final" }));
    expect(screen.getAllByRole("row")).toHaveLength(2);
    fireEvent.click(screen.getAllByRole("button", { name: "fail" })[0] as HTMLElement);
    expect(screen.getByText("Nenhuma execução com esses filtros.")).toBeTruthy();
  });
});

describe("lessons", () => {
  it("lists signal, scope, text and source (LOG-03, LOG-04)", () => {
    const lessons = [{ ts: null, feature: "f", signal: "gate_fail", source: "src/a.rs:12", text: "TTL acima do prazo.", scope: "integration/cache" }];
    render(<LessonCards lessons={lessons} />);
    expect(screen.getByLabelText("Sinal gate_fail")).toBeTruthy();
    expect(screen.getByText("integration/cache")).toBeTruthy();
    expect(screen.getByText("TTL acima do prazo.")).toBeTruthy();
    expect(screen.getByText("src/a.rs:12")).toBeTruthy();
    cleanup();
    render(<LessonsPanel lessons={{ lessons, skipped: 1, skippedLines: [3] }} />);
    expect(screen.getByRole("status").textContent).toBe("1 linha ignorada em lessons.jsonl (linha 3).");
  });
});
