import { Info, MinusIcon, PauseIcon, PlayIcon, PlusIcon, RefreshCcwDotIcon, RefreshCcwIcon, StepForwardIcon } from "lucide-react";
import { Button } from "../ui/button";
import { ButtonGroup } from "../ui/button-group";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { useHotkeys } from "react-hotkeys-hook";
import { ProcessorContext } from "@/lib/ProcessorContext";
import { useContext } from "react";
import { DispatchContext } from "@/lib/DispatchContext";
import { AppDispatch } from "@/AppAction";
import { Processor, resynchronise } from "@/lib/processor";
import { invoke } from "@tauri-apps/api/core";

async function stepOnce(processor: Processor, dispatch: AppDispatch) {
  if (processor.playing)
    return;

  const newUserInput: string | undefined = await invoke('step_times', { steps: 1 });
  if (newUserInput) {
    dispatch({ type: "user_input_update", newUserInput })
  }
  const newProcessor = await resynchronise(processor);
  dispatch({ type: "processor_update", newProcessor });
}

async function play(dispatch: AppDispatch) {
  dispatch({ type: "set_playing", playing: true, dispatch });
}

async function pause(dispatch: AppDispatch) {
  dispatch({ type: "set_playing", playing: false, dispatch });
}

export default function Status() {
  const processor = useContext(ProcessorContext);
  const dispatch = useContext(DispatchContext);
  useHotkeys('f2', () => stepOnce(processor, dispatch), { preventDefault: true });
  useHotkeys('f5', () => { processor.playing ? pause(dispatch) : play(dispatch) }, { preventDefault: true });
  useHotkeys('-', () => dispatch({ type: "simulation_speed", multiplier: 0.5 }), { useKey: true });
  useHotkeys('=', () => dispatch({ type: "simulation_speed", multiplier: 2 }), { useKey: true });
  useHotkeys('ctrl+r', () => dispatch({ type: "reset", hard: false, dispatch }), { preventDefault: true });
  useHotkeys('ctrl+shift+r', () => dispatch({ type: "reset", hard: true, dispatch }), { preventDefault: true });

  var state;
  if ('Ok' in processor.info.state) {
    state = processor.info.state.Ok;
  } else {
    state = <span className="text-destructive">{processor.info.state.Err}</span>;
  }

  return <div className="flex flex-col">
    <div className="flex flex-row p-2 justify-center">
      <ButtonGroup>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" className="rounded" onClick={() => { processor.playing ? pause(dispatch) : play(dispatch) }}>
              {processor.playing ? <PauseIcon /> : <PlayIcon />}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            {processor.playing ? "Pause" : "Run"}&emsp;<span className="rounded text-muted-foreground tracking-widest ml-auto">F5</span>
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" onClick={() => stepOnce(processor, dispatch)} disabled={processor.playing}><StepForwardIcon /></Button>
          </TooltipTrigger>
          <TooltipContent>
            Step Once&emsp;<span className="rounded text-muted-foreground tracking-widest ml-auto">F2</span>
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" className="rounded" disabled={processor.playing} onClick={() => dispatch({ type: "reset", hard: false, dispatch })}><RefreshCcwIcon /></Button>
          </TooltipTrigger>
          <TooltipContent>
            Soft reset (preserving memory and registers)&emsp;<span className="rounded text-muted-foreground tracking-widest ml-auto">Ctrl+R</span>
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="outline" className="rounded" disabled={processor.playing} onClick={() => dispatch({ type: "reset", hard: true, dispatch })}><RefreshCcwDotIcon /></Button>
          </TooltipTrigger>
          <TooltipContent>
            Hard reset (including memory and registers)&emsp;<span className="rounded text-muted-foreground tracking-widest ml-auto">Ctrl+Shift+R</span>
          </TooltipContent>
        </Tooltip>
      </ButtonGroup>
    </div>
    <div className="flex flex-row p-2 pt-0 justify-center">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="outline" className="rounded-l" onClick={() => dispatch({ type: "simulation_speed", multiplier: 0.5 })}><MinusIcon className="size-3" /></Button>
        </TooltipTrigger>
        <TooltipContent>
          Slow down simulation&emsp;<span className="rounded text-muted-foreground tracking-widest ml-auto">-</span>
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="w-11 grid place-items-center text-sm">{processor.simulation_speed}&#xd7;</div>
        </TooltipTrigger>
        <TooltipContent>
          Speed of the simulation
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="outline" className="rounded-r" onClick={() => dispatch({ type: "simulation_speed", multiplier: 2.0 })}><PlusIcon className="size-3" /></Button>
        </TooltipTrigger>
        <TooltipContent>
          Speed up simulation&emsp;<span className="rounded text-muted-foreground tracking-widest ml-auto">=</span>
        </TooltipContent>
      </Tooltip>
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
        <div>{state}</div>
      </div>
      <div className="flex">
        <Tooltip>
          <TooltipTrigger asChild>
            <div>Steps <Info className="inline" height={16} /></div>
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
            <div>Processor time <Info className="inline" height={16} /></div>
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
