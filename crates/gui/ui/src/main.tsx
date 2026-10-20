import { ThemeProvider } from "next-themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { HashRouter } from "react-router";
import App from "./App";
import { themeOptions } from "./theme";
import "../styles.css";

// Suppress WebKit's native context menu; custom triggers still receive the event.
if (/Macintosh|MacIntel/.test(navigator.userAgent)) {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider {...themeOptions}>
      <HashRouter>
        <App />
      </HashRouter>
    </ThemeProvider>
  </StrictMode>,
);
