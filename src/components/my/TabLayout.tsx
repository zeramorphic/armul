import { IJsonModel, Layout, Model, TabNode } from "flexlayout-react";
import Status from "./Status";
import { MemoryView } from "./MemoryView";
import Registers from "./Registers";
import './TabLayout.css';
import Terminal from "./Terminal";

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
        type: "row",
        weight: 20,
        children: [
          {
            type: "tabset",
            enableTabStrip: false,
            weight: 30,
            children: [
              {
                type: "tab",
                enableClose: false,
                name: "Status",
                component: "status",
              }
            ],
          },
          {
            type: "tabset",
            weight: 70,
            children: [
              {
                type: "tab",
                enableClose: false,
                name: "Registers",
                component: "registers",
              }
            ]
          },
        ]
      },
      {
        type: "row",
        weight: 80,
        children: [
          {
            type: "tabset",
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
            type: "row",
            weight: 30,
            children: [
              {
                type: "tabset",
                weight: 50,
                children: [
                  {
                    type: "tab",
                    enableClose: false,
                    name: "Memory",
                    component: "memory",
                  }
                ],
              },
              {
                type: "tabset",
                weight: 50,
                children: [
                  {
                    type: "tab",
                    enableClose: false,
                    name: "Terminal",
                    component: "terminal",
                  }
                ]
              },
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
    case 'registers': return <Registers />;
    case 'disas': return <MemoryView mode={'Disassemble'} />;
    case 'memory': return <MemoryView mode={'Memory'} />;
    case 'terminal': return <Terminal />;
  }
};

export default function TabLayout() {
  return <Layout model={model} factory={factory} realtimeResize={true} />
}
