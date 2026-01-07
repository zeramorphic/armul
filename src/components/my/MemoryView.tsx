import { useState } from 'react';
import MemoryRow from './MemoryRow';
import { List } from 'react-window';

interface MemoryViewProps {
  mode: 'Disassemble' | 'Memory',
}

export function MemoryView(props: MemoryViewProps) {
  const [rowCount, setRowCount] = useState(128);

  const rowsRendered = (stopIndex: number) => {
    // Every so often, double the scrollable row count.
    setRowCount(1 << (Math.log2(stopIndex + 100) + 1));
  };

  return <div className="flex flex-col" style={{ maxHeight: "100%" }}>
    <div className="w-full flex-none flex flex-row text-sm px-2 bg-(--muted)">
      <div className="text-(--muted-foreground) flex-none w-[50px]">Regs</div>
      <div className="text-(--muted-foreground) flex-none w-[80px]">Address</div>
      <div className="text-(--muted-foreground) flex-none w-[80px]">Hex</div>
      <div className="text-(--muted-foreground) flex-1">{props.mode === "Disassemble" ? "Disassembly" : "Memory"}</div>
    </div>
    <List
      className="mx-2"
      rowComponent={MemoryRow}
      onRowsRendered={(_, { stopIndex }) => rowsRendered(stopIndex)}
      rowCount={rowCount}
      rowHeight={16}
      overscanCount={20}
      rowProps={{ mode: props.mode }}>
    </List>
  </div>;
}
