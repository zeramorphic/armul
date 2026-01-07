import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Menu } from "./components/my/Menu";
import { MemoryView } from "./components/my/MemoryView";
import { Toaster } from "./components/ui/sonner";
import { toast } from "sonner";
import { IJsonModel, Layout, Model, TabNode } from 'flexlayout-react';
import './flexlayout.css';
import { useTheme } from "./components/theme-provider";
import Status from "./components/my/Status";
import Processor from "./lib/processor";
import { ProcessorContext } from "./lib/ProcessorContext";

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
            name: "Status",
            component: "status",
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
    case 'status': return <Status />;
    case 'disas': return <MemoryView mode={'Disassemble'} />;
    case 'memory': return <MemoryView mode={'Memory'} />;
  }
};

function App() {
  const [processor, setProcessor] = useState<Processor>(new Processor(() => setProcessor(proc => proc.repack())));
  const theme = useTheme();

  return (
    <>
      <main className="container">
        <div className="row">
          <Menu openFile={file => {
            console.log("Loading", file);
            const loadEnvFile = async () => {
              const contents = await file.text();
              await invoke("load_program", { contents });
              await processor.resynchronise();
            };
            toast.promise(loadEnvFile().catch(err => {
              console.error("Couldn't load", file, err);
              throw err
            }), {
              loading: "Loading " + file.name + "...",
              success: "Loaded " + file.name + ".",
              error: "Could not load " + file.name + "."
            });
          }} />
        </div>

        <div className={'mainbody row ' + (theme.theme === 'light' ? "flexlayout__theme_light" : "flexlayout__theme_dark")}>
          <ProcessorContext value={processor}>
            <Layout model={model} factory={factory} realtimeResize={true} />
          </ProcessorContext>
        </div>
      </main>
      <Toaster />
    </>
  );
}

export default App;
