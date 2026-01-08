import { ChangeEventHandler, RefObject, useContext, useEffect, useRef } from "react";
import { Input } from "../ui/input";
import { ProcessorContext } from "@/lib/ProcessorContext";
import { DispatchContext } from "@/lib/DispatchContext";
import { invoke } from "@tauri-apps/api/core";

export default function Terminal() {
  const dispatch = useContext(DispatchContext);
  const processor = useContext(ProcessorContext);
  const inputRef: RefObject<HTMLInputElement | null> = useRef(null);
  const terminalOutput = processor.info.output;

  useEffect(() => {
    dispatch({
      type: "set_user_input_callback",
      callback(userInput) {
        console.log("Updated user input from the backend.");
        const input = inputRef.current;
        if (input) {
          input.value = userInput;
        }
      }
    })
  }, []);

  const inputChange: ChangeEventHandler<HTMLInputElement> = (event) => {
    (async () => { await invoke('set_user_input', { userInput: event.target.value }); })();
  };

  var body;
  if (terminalOutput === "") {
    body = <div className="font-mono text-xs flex-1 overflow-scroll TerminalContent">
      <pre className="p-2 text-muted-foreground">
        Program output here
      </pre>
    </div>;
  } else {
    body = <div className="font-mono text-sm flex-1 overflow-scroll TerminalContent">
      <pre className="p-2">
        {terminalOutput}
        <span className="animate-caret-blink bg-primary w-[5px] h-[16px] my-[-2px] inline-block"></span>
      </pre>
    </div>;
  }

  return <div className="flex flex-col" style={{ height: "100%", maxHeight: "100%" }}>
    {body}
    <Input ref={inputRef} onChange={inputChange} placeholder="Program input here" className="flex-none font-mono"></Input>
  </div >;
}
