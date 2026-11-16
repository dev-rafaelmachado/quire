// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const removeOrphanTemp = vi.fn(async () => null);
vi.mock("../../lib/ipc", () => ({
  asCmdError: (e: unknown) => e,
  commands: {
    orphanTemps: async () => [".spec.md.quire-77-3.tmp"],
    removeOrphanTemp: (...a: unknown[]) => removeOrphanTemp(...(a as [])),
  },
}));
const { OrphanNotice } = await import("./OrphanNotice");
afterEach(cleanup);

describe("orphan temporary files (SAV-10)", () => {
  it("are listed with the git warning and removed only on a click", async () => {
    render(<OrphanNotice instanceId={4} version={1} />);
    expect(await screen.findByText(".spec.md.quire-77-3.tmp")).toBeTruthy();
    expect(screen.getByText(/git add -A/)).toBeTruthy();
    expect(removeOrphanTemp).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Remover .spec.md.quire-77-3.tmp" }));
    await waitFor(() => expect(removeOrphanTemp).toHaveBeenCalledWith(4, ".spec.md.quire-77-3.tmp"));
    await waitFor(() => expect(screen.queryByText(".spec.md.quire-77-3.tmp")).toBeNull());
  });
});
