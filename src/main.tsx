import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./components/theme-provider";
import { StoreContext } from "./lib/StoreContext";
import { LazyStore } from "@tauri-apps/plugin-store";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="light" storageKey="vite-ui-theme">
      <StoreContext value={new LazyStore('settings.json')}>
        <App />
      </StoreContext>
    </ThemeProvider>
  </React.StrictMode>,
);
