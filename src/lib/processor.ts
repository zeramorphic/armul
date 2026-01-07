import { invoke } from "@tauri-apps/api/core";
import { LineInfo, Registers } from "./serde-types";
import { AppDispatch } from "@/App";

/**
 * A copy of some of the processor's information,
 * stored on the Typescript side for ease of access.
 *
 * Also manages the asynchronous retrieval of more information.
 */
export interface Processor {
    /** Nulls mean that we're waiting to receive the value. */
    memory: Map<number, LineInfo | null>;
    registers: Registers;
};

export function newProcessor(): Processor {
    return { memory: new Map(), registers: { regs: Array(37).fill(0) } };
}

/** Get back in sync with the backend using Tauri calls. */
export async function resynchronise(processor: Processor): Promise<Processor> {
    const registers: Registers = await invoke('registers');
    // For now, update all memory addresses we've ever seen.
    const entries = await Promise.all([...processor.memory.keys()]
        .map(addr => fetch_memory(addr).then((mem) => ({ addr, mem }))));
    const memory = new Map();
    for (const { addr, mem } of entries) {
        memory.set(addr, mem);
    }
    return { registers, memory }
}

/**
 * Gets the memory at this address, fetching it asynchronously
 * (but returning undefined) if not present.
 * When the promise resolves, we dispatch a processor update event
 * with the new data.
 * */
export function get_memory(processor: Processor, addr: number, dispatch: AppDispatch): LineInfo | undefined {
    const value = processor.memory.get(addr);
    if (value === undefined) {
        // Put a null in the memory store so we don't re-request this value.
        // This doesn't count as a mutation for React's purposes.
        processor.memory.set(addr, null);
        // Asynchronously get the value, calling the dispatcher once we have the result.
        fetch_memory(addr).then(info => {
            dispatch({ type: "processor_read", addr, info })
        });
        return undefined;
    } else {
        return value ?? undefined;
    }
}

/**
 * Fetch the value of the given memory address.
 */
export async function fetch_memory(addr: number): Promise<LineInfo> {
    return await invoke('line_at', { addr, disassemble: true });
}
