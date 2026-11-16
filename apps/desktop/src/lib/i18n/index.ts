import { ptBR } from "./pt-BR";

type Plural = { readonly one: string; readonly other: string };
type Leaf = string | Plural;

/** Dotted paths to every string of the catalog, e.g. `"sidebar.title"`. */
type Paths<T, P extends string = ""> = {
  [K in keyof T & string]: T[K] extends Leaf ? `${P}${K}` : Paths<T[K], `${P}${K}.`>;
}[keyof T & string];

export type MessageKey = Paths<typeof ptBR>;
export type Params = Readonly<Record<string, string | number>>;

const catalogs = { "pt-BR": ptBR } as const;
export type Locale = keyof typeof catalogs;

let locale: Locale = "pt-BR";
const plural = new Intl.PluralRules("pt-BR");

export function setLocale(next: Locale): void {
  locale = next;
}

function lookup(key: string): Leaf | undefined {
  let node: unknown = catalogs[locale];
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
  const entry = lookup(key);
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
