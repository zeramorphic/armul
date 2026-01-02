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
        weight: 30,
        children: [
          {
            type: "tab",
            enableClose: false,
            name: "One",
            component: "placeholder",
          }
        ]
      },
      {
        type: "row",
        weight: 70,
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

function App() {
  const [generation, setGeneration] = useState(0);
  const theme = useTheme();

  const model = Model.fromJson(modelJson);

  const factory = (node: TabNode) => {
    const component = node.getComponent();

    switch (component) {
      case 'placeholder': return <div>{node.getName()}</div>;
      case 'disas': return <MemoryView mode={'Disassemble'} generation={generation} />
      case 'memory': return <MemoryView mode={'Memory'} generation={generation} />
    }
  }

  return (
    <>
      <main className="container">
        <div className="row">
          <Menu openFile={file => {
            console.log("Loading", file);
            const loadEnvFile = async () => {
              const contents = await file.text();
              await invoke("load_program", { contents });
              setGeneration(i => i + 1);
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
          <Layout model={model} factory={factory} supportsPopout={true} onModelChange={(model, _action) => {
            // Save the model to JSON so that when we change to/from dark mode everything stays put.
            modelJson = model.toJson();
          }} />
        </div>
      </main>
      <Toaster />
    </>
  );
}

export default App;
