import { devParam } from "../lib/devParams";
import { create } from "zustand";

export type Theme = "dark" | "light";

const CLASS: Record<Theme, string> = { dark: "qd", light: "ql" };

const URL_THEME = devParam("theme");

function initialTheme(): Theme {
  // Dark is the default; `?theme=light|dark` (screenshots and demos) wins over config.toml.
  return URL_THEME === "light" ? "light" : "dark";
}

/** Applies the theme saved in config.toml unless the URL forces one. */
export function applyConfiguredTheme(theme: string | null): void {
  if (URL_THEME === "light" || URL_THEME === "dark") return;
  useTheme.getState().setTheme(theme === "light" ? "light" : "dark");
}

function apply(theme: Theme): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.classList.remove(CLASS.dark, CLASS.light);
  root.classList.add(CLASS[theme]);
  root.style.colorScheme = theme;
}

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  toggle: () => void;
}

export const useTheme = create<ThemeState>((set, get) => ({
  theme: initialTheme(),
  setTheme: (theme) => {
    apply(theme);
    set({ theme });
  },
  toggle: () => get().setTheme(get().theme === "dark" ? "light" : "dark"),
}));

apply(useTheme.getState().theme);
