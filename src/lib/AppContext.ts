import { AppDispatch } from "@/AppAction";
import { Context, createContext } from "react";

export const AppContext: Context<AppDispatch> = createContext((action) => console.error("action occurred outside app tree:", action));
