import { useContext, useEffect, useRef, useState } from 'react';
import MemoryRow from './MemoryRow';
import { List, useListRef } from 'react-window';
import { ProcessorContext } from '@/lib/ProcessorContext';
import { Tooltip, TooltipContent, TooltipTrigger } from '../ui/tooltip';
import { Input } from '../ui/input';

interface MemoryViewProps {
  mode: 'Disassemble' | 'Memory',
}

function rowsFor(index: number) { return 1 << (Math.log2(index + 100) + 1) };

export function MemoryView(props: MemoryViewProps) {
  const [rowCount, setRowCount] = useState(128);
  const processor = useContext(ProcessorContext);

  const [desiredRow, setDesiredRow] = useState<number | undefined>(undefined);
  const [desireMoveTriggered, setDesireMoveTriggered] = useState(true);

  const rowsRendered = (startIndex: number, stopIndex: number) => {
    // Every so often, double the scrollable row count.
    if (desiredRow === undefined) {
      setRowCount(rowsFor(stopIndex));
    } else if (startIndex <= desiredRow && desiredRow <= stopIndex) {
      setDesiredRow(undefined);
    }
    if (props.mode === "Disassemble") {
      processor.visible_memory_disas.start = startIndex * 4;
      processor.visible_memory_disas.end = stopIndex * 4;
    } else {
      processor.visible_memory_memory.start = startIndex * 4;
      processor.visible_memory_memory.end = stopIndex * 4;
    }
  };

  const goToRef = useRef<HTMLInputElement>(null);
  const listRef = useListRef(null);

  useEffect(() => {
    if (desiredRow !== undefined && listRef.current !== null) {
      if (desiredRow < rowCount) {
        if (!desireMoveTriggered) {
          listRef.current.scrollToRow({ align: "center", index: desiredRow, behavior: "smooth" });
          setDesireMoveTriggered(true);
        }
      } else {
        setRowCount(rowsFor(desiredRow));
      }
    }
  }, [rowCount, desiredRow, listRef, desireMoveTriggered]);

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
      listRef={listRef}
      className="mx-2"
      rowComponent={MemoryRow}
      onRowsRendered={(_, { startIndex, stopIndex }) => rowsRendered(startIndex, stopIndex)}
      rowCount={rowCount}
      rowHeight={16}
      overscanCount={20}
      rowProps={{ mode: props.mode }}>
    </List>
    <div className="absolute bottom-4 right-4 flex flex-row z-30">
      <Input ref={goToRef} className="h-6 font-mono" size={10} placeholder="Go to..." onKeyDown={(event) => {
        if (event.key === 'Enter') {
          const value = parseInt(goToRef.current?.value ?? "", 16);
          if (!Number.isNaN(value)) {
            setDesiredRow(value >>> 2);
            setDesireMoveTriggered(false);
          }
        }
      }}></Input>
    </div>
  </div >;
}
