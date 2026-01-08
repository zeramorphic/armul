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
    memory: Map<number, LineInfo | null>,
    /** The ranges of memory that is currently on screen for the disassembly view. */
    visible_memory_disas: { start: number, end: number },
    /** The ranges of memory that is currently on screen for the memory view. */
    visible_memory_memory: { start: number, end: number },
    registers: Registers,
    info: ProcessorInformation,
    program_output: string,
};

type ProcessorState = 'Running' | 'Stopped';

interface ProcessorInformation {
    file: string,
    state: ProcessorState,
    previous_pc: number,

    steps: number,
    nonseq_cycles: number,
    seq_cycles: number,
    internal_cycles: number,
}

export function newProcessor(): Processor {
    return {
        memory: new Map(),
        registers: { regs: Array(37).fill(0) },
        visible_memory_disas: { start: 0, end: 0 },
        visible_memory_memory: { start: 0, end: 0 },
        info: {
            file: 'unknown',
            state: 'Stopped',
            previous_pc: 0,
            steps: 0, nonseq_cycles: 0, seq_cycles: 0, internal_cycles: 0,
        },
        program_output: "",
    };
}

/** Get back in sync with the backend using Tauri calls. */
export async function resynchronise(processor: Processor): Promise<Processor> {
    const registers: Registers = await invoke('registers');
    const info: ProcessorInformation = await invoke('processor_info');
    const keys = [];
    for (var i = processor.visible_memory_disas.start; i < processor.visible_memory_disas.end; i += 4) {
        keys.push(i);
    }
    for (var i = processor.visible_memory_memory.start; i < processor.visible_memory_memory.end; i += 4) {
        keys.push(i);
    }
    const entries = await Promise.all(keys.map(addr => fetch_memory(addr).then((mem) => ({ addr, mem }))));
    const memory = new Map();
    for (const { addr, mem } of entries) {
        memory.set(addr, mem);
    }
    return { registers, visible_memory_disas: processor.visible_memory_disas, visible_memory_memory: processor.visible_memory_memory, memory, info, program_output: processor.program_output }
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
    return await invoke('line_at', { addr });
}
