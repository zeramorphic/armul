import { registerToString, renderNumber } from './MemoryRow';
import { useContext } from 'react';
import { ProcessorContext } from '@/lib/ProcessorContext';
import { CheckIcon, XIcon } from 'lucide-react';

function condName(cond: number): string | undefined {
  switch (cond) {
    case 0: return "EQ";
    case 1: return "NE";
    case 2: return "CS";
    case 3: return "CC";
    case 4: return "MI";
    case 5: return "PL";
    case 6: return "VS";
    case 7: return "VC";
    case 8: return "HI";
    case 9: return "LS";
    case 10: return "GE";
    case 11: return "LT";
    case 12: return "GT";
    case 13: return "LE";
  }
}

function condSatisfied(cpsr: number, cond: string): boolean {
  const n = (cpsr & (1 << 31)) !== 0;
  const z = (cpsr & (1 << 30)) !== 0;
  const c = (cpsr & (1 << 29)) !== 0;
  const v = (cpsr & (1 << 28)) !== 0;

  console.log(n, z, c, v)

  switch (cond) {
    case "EQ": return z;
    case "NE": return !z;
    case "CS": return c;
    case "CC": return !c;
    case "MI": return n;
    case "PL": return !n;
    case "VS": return v;
    case "VC": return !v;
    case "HI": return c && !z;
    case "LS": return !c || z;
    case "GE": return n == v;
    case "LT": return n != v;
    case "GT": return !z && (n == v);
    case "LE": return z || (n != v);
    default: return false;
  }
}

export default function Registers() {
  const processor = useContext(ProcessorContext);
  const registers = processor.registers;

  const cond = processor.info.current_cond;
  const condStr = condName(cond);

  const cpsr = registers.regs[31];
  var flags = [];
  for (const { bit, letter } of [{ bit: 31, letter: 'N' }, { bit: 30, letter: 'Z' }, { bit: 29, letter: 'C' }, { bit: 28, letter: 'V' }]) {
    flags.push(<span key={bit} className={(cpsr & (1 << bit)) !== 0 ? "pr-1" : "text-(--extremely-muted-foreground) pr-1"}>{letter}</span>);
  }
  flags.push(<span className="mx-2"></span>);
  for (const { bit, letter } of [{ bit: 7, letter: 'I' }, { bit: 6, letter: 'F' }, { bit: 5, letter: 'T' }]) {
    flags.push(<span key={bit} className={(cpsr & (1 << bit)) !== 0 ? "pr-1" : "text-(--extremely-muted-foreground) pr-1"}>{letter}</span>);
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
    <div className="px-2 flex">
      <div className="font-mono mr-2 pt-[1px]">{condStr}</div>
      {condStr
        ? condSatisfied(cpsr, condStr) ? <CheckIcon /> : <XIcon />
        : <></>}
    </div>
  </div>;
}
