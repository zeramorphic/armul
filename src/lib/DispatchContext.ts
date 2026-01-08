import { AppDispatch } from "@/AppAction";
import { Context, createContext } from "react";

export const DispatchContext: Context<AppDispatch> = createContext((action) => console.error("action occurred outside app tree:", action));
