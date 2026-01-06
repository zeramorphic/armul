import { invoke } from "@tauri-apps/api/core";
import { LineInfo, Registers } from "./serde-types";

/**
 * A copy of some of the processor's information,
 * stored on the Typescript side for ease of access.
 *
 * Also manages the asynchronous retrieval of more information.
 */
export default class Processor {
    private onUpdate: () => void;
    private memory: Map<number, LineInfo | null>;
    registers: Registers;

    /**
     * The parameter is a callback to be invoked whenever the processor's state changes.
     * This may be called asynchronously.
     */
    constructor(onUpdate: () => void) {
        this.onUpdate = onUpdate;
        this.memory = new Map();
        this.registers = { regs: new Array(37).fill(0) };
    }

    /** Get back in sync with the backend. Calls `onUpdate`. */
    async resynchronise() {
        this.registers = await invoke('registers');
        // For now, update all memory addresses we've ever seen.
        await Promise.all([...this.memory.keys()].map(addr => this.fetch_memory(addr)));
        this.onUpdate();
    }

    /** Unpacks and repacks a processor so that React doesn't know it's the same thing. */
    repack(): Processor {
        const proc = new Processor(this.onUpdate);
        proc.memory = this.memory;
        proc.registers = this.registers;
        return proc;
    }

    /** Gets the memory at this address, fetching it asynchronously (but returning undefined) if not present. */
    get_memory(addr: number): LineInfo | undefined {
        const value = this.memory.get(addr);
        if (value === undefined) {
            // Put a null in the memory store so we don't re-request this value.
            // This doesn't count as a mutation for the purposes of `onUpdate`.
            this.memory.set(addr, null);
            // Asynchronously get the value, calling onUpdate once we have the result.
            this.fetch_memory(addr, true);
            return undefined;
        } else {
            return value ?? undefined;
        }
    }

    /**
     * Fetch the value of the given memory address.
     * If `callOnUpdate` is true, `onUpdate` is called.
     */
    async fetch_memory(addr: number, callOnUpdate?: boolean): Promise<LineInfo> {
        const info: LineInfo = await invoke('line_at', { addr, disassemble: true });
        this.memory.set(addr, info);
        if (callOnUpdate)
            this.onUpdate();
        return info;
    }
};
