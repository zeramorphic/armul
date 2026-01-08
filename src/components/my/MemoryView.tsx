import { useContext, useState } from 'react';
import MemoryRow from './MemoryRow';
import { List } from 'react-window';
import { ProcessorContext } from '@/lib/ProcessorContext';
import { Tooltip, TooltipContent, TooltipTrigger } from '../ui/tooltip';

interface MemoryViewProps {
  mode: 'Disassemble' | 'Memory',
}

export function MemoryView(props: MemoryViewProps) {
  const [rowCount, setRowCount] = useState(128);
  const processor = useContext(ProcessorContext);

  const rowsRendered = (startIndex: number, stopIndex: number) => {
    // Every so often, double the scrollable row count.
    setRowCount(1 << (Math.log2(stopIndex + 100) + 1));
    if (props.mode === "Disassemble") {
      processor.visible_memory_disas.start = startIndex * 4;
      processor.visible_memory_disas.end = stopIndex * 4;
    } else {
      processor.visible_memory_memory.start = startIndex * 4;
      processor.visible_memory_memory.end = stopIndex * 4;
    }
  };

  return <div className="flex flex-col" style={{ maxHeight: "100%" }}>
    <div className="w-full flex-none flex flex-row text-sm px-2 bg-(--muted)">
      {props.mode === "Disassemble" ?
        <div className="text-(--muted-foreground) flex-none w-[24px]">
          <Tooltip>
            <TooltipTrigger>
              BP
            </TooltipTrigger>
            <TooltipContent>
              Set breakpoints
            </TooltipContent>
          </Tooltip>
        </div>
        : <></>}
      <div className="text-(--muted-foreground) flex-none w-[50px]">
        <Tooltip>
          <TooltipTrigger>
            Regs
          </TooltipTrigger>
          <TooltipContent>
            Shows registers that are pointing to this address
          </TooltipContent>
        </Tooltip>
      </div>
      <div className="text-(--muted-foreground) flex-none w-[80px]">Address</div>
      <div className="text-(--muted-foreground) flex-none w-[80px]">Hex</div>
      <div className="text-(--muted-foreground) flex-none w-[180px]">{props.mode === "Disassemble" ? "Disassembly" : "Decimal"}</div>
      {props.mode === "Disassemble" ? <div className="text-(--muted-foreground) flex-none w-[80px]">Comment</div> : <></>}
    </div>
    <List
      className="mx-2"
      rowComponent={MemoryRow}
      onRowsRendered={(_, { startIndex, stopIndex }) => rowsRendered(startIndex, stopIndex)}
      rowCount={rowCount}
      rowHeight={16}
      overscanCount={20}
      rowProps={{ mode: props.mode }}>
    </List>
  </div >;
}
