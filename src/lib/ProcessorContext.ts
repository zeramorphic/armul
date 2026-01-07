import { newProcessor } from "@/lib/processor";
import { createContext } from "react";

export const ProcessorContext = createContext(newProcessor());
