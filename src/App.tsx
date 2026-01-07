import { RefObject, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Menu } from "./components/my/Menu";
import { MemoryView } from "./components/my/MemoryView";
import { Toaster } from "./components/ui/sonner";
import { toast } from "sonner";
import { IJsonModel, Layout, Model, TabNode } from 'flexlayout-react';
import './flexlayout.css';
import { useTheme } from "./components/theme-provider";
import Registers from "./components/my/Registers";
import { ProcessorContext } from "./lib/ProcessorContext";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia } from "./components/ui/empty";
import { Binary } from "lucide-react";
import { Button } from "./components/ui/button";
import { AppContext } from "./lib/AppContext";
import { LineInfo } from "./lib/serde-types";
import * as processor from "./lib/processor";

var modelJson: IJsonModel = {
  global: {
    splitterSize: 4,
    splitterExtra: 4,
  },
  borders: [],
  layout: {
    type: "row",
    weight: 100,
    children: [
      {
        type: "tabset",
        enableSingleTabStretch: true,
        enableClose: false,
        weight: 20,
        children: [
          {
            type: "tab",
            enableClose: false,
            name: "Registers",
            component: "registers",
          }
        ]
      },
      {
        type: "row",
        weight: 80,
        children: [
          {
            type: "tabset",
            enableSingleTabStretch: true,
            weight: 70,
            children: [
              {
                type: "tab",
                enableClose: false,
                name: "Disassembly",
                component: "disas",
              }
            ],
          },
          {
            type: "tabset",
            enableSingleTabStretch: true,
            weight: 30,
            children: [
              {
                type: "tab",
                enableClose: false,
                name: "Memory",
                component: "memory",
              }
            ]
          },
        ]
      }
    ]
  }
};

const model = Model.fromJson(modelJson);
const factory = (node: TabNode) => {
  const component = node.getComponent();

  switch (component) {
    case 'placeholder': return <div>{node.getName()}</div>;
    case 'registers': return <Registers />;
    case 'disas': return <MemoryView mode={'Disassemble'} />;
    case 'memory': return <MemoryView mode={'Memory'} />;
  }
};

interface AppState {
  processor: processor.Processor,
  ready: boolean,
};

function newAppState(): AppState {
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
}

interface OpenFileResolve {
  type: "open_file_resolve",
  file: File,
  newProcessor: processor.Processor,
}

export function performOpenFile(proc: processor.Processor, dispatch: AppDispatch, file: File) {
  const loadProgram = async () => {
    const contents = await file.text();
    await invoke("load_program", { contents });
    const newProcessor = await processor.resynchronise(proc);
    dispatch({ type: "open_file_resolve", file, newProcessor });
  };
  toast.promise(loadProgram().catch(err => {
    console.error("Couldn't load", file, err);
    throw err
  }), {
    loading: "Loading " + file.name + "...",
    success: "Loaded " + file.name + ".",
    error: "Could not load " + file.name + "."
  });
}

function performAction(
  appState: AppState,
  action: AppAction,
  openFileInput: RefObject<HTMLInputElement | null>): AppState {
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
      openFileInput.current?.click();
      return appState;
    case "open_file_resolve":
      return {
        ...appState,
        ready: true,
        processor: action.newProcessor,
      };
  }
}

function App() {
  const [state, setState] = useState(newAppState());
  const theme = useTheme();
  const openFileInput = useRef<HTMLInputElement>(null);

  // A callback to execute the given action.
  const dispatch: AppDispatch = (action: AppAction) => setState(appState => performAction(appState, action, openFileInput));

  var body;
  if (state.ready) {
    body = <Layout model={model} factory={factory} realtimeResize={true} />;
  } else {
    body = <Empty>
      <EmptyMedia className="bg-muted p-3">
        <Binary size={28} />
      </EmptyMedia>
      <EmptyHeader>Emulator Ready</EmptyHeader>
      <EmptyDescription>
        Open an ARM assembly file (<span className="font-mono">.s</span>) to load it into the emulator.
      </EmptyDescription>
      <EmptyContent>
        <Button onClick={() => dispatch({ type: "open_file" })}>Open File</Button>
      </EmptyContent>
    </Empty>;
  }

  return (
    <>
      {/* TODO: The `accept` attribute isn't working as intended. */}
      <input type="file" style={{ "display": "none" }} ref={openFileInput} accept=".s," onChange={event => {
        if (event.target.files?.length === 1) {
          performOpenFile(state.processor, dispatch, event.target.files[0]);
        }
      }} />
      <AppContext value={dispatch}>
        <ProcessorContext value={state.processor}>
          <main className="container">
            <div className="row">
              <Menu />
            </div>

            <div className={'mainbody row ' + (theme.theme === 'light' ? "flexlayout__theme_light" : "flexlayout__theme_dark")}>
              {body}
            </div>
          </main>
          <Toaster />
        </ProcessorContext>
      </AppContext>
    </>
  );
}

export default App;
