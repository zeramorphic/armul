import { ReactNode, useContext } from "react";
import "./MemoryRow.css"
import { PrettyArgument, PrettyInstr, ShiftType } from "@/lib/serde-types";
import { ProcessorContext } from "@/lib/ProcessorContext";
import { RowComponentProps } from "react-window";

interface MemoryRowProps {
  mode: 'Disassemble' | 'Memory',
};

function Skip() {
  return (<span style={{ paddingRight: `10pt` }}></span>);
}

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
          return <span>{renderAddress(arg.value)}</span>;
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

function registerRanges(registers: number[]): string[] {
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

function registerToString(register: number): string {
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

function renderAddress(address: number): ReactNode {
  const str = address.toString(16).toUpperCase();
  return <><span className="addr-faint">0x{'0'.repeat(8 - str.length)}</span><span className="addr">{str}</span></>;
}

export default function MemoryRow(props: RowComponentProps<MemoryRowProps>) {
  const processor = useContext(ProcessorContext);
  const addr = props.index * 4;
  const info = processor.get_memory(addr);

  const hex = info
    ? info.value.toString(16).toUpperCase().padStart(8, "0")
    : "";

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

  return (
    <p className="MemoryRow" style={props.style}>
      <span style={{
        color: `var(--very-muted-foreground)`
      }}>
        {addr.toString(16).toUpperCase().padStart(8, "0")}
      </span>
      <Skip />
      <span style={{
        color: `var(--muted-foreground)`
      }}>
        {hex}
      </span>
      <Skip />
      <span>{body}</span>
    </p>
  )
}

export function renderNumber(value: number) {
  if (value >= ~(1 << 31)) {
    const start = "-" + (~value + 1);
    return <span>{start.padStart(11).replace(/ /g, "\u00A0")} <span className="faint">({value})</span></span>;
  } else {
    return <span>{value.toString().padStart(11).replace(/ /g, "\u00A0")}</span>;
  }
}
