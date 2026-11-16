import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import "./app/theme";
import { App } from "./App";
import { applyConfiguredLanguage } from "./lib/i18n/locale";

// Until config.toml arrives, the system language decides (DEB-10).
applyConfiguredLanguage(null);

const root = document.getElementById("root");
if (!root) {
  throw new Error("#root element not found");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
