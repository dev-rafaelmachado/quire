import { create } from "zustand";
import { en } from "./en";
import { ptBR } from "./pt-BR";

export type Plural = { readonly one: string; readonly other: string };
type Leaf = string | Plural;

/** Dotted paths to every string of the catalog, e.g. `"sidebar.title"`. */
type Paths<T, P extends string = ""> = {
  [K in keyof T & string]: T[K] extends Leaf ? `${P}${K}` : Paths<T[K], `${P}${K}.`>;
}[keyof T & string];

/** The shape of `pt-BR` with plain strings: every other catalog must match it exactly (TD-19). */
export type Catalog<T = typeof ptBR> = {
  readonly [K in keyof T]: T[K] extends string ? string : T[K] extends Plural ? Plural : Catalog<T[K]>;
};

export type MessageKey = Paths<typeof ptBR>;
export type Params = Readonly<Record<string, string | number>>;

export const catalogs: Readonly<Record<"pt-BR" | "en", Catalog>> = { "pt-BR": ptBR, en };
export type Locale = keyof typeof catalogs;
export const LOCALES: readonly Locale[] = ["pt-BR", "en"];

export function isLocale(value: unknown): value is Locale {
  return value === "pt-BR" || value === "en";
}

interface LocaleState {
  locale: Locale;
}

/** The language in use. Components that call `t()` re-render when it changes (the app root
 *  subscribes), so switching needs no restart. */
export const useLocale = create<LocaleState>(() => ({ locale: "pt-BR" }));

let plural = new Intl.PluralRules("pt-BR");

export function setLocale(next: Locale): void {
  if (useLocale.getState().locale === next) return;
  plural = new Intl.PluralRules(next);
  if (typeof document !== "undefined") document.documentElement.lang = next;
  useLocale.setState({ locale: next });
}

export function getLocale(): Locale {
  return useLocale.getState().locale;
}

function lookup(catalog: Catalog, key: string): Leaf | undefined {
  let node: unknown = catalog;
  for (const part of key.split(".")) {
    if (typeof node !== "object" || node === null || !(part in node)) return undefined;
    node = (node as Record<string, unknown>)[part];
  }
  if (typeof node === "string") return node;
  if (typeof node === "object" && node !== null && "one" in node && "other" in node) return node as Plural;
  return undefined;
}

/** Translates `key`, interpolating `{name}` from `params`; `n` picks the plural form. */
export function t(key: MessageKey, params: Params = {}): string {
  const entry = lookup(catalogs[getLocale()], key) ?? lookup(ptBR, key);
  if (entry === undefined) return key;
  let text: string;
  if (typeof entry === "string") {
    text = entry;
  } else {
    const n = typeof params["n"] === "number" ? params["n"] : 0;
    text = plural.select(n) === "one" ? entry.one : entry.other;
  }
  return text.replace(/\{(\w+)\}/g, (match, name: string) => {
    const value = params[name];
    return value === undefined ? match : String(value);
  });
}

/** A number in the language in use (`1,5` in pt-BR, `1.5` in English). */
export function formatNumber(value: number, fractionDigits = 0): string {
  return new Intl.NumberFormat(getLocale(), {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  }).format(value);
}
