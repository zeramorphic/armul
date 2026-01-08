import { registerToString, renderNumber } from './MemoryRow';
import { useContext } from 'react';
import { ProcessorContext } from '@/lib/ProcessorContext';

export default function Registers() {
  const registers = useContext(ProcessorContext).registers;

  const cpsr = registers.regs[31];
  var flags = [];
  for (const { bit, letter } of [{ bit: 31, letter: 'N' }, { bit: 30, letter: 'Z' }, { bit: 29, letter: 'C' }, { bit: 28, letter: 'V' }]) {
    flags.push(<span key={bit} className={(cpsr & (1 << bit)) > 0 ? "pr-1" : "text-(--extremely-muted-foreground) pr-1"}>{letter}</span>);
  }
  flags.push(<span className="mx-2"></span>);
  for (const { bit, letter } of [{ bit: 7, letter: 'I' }, { bit: 6, letter: 'F' }, { bit: 5, letter: 'T' }]) {
    flags.push(<span key={bit} className={(cpsr & (1 << bit)) > 0 ? "pr-1" : "text-(--extremely-muted-foreground) pr-1"}>{letter}</span>);
  }

  var mode = "??? mode";
  switch (cpsr & 0b11111) {
    case 0b10000: mode = "USR mode"; break;
    case 0b10001: mode = "FIQ mode"; break;
    case 0b10010: mode = "IRQ mode"; break;
    case 0b10011: mode = "SVC mode"; break;
    case 0b10111: mode = "ABT mode"; break;
    case 0b11011: mode = "UND mode"; break;
    case 0b11111: mode = "SYS mode"; break;
  }

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
    <div className="h-2"></div>
    <div className="px-2 flex">
      <div className="font-mono">{flags}</div>
      <div className="flex-1"></div>
      <div className="text-sm font-mono flex items-center">{mode}</div>
    </div>
  </div>;
}
