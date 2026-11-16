// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/ipc", () => ({ asCmdError: (e: unknown) => e, commands: {} }));
const { EditBar } = await import("./EditBar");
const { useEditor } = await import("./store");

const buffer = { instanceId: 1, name: "spec.md", savedText: "", base: "1-a", format: { bom: false, eol: "lf" as const, mixed: false, finalNewline: true }, readOnly: false, locked: null };

beforeEach(() => useEditor.setState({ mode: "read", buffer, dirty: false, saving: false, error: null, confirmingMixed: false }));
afterEach(cleanup);

describe("edit bar (EDT-01, EDT-08)", () => {
  it("offers Ler, Editar and Dividir with Ler selected", () => {
    render(<EditBar />);
    const group = screen.getByRole("group", { name: "Modo do arquivo" });
    expect(within(group).getByRole("button", { name: "Ler" }).getAttribute("aria-pressed")).toBe("true");
    expect((within(group).getByRole("button", { name: "Editar" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("disables Editar and Dividir with the reason when locked", () => {
    useEditor.setState({ buffer: { ...buffer, locked: { code: "write_unreliable_fs", message: "9p" } } });
    render(<EditBar />);
    const group = screen.getByRole("group", { name: "Modo do arquivo" });
    expect((within(group).getByRole("button", { name: "Editar" }) as HTMLButtonElement).disabled).toBe(true);
    expect((within(group).getByRole("button", { name: "Dividir" }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(/Edição indisponível: escrita desativada neste sistema de arquivos/)).toBeTruthy();
  });

  it("shows the unsaved state and an enabled Save only while dirty", () => {
    useEditor.setState({ mode: "edit", dirty: false });
    const { rerender } = render(<EditBar />);
    expect((screen.getByRole("button", { name: "Salvar" }) as HTMLButtonElement).disabled).toBe(true);
    useEditor.setState({ dirty: true });
    rerender(<EditBar />);
    expect(screen.getByText("Alterações não salvas")).toBeTruthy();
    expect((screen.getByRole("button", { name: "Salvar" }) as HTMLButtonElement).disabled).toBe(false);
  });
});
