import { create } from "zustand";

export type Theme = "dark" | "light";

const CLASS: Record<Theme, string> = { dark: "qd", light: "ql" };

function initialTheme(): Theme {
  // Dark is the default; `?theme=light` exists for screenshots until preferences (T23).
  return new URLSearchParams(window.location.search).get("theme") === "light" ? "light" : "dark";
}

function apply(theme: Theme): void {
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
