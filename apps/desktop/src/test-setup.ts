// Tests read the pt-BR catalog: pin the "system" language so the result does not depend on the
// machine running them (LANG on Linux, display language on the Windows runner).
if (typeof navigator !== "undefined") {
  Object.defineProperty(navigator, "languages", { value: ["pt-BR"], configurable: true });
  Object.defineProperty(navigator, "language", { value: "pt-BR", configurable: true });
}
