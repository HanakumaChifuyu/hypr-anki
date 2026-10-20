import { ThemeProvider } from "next-themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { HashRouter } from "react-router";
import App from "./App";
import { themeOptions } from "./theme";
import "../styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider {...themeOptions}>
      <HashRouter>
        <App />
      </HashRouter>
    </ThemeProvider>
  </StrictMode>,
);
