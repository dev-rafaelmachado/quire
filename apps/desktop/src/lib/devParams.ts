// Start parameters for screenshots, demos and measurements (`?open=`, `?mode=`, `?search=`…).
// Development builds only: in production `import.meta.env.DEV` is the constant `false`, so this
// always returns `null` and the bundler drops what depends on it.
export function devParam(name: string): string | null {
  if (!import.meta.env.DEV || typeof window === "undefined") return null;
  return new URLSearchParams(window.location.search).get(name);
}
