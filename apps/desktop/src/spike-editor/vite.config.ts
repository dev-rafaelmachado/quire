// T9 spike (disposable, v0.2): CodeMirror 6 in the real Tauri webview. Separate Vite project
// rooted at src/spike-editor, never part of the app build. The page POSTs its metrics to
// /__spike/report, appended to $SPIKE_REPORT (or printed).
import { appendFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
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
  plugins: [reportPlugin()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  preview: { port: 1420, strictPort: true },
  build: { outDir: "../../dist-spike-editor", emptyOutDir: true },
});
