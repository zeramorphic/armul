import { ReactNode } from "react";
import * as processor from "./lib/processor";
import { LineInfo, Registers } from "./lib/serde-types";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { toast } from "sonner";
import { AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./components/ui/alert-dialog";
import { path } from "@tauri-apps/api";

export interface AppState {
  processor: processor.Processor,
  ready: boolean,
  setUserInput(userInput: string): void,
  stopPlaying(): void,
};

export function newAppState(): AppState {
  return {
    processor: processor.newProcessor(),
    ready: false,
    setUserInput(_) { },
    stopPlaying() { },
  }
}

export type AppAction
  = ProcessorRead
  | ProcessorUpdate
  | RequestProcessorUpdate
  | OpenFile
  | OpenFileResolve
  | UserInputUpdate
  | SetUserInputCallback
  | ToggleBreakpoint
  | SetPlaying
  | SimulationSpeed
  | Reset;

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
  newProcessor: (proc: processor.Processor) => processor.Processor,
}

interface RequestProcessorUpdate {
  type: "request_processor_update",
  dispatch: AppDispatch,
  /** The function to execute when the processor has been updated. */
  callback(processor: processor.Processor): void,
}

interface OpenFile {
  type: "open_file",
  dispatch: AppDispatch,
}

interface OpenFileResolve {
  type: "open_file_resolve",
  newProcessor: (proc: processor.Processor) => processor.Processor,
}

interface UserInputUpdate {
  type: "user_input_update",
  newUserInput: string,
}

interface SetUserInputCallback {
  type: "set_user_input_callback",
  callback: (userInput: string) => void,
}

interface ToggleBreakpoint {
  type: "toggle_breakpoint",
  address: number,
}

interface SetPlaying {
  type: "set_playing",
  playing: boolean,
  dispatch: AppDispatch,
}

interface SimulationSpeed {
  type: "simulation_speed",
  multiplier: number,
}

interface Reset {
  type: "reset",
  /** Whether a hard or soft reset was desired. */
  hard: boolean,
  dispatch: AppDispatch,
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
  console.log("Dispatching", action.type);
  switch (action.type) {
    case "processor_read":
      const memory = new Map(appState.processor.memory);
      memory.set(action.addr, action.info);
      return { ...appState, processor: { ...appState.processor, memory } };
    case "processor_update":
      return {
        ...appState,
        processor: action.newProcessor(appState.processor),
      };
    case "request_processor_update":
      (async () => {
        const update = await processor.resynchronise(appState.processor);
        action.dispatch({
          type: "processor_update", newProcessor(proc) {
            var newProc = update(proc);
            action.callback(newProc);
            return newProc;
          }
        });
      })();
      return appState;
    case "open_file":
      performOpenFile(appState.processor, action.dispatch, errorDialog);
      return appState;
    case "open_file_resolve":
      return {
        ...appState,
        ready: true,
        processor: action.newProcessor(appState.processor),
      };
    case "user_input_update":
      appState.setUserInput(action.newUserInput);
      return {
        ...appState,
      }
    case "set_user_input_callback":
      return {
        ...appState,
        setUserInput: action.callback,
      }
    case "toggle_breakpoint":
      const breakpoints = new Set(appState.processor.breakpoints);
      if (breakpoints.has(action.address)) {
        breakpoints.delete(action.address);
      } else {
        breakpoints.add(action.address);
      }
      return { ...appState, processor: { ...appState.processor, breakpoints } }
    case "set_playing":
      console.log("Playing:", action.playing);
      if (action.playing) {
        // Spawn a new task to continuously step the processor.
        return {
          ...appState,
          stopPlaying: play(appState.processor, action.dispatch),
          processor: { ...appState.processor, playing: true },
        }
      } else {
        // Stop the task that's stepping the processor.
        appState.stopPlaying();
        return {
          ...appState,
          stopPlaying() { },
          processor: { ...appState.processor, playing: false },
        }
      }
    case "simulation_speed":
      var newSpeed = appState.processor.simulation_speed * action.multiplier;
      if (newSpeed < 1) {
        newSpeed = 1;
      } else if (newSpeed > 512) {
        newSpeed = 512;
      }
      return { ...appState, processor: { ...appState.processor, simulation_speed: newSpeed } };
    case "reset":
      (async () => {
        await invoke('reset', { 'hard': action.hard });
        action.dispatch({ type: "request_processor_update", dispatch: action.dispatch, callback() { } });
      })();
      return appState;
  }
}

/**
 * Spawns a task that continuously steps the processor.
 * Returns a callback that can be invoked to stop this task.
 */
function play(processor: processor.Processor, dispatch: AppDispatch): () => void {
  const status = { processor, shouldStop: false };

  const singleStep = async () => {
    // Perform a single step (or more if the simulation speed was increased).
    var newUserInput: string | undefined = undefined;
    var shouldStop = false;
    for (let i = 0; i < processor.simulation_speed; i++) {
      const nextUserInput: string | undefined = await invoke('step_once');
      if (nextUserInput !== undefined) newUserInput = nextUserInput;

      // Check if the processor is now stopped or at a breakpoint.
      const info: processor.ProcessorInformation = await invoke('processor_info');
      if (!('Ok' in info.state) || info.state.Ok != "Running") {
        shouldStop = true;
        break;
      }

      const registers: Registers = await invoke('registers');
      if (processor.breakpoints.has(registers.regs[15])) {
        shouldStop = true;
        await invoke('hit_breakpoint');
        break;
      }
    }

    // Dispatch UI updates.
    if (newUserInput) {
      dispatch({ type: "user_input_update", newUserInput })
    }
    if (shouldStop) {
      status.shouldStop = true;
      dispatch({ type: "set_playing", dispatch, playing: false });
    }

    // Request a processor update. This refreshes the UI.
    dispatch({
      type: "request_processor_update",
      dispatch,
      callback(newProcessor) {
        processor = newProcessor;
        // Once the processor has been updated, check the `shouldStop` flag and the processor's status.
        if (!status.shouldStop) {
          // If we don't need to stop, go again.
          singleStep();
        }
      },
    });
  };

  singleStep();

  return () => { status.shouldStop = true; };
}
