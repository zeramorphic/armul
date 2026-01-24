import { LazyStore } from "@tauri-apps/plugin-store";
import { createContext } from "react";

export const StoreContext = createContext(new LazyStore('settings.json'));
