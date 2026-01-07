import { registerToString, renderNumber } from './MemoryRow';
import { ButtonGroup } from '../ui/button-group';
import { Button } from '../ui/button';
import { PlayIcon, RefreshCwIcon, StepForwardIcon } from 'lucide-react';
import { useContext } from 'react';
import { ProcessorContext } from '@/lib/ProcessorContext';
import Processor from '@/lib/processor';
import { invoke } from '@tauri-apps/api/core';
import { useHotkeys } from 'react-hotkeys-hook';
import { Tooltip, TooltipContent, TooltipTrigger } from '../ui/tooltip';

function Vspace() {
  return <div style={{ paddingBottom: `5px` }}></div>;
}

async function stepOnce(processor: Processor) {
  await invoke('step_once');
  await processor.resynchronise();
}

export default function Registers() {
  const processor = useContext(ProcessorContext);

  useHotkeys('f2', () => stepOnce(processor));

  const transport = <ButtonGroup>
    <Button variant="outline"><PlayIcon /></Button>
    <Tooltip>
      <TooltipTrigger asChild>
        <Button variant="outline" onClick={() => stepOnce(processor)}><StepForwardIcon /></Button>
      </TooltipTrigger>
      <TooltipContent>
        Step Once&emsp;<span className="text-muted-foreground tracking-widest ml-auto">F2</span>
      </TooltipContent>
    </Tooltip>
    <Button variant="outline"><RefreshCwIcon /></Button>
  </ButtonGroup>;

  const registers = useContext(ProcessorContext).registers;
  const cpsr = <span>Flags: {registers.regs[31].toString(16).toUpperCase().padStart(8, '0')}</span>;

  return <div className="status">
    <div className="w-full flex-none flex flex-row text-sm px-2 bg-(--muted)">
      <div className="text-(--muted-foreground) flex-none w-[50px]">Reg</div>
      <div className="text-(--muted-foreground) flex-none w-[80px]">Hex</div>
      <div className="text-(--muted-foreground) flex-1">Decimal</div>
    </div>
    {[...Array(16).keys()].map(n => {
      const hexStr = registers.regs[n].toString(16).toUpperCase();
      const hex = <><span className="text-(--extremely-muted-foreground)">{'0'.repeat(8 - hexStr.length)}</span>{hexStr}</>;
      return <div key={n} className="flex flex-row px-2 font-mono text-sm">
        <div className="flex-none w-[50px]">{registerToString(n)}</div>
        <div className="flex-none w-[80px]">{hex}</div>
        <div className="flex-1">{renderNumber(registers.regs[n])}</div>
      </div>;
    })}
    <Vspace />
    {transport}
    <Vspace />
    {cpsr}
  </div>;
}
