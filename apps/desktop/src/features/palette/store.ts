import { create } from "zustand";

const RECENT_MAX = 8;
// `?palette=<text>` opens the palette with that text (screenshots and demos).
const URL_PALETTE = typeof window === "undefined" ? null : new URLSearchParams(window.location.search).get("palette");

interface PaletteState {
  open: boolean;
  query: string;
  /** Ids of items run recently, newest first, for this session only (CMD-04). */
  recent: string[];
  setOpen: (open: boolean) => void;
  setQuery: (query: string) => void;
  remember: (id: string) => void;
}

export const usePalette = create<PaletteState>((set, get) => ({
  open: URL_PALETTE !== null,
  query: URL_PALETTE ?? "",
  recent: [],
  setOpen: (open) => set({ open, query: open ? get().query : "" }),
  setQuery: (query) => set({ query }),
  remember: (id) => set({ recent: [id, ...get().recent.filter((r) => r !== id)].slice(0, RECENT_MAX) }),
}));

/** Text after which the palette searches the content of the specs (CMD-03). */
export const SEARCH_PREFIX = "/";
