import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (cmd: string, args?: unknown) => invoke(cmd, args) }));

const { asCmdError, commands } = await import("./index");

describe("ipc commands", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue([]);
  });

  it("calls the Rust command names with camelCase arguments", async () => {
    await commands.listProjects({ text: "acme" });
    await commands.getSpec(4, null);
    await commands.readFile(7, "spec.md");
    expect(invoke.mock.calls).toEqual([
      ["list_projects", { filter: { text: "acme" } }],
      ["get_spec", { specId: 4, instanceId: null }],
      ["read_file", { instanceId: 7, name: "spec.md" }],
    ]);
  });

  it("narrows command errors", () => {
    expect(asCmdError({ code: "not_found", message: "x" })).toEqual({ code: "not_found", message: "x" });
    expect(asCmdError(new Error("boom"))).toEqual({ code: "unknown", message: "Error: boom" });
  });
});
