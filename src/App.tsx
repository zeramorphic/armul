import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Menu } from "./components/my/Menu";
import { TestApp } from "./components/my/MemoryView";
import { Toaster } from "./components/ui/sonner";
import { toast } from "sonner";

function App() {
  const [generation, setGeneration] = useState(0);

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

        <TestApp generation={generation} />
      </main>
      <Toaster />
    </>
  );
}

export default App;
