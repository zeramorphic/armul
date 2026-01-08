import { PlayIcon, RefreshCwIcon, StepBackIcon, StepForwardIcon } from "lucide-react";
import { Button } from "../ui/button";
import { ButtonGroup } from "../ui/button-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { useHotkeys } from "react-hotkeys-hook";
import { ProcessorContext } from "@/lib/ProcessorContext";
import { useContext } from "react";
import { AppContext } from "@/lib/AppContext";
import { AppDispatch } from "@/AppAction";
import { Processor, resynchronise } from "@/lib/processor";
import { invoke } from "@tauri-apps/api/core";

async function stepOnce(processor: Processor, dispatch: AppDispatch) {
  await invoke('step_once');
  const newProcessor = await resynchronise(processor);
  dispatch({ type: "processor_update", newProcessor })
}

export default function Status() {
  const processor = useContext(ProcessorContext);
  const dispatch = useContext(AppContext);
  useHotkeys('f2', () => stepOnce(processor, dispatch));

  return <div className="flex flex-col">
    <div className="flex flex-row p-2 justify-center">
      <ButtonGroup>
        <Button variant="outline" className="rounded"><StepBackIcon /></Button>
        <Button variant="outline"><PlayIcon /></Button>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" onClick={() => stepOnce(processor, dispatch)}><StepForwardIcon /></Button>
          </TooltipTrigger>
          <TooltipContent>
            Step Once&emsp;<span className="text-muted-foreground tracking-widest ml-auto">F2</span>
          </TooltipContent>
        </Tooltip>
        <Button variant="outline" className="rounded"><RefreshCwIcon /></Button>
      </ButtonGroup>
    </div>
    <div className="text-sm px-2">
      <div className="flex">
        <div>File</div>
        <div className="flex-1"></div>
        <div>{processor.info.file}</div>
      </div>
      <div className="flex">
        <div>Status</div>
        <div className="flex-1"></div>
        <div>{processor.info.state}</div>
      </div>
      <div className="flex">
        <Tooltip>
          <TooltipTrigger asChild>
            <div>Steps &#9432;</div>
          </TooltipTrigger>
          <TooltipContent>
            Counts the number of instructions that have been executed.
          </TooltipContent>
        </Tooltip>
        <div className="flex-1"></div>
        <div className="font-mono">{processor.info.steps}</div>
      </div>
      <div className="flex">
        <Tooltip>
          <TooltipTrigger asChild>
            <div>Processor time &#9432;</div>
          </TooltipTrigger>
          <TooltipContent>
            An estimation of the amount of time it would have taken to run this program on a real ARM7TDMI chip.
          </TooltipContent>
        </Tooltip>
        <div className="flex-1"></div>
        <div className="font-mono">~{((processor.info.nonseq_cycles * 2 + processor.info.seq_cycles + processor.info.internal_cycles) / 100).toFixed(2)}&micro;s</div>
      </div>
    </div>
  </div>;
}
