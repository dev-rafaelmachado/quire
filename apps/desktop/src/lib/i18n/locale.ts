import { isLocale, type Locale, setLocale } from "./index";

/** The language to use (TD-23): the one in `config.toml` when supported; else the first system
 *  preference that is Portuguese or English; else English. An unsupported configured value
 *  counts as absent (DEB-10). */
export function resolveLocale(configured: string | null | undefined, system: readonly string[]): Locale {
  if (isLocale(configured)) return configured;
  for (const tag of system) {
    const lang = tag.toLowerCase().split(/[-_]/)[0];
    if (lang === "pt") return "pt-BR";
    if (lang === "en") return "en";
  }
  return "en";
}

/** The system preferences as the webview exposes them (WebKitGTK: `LANGUAGE`/`LC_*`/`LANG`;
 *  WebView2: the Windows display language). */
export function systemLanguages(): readonly string[] {
  if (typeof navigator === "undefined") return [];
  if (navigator.languages.length > 0) return navigator.languages;
  return navigator.language ? [navigator.language] : [];
}

/** Applies the configured language (or the system's) to the running UI. */
export function applyConfiguredLanguage(configured: string | null | undefined): void {
  setLocale(resolveLocale(configured, systemLanguages()));
}
