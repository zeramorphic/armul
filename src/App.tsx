import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "./components/ui/button";
import { Menu } from "./components/my/Menu";
import MemoryView from "./components/my/MemoryView";
import { TestApp } from "./components/my/Test";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
      <div className="row">
        <Menu />
      </div>

      <h1>Hello!</h1>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <Button variant="outline" type="submit">Greet</Button>
      </form>
      <p>{greetMsg}</p>

      {/* <div style={{ flex: `1`, backgroundColor: "cyan" }}>
      </div> */}
      {/* <div style={{ flex: `1 0 100%`, overflow: "auto" }}> */}
      <TestApp />
      {/* </div> */}

    </main>
  );
}

export default App;
