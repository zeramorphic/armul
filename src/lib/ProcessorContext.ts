import Processor from "@/lib/processor";
import { createContext } from "react";

export const ProcessorContext = createContext(new Processor(() => { }));
