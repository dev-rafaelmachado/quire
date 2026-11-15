import { beforeEach, describe, expect, it, vi } from "vitest";

type Handler = (e: { payload: unknown }) => void;
const handlers: Record<string, Handler> = {};
const calls: string[] = [];
let lists: unknown[][] = [];

vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => ({ code: "x", message: String(e) }),
  commands: {
    getConfig: async () => {
      calls.push("getConfig");
      return { path: null, error: null, scanPaths: ["/ws"], pollIntervalMs: 3000, projects: [], presets: [] };
    },
    listProjects: async () => {
      calls.push("listProjects");
      return lists.shift() ?? [];
    },
    startScan: async () => {
      calls.push("startScan");
      return null;
    },
  },
  events: {
    scanProgress: { listen: async (h: Handler) => void (handlers["scan"] = h) },
    specsChanged: { listen: async (h: Handler) => void (handlers["changed"] = h) },
  },
}));

const { useWorkspace } = await import("./store");

describe("workspace store", () => {
  beforeEach(() => {
    calls.length = 0;
  });

  it("lists from the cache first, then scans, and reloads as the scan reports (DSC-04)", async () => {
    vi.useFakeTimers();
    lists = [[{ id: 1, name: "cached" }], [{ id: 1 }, { id: 2 }]];
    await useWorkspace.getState().init();
    expect(calls).toEqual(["getConfig", "listProjects", "startScan"]);
    expect(useWorkspace.getState().projects).toHaveLength(1);

    handlers["scan"]?.({ payload: { phase: "indexing", found: 2, processed: 1, total: 2, errors: [], message: null } });
    expect(useWorkspace.getState().scan?.phase).toBe("indexing");
    handlers["changed"]?.({ payload: null });
    handlers["changed"]?.({ payload: null }); // throttled into one reload
    await vi.advanceTimersByTimeAsync(200);
    expect(calls.filter((c) => c === "listProjects")).toHaveLength(2);
    expect(useWorkspace.getState().projects).toHaveLength(2);
    vi.useRealTimers();
  });
});
