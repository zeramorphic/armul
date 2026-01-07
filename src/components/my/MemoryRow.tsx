import { ReactNode, useContext } from "react";
import "./MemoryRow.css"
import { PrettyArgument, PrettyInstr, ShiftType } from "@/lib/serde-types";
import { ProcessorContext } from "@/lib/ProcessorContext";
import { RowComponentProps } from "react-window";
import { Badge } from "../ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { cn } from "@/lib/utils";
import { get_memory } from "@/lib/processor";
import { AppContext } from "@/lib/AppContext";

interface MemoryRowProps {
  mode: 'Disassemble' | 'Memory',
};

function renderPrettyInstr(pretty: PrettyInstr): ReactNode {
  return (<span>
    <span className="opcode">{pretty.opcode_prefix}</span>
    <span className="opcode-cond">{pretty.cond}</span>
    <span className="opcode-suffix">{pretty.opcode_suffix}</span>
    &nbsp;
    <span>
      {pretty.args.map((arg, index) => <span key={index}><span className="faint">{index ? ", " : ""}</span>{renderPrettyArgument(arg)}</span>)}
    </span>
  </span>)
}

function renderPrettyArgument(arg: PrettyArgument): ReactNode {
  switch (arg.type) {
    case 'Register':
      return <span className="faint">{arg.negative ? "-" : ""}<span className="register">{registerToString(arg.register)}</span>{arg.write_back ? "!" : ""}</span>;
    case 'Psr':
      return <span className="register">{arg.psr.toUpperCase()}{arg.flag ? "_flg" : ""}</span>;
    case 'Shift':
      var amount;
      switch (arg.shift_amount.type) {
        case 'Register':
          amount = <span className="register">{registerToString(arg.shift_amount.value)}</span>;
          break;
        case 'Constant':
          amount = <span className="addr">{arg.shift_amount.value}</span>;
          break;
      }
      return <span className="faint">{shiftTypeToString(arg.shift_type)} {arg.shift_type == 'RotateRightExtended' ? "" : amount}</span>;
    case 'Constant':
      switch (arg.style) {
        case 'Address':
          return <span>{renderAddress(arg.value, 'addr-faint', 'addr', true)}</span>;
        case 'UnsignedDecimal':
          return <span className="addr">{arg.value}</span>;
        case 'Unknown':
          return <span className="addr">{arg.value}</span>;
        default: return <span>unknown_constant</span>;
      }
    case 'RegisterSet':
      return <span className="faint">&#123;
        {registerRanges(arg.registers).map((range, index) =>
          <span key={index}>{index ? ", " : ""}<span className="register">{range}</span>{arg.caret ? "^" : ""}</span>)}
        &#125;</span>
    default: return <span>unknown_arg {JSON.stringify(arg)}</span>;
  }
}

export function registerRanges(registers: number[]): string[] {
  var ranges: { start: number, end: number }[] = [];
  for (var i = 0; i < registers.length; i++) {
    if (ranges.length > 0 && ranges[ranges.length - 1].end == registers[i] - 1) {
      ranges[ranges.length - 1].end = registers[i];
    } else {
      ranges.push({ start: registers[i], end: registers[i] });
    }
  }
  return ranges.map(({ start, end }) => start == end ? registerToString(start) : registerToString(start) + "-" + registerToString(end));
}

export function registerToString(register: number): string {
  switch (register) {
    case 13: return 'SP';
    case 14: return 'LR';
    case 15: return 'PC';
    default: return `R${register}`
  }
}

function shiftTypeToString(shiftType: ShiftType): string {
  switch (shiftType) {
    case 'LogicalLeft': return 'LSL';
    case 'LogicalRight': return 'LSR';
    case 'ArithmeticRight': return 'ASR';
    case 'RotateRight': return 'ROR';
    case 'RotateRightExtended': return 'RRX';
  }
}

export function renderAddress(address: number, faintClass?: string, boldClass?: string, zeroX?: boolean): ReactNode {
  const str = address.toString(16).toUpperCase();
  return <><span className={faintClass}>{zeroX ? '0x' : ''}{'0'.repeat(8 - str.length)}</span><span className={boldClass}>{str}</span></>;
}

export default function MemoryRow(props: RowComponentProps<MemoryRowProps>) {
  const processor = useContext(ProcessorContext);
  const dispatch = useContext(AppContext);
  const addr = props.index * 4;
  const info = get_memory(processor, addr, dispatch);

  var body: ReactNode = "";
  if (info) {
    switch (props.mode) {
      case 'Disassemble':
        body = info?.instr
          ? renderPrettyInstr(info?.instr)
          : (<span style={{ color: `var(--muted-foreground)` }}>???</span>);
        break;
      case 'Memory':
        body = renderNumber(info.value);
        break;
    }
  }

  const regs = [...Array(16).keys()].filter(ix => processor.registers.regs[ix] === addr);
  let badges;
  if (regs.length !== 0) {
    let className = "badge-reg";
    switch (regs[0]) {
      case 13: className = "badge-reg-sp"; break;
      case 14: className = "badge-reg-lr"; break;
      case 15: className = "badge-reg-pc"; break;
    }
    const content = registerToString(regs[0]) + (regs.length === 1 ? "" : "+");
    badges = <Badge className={`rounded-full ${className}`}>{content}</Badge>;
    if (regs.length > 1) {
      badges = <Tooltip>
        <TooltipTrigger asChild>
          {badges}
        </TooltipTrigger>
        <TooltipContent style={{ fontFamily: "var(--font-mono)" }}>
          {registerRanges(regs).join(", ")}
        </TooltipContent>
      </Tooltip>;
    }
  } else if (processor.info.previous_pc === addr && processor.info.previous_pc + 4 !== processor.registers.regs[15]) {
    badges = <Tooltip>
      <TooltipTrigger asChild>
        <Badge className="rounded-full badge-reg-prev-pc">Src</Badge>
      </TooltipTrigger>
      <TooltipContent>
        This marker shows where the program counter was immediately before it moved.
      </TooltipContent>
    </Tooltip>;
  }

  return <div className="flex flex-row MemoryRow" style={props.style}>
    <div className="flex-none w-[50px]">{badges}</div>
    <div className="text-(--muted-foreground) flex-none w-[80px]">{renderAddress(addr, "text-(--extremely-muted-foreground)")}</div>
    <div className={cn(props.mode === "Disassemble" ? "text-(--muted-foreground)" : "", "flex-none w-[80px]")}>{renderAddress(info?.value ?? 0, "text-(--extremely-muted-foreground)")}</div>
    <div className="flex-1">{body}</div>
  </div>;
}

export function renderNumber(value: number, bracketPositive?: boolean) {
  if (value >= ~(1 << 31)) {
    const start = "-" + (~value + 1);
    if (bracketPositive) {
      return <span>{start.padStart(11).replace(/ /g, "\u00A0")} <span className="faint">({value})</span></span>;
    } else {
      return <span>{start.padStart(11).replace(/ /g, "\u00A0")}</span>;
    }
  } else {
    return <span>{value.toString().padStart(11).replace(/ /g, "\u00A0")}</span>;
  }
}
