import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Tauri expects a fixed dev port and must see Rust errors in the terminal.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  test: {
    setupFiles: ["src/test-setup.ts"],
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
