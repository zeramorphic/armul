import { ReactNode } from "react";
import * as processor from "./lib/processor";
import { LineInfo } from "./lib/serde-types";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { toast } from "sonner";
import { AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./components/ui/alert-dialog";
import { path } from "@tauri-apps/api";

export interface AppState {
  processor: processor.Processor,
  ready: boolean,
};

export function newAppState(): AppState {
  return {
    processor: processor.newProcessor(),
    ready: false,
  }
}

export type AppAction = ProcessorRead | ProcessorUpdate | OpenFile | OpenFileResolve;

export type AppDispatch = (action: AppAction) => void;

/**
 * We asynchronously read some memory from the Rust-side processor.
 * We now need to store it in the TS-side processor state.
 */
interface ProcessorRead {
  type: "processor_read",
  addr: number,
  info: LineInfo,
}

interface ProcessorUpdate {
  type: "processor_update",
  newProcessor: processor.Processor,
}

interface OpenFile {
  type: "open_file",
  dispatch: AppDispatch,
}

interface OpenFileResolve {
  type: "open_file_resolve",
  newProcessor: processor.Processor,
}

async function performOpenFile(proc: processor.Processor, dispatch: AppDispatch, errorDialog: (contents: ReactNode) => void) {
  const filePath = await open({
    filters: [{ name: "Assembly file (.s)", extensions: ['s', 'S'] }]
  });
  if (!filePath) return;
  const baseName = await path.basename(filePath);

  const loadProgram = async () => {
    await invoke("load_program", { path: filePath });
    const newProcessor = await processor.resynchronise(proc);
    dispatch({ type: "open_file_resolve", newProcessor });
  };
  toast.promise(loadProgram().catch((errs: { line_number: number, error: string }[]) => {
    errorDialog(<>
      <AlertDialogHeader>
        <AlertDialogTitle>
          Could not load {baseName}
        </AlertDialogTitle>
        <AlertDialogDescription className="text-sm">
          {errs.map(({ line_number, error }) => <div>
            Line {line_number}: {error}
          </div>)}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Ok</AlertDialogCancel>
      </AlertDialogFooter>
    </>);
    throw errs
  }), {
    loading: "Loading " + baseName + "...",
    success: "Loaded " + baseName + "."
  });
}

export function performAction(
  appState: AppState,
  action: AppAction,
  errorDialog: (contents: ReactNode) => void,
): AppState {
  switch (action.type) {
    case "processor_read":
      const memory = new Map(appState.processor.memory);
      memory.set(action.addr, action.info);
      return { ...appState, processor: { ...appState.processor, memory } };
    case "processor_update":
      return {
        ...appState,
        processor: action.newProcessor,
      };
    case "open_file":
      performOpenFile(appState.processor, action.dispatch, errorDialog);
      return appState;
    case "open_file_resolve":
      return {
        ...appState,
        ready: true,
        processor: action.newProcessor,
      };
  }
}
