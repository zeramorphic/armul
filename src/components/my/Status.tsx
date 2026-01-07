import { PlayIcon, RefreshCwIcon, StepBackIcon, StepForwardIcon } from "lucide-react";
import { Button } from "../ui/button";
import { ButtonGroup } from "../ui/button-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { useHotkeys } from "react-hotkeys-hook";
import { ProcessorContext } from "@/lib/ProcessorContext";
import { useContext } from "react";
import { AppContext } from "@/lib/AppContext";
import { AppDispatch } from "@/App";
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
        <div>Status</div>
        <div className="flex-1"></div>
        <div>[Processor status]</div>
      </div>
      <div className="flex">
        <div>Cycles</div>
        <div className="flex-1"></div>
        <div>[Total cycles]</div>
      </div>
    </div>
  </div>;
}
