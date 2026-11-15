// T2 spike (disposable): separate Vite project rooted at src/spike, never part of the app build.
// A small middleware receives the metrics POSTed by the page and appends them to $SPIKE_REPORT.
import { appendFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { type Connect, defineConfig, type Plugin } from "vite";

const root = fileURLToPath(new URL(".", import.meta.url));

function reportPlugin(): Plugin {
  const handler: Connect.NextHandleFunction = (req, res, next) => {
    if (req.url !== "/__spike/report" || req.method !== "POST") {
      next();
      return;
    }
    let body = "";
    req.on("data", (chunk: Buffer) => {
      body += chunk.toString("utf8");
    });
    req.on("end", () => {
      const line = `${body.replace(/\n/g, " ")}\n`;
      const file = process.env.SPIKE_REPORT;
      if (file) appendFileSync(file, line);
      else process.stdout.write(line);
      res.statusCode = 204;
      res.end();
    });
  };
  return {
    name: "spike-report",
    configureServer(server) {
      server.middlewares.use(handler);
    },
    configurePreviewServer(server) {
      server.middlewares.use(handler);
    },
  };
}

export default defineConfig({
  root,
  plugins: [react(), tailwindcss(), reportPlugin()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  preview: { port: 1420, strictPort: true },
  build: {
    outDir: "../../dist-spike",
    emptyOutDir: true,
    rollupOptions: { input: { index: `${root}index.html`, app: `${root}app.html` } },
  },
});
