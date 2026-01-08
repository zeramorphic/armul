import { useContext } from "react";
import { Input } from "../ui/input";
import { ProcessorContext } from "@/lib/ProcessorContext";

export default function Terminal() {
  const processor = useContext(ProcessorContext);
  const terminalOutput = processor.program_output;

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
    <Input placeholder="Program input here" className="flex-none font-mono"></Input>
  </div >;
}
