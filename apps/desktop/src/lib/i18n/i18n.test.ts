import { describe, expect, it } from "vitest";
import { t } from "./index";

describe("t()", () => {
  it("returns catalog strings", () => {
    expect(t("sidebar.title")).toBe("Projetos");
  });

  it("interpolates and picks plural forms", () => {
    expect(t("sidebar.versions", { n: 1 })).toBe("1 versão");
    expect(t("sidebar.versions", { n: 3 })).toBe("3 versões");
    expect(t("scan.scanning", { path: "~/projects" })).toBe("Varrendo ~/projects");
    expect(t("progress.label", { done: 2, total: 5 })).toBe("Progresso: 2 de 5 tarefas");
  });

  it("leaves unknown placeholders visible", () => {
    expect(t("scan.scanning")).toBe("Varrendo {path}");
  });
});
