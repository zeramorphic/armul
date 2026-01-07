import './Status.css';
import { renderNumber } from './MemoryRow';
import { ButtonGroup } from '../ui/button-group';
import { Button } from '../ui/button';
import { PlayIcon, RefreshCwIcon, StepForwardIcon } from 'lucide-react';
import { useContext } from 'react';
import { ProcessorContext } from '@/lib/ProcessorContext';
import Processor from '@/lib/processor';
import { invoke } from '@tauri-apps/api/core';
import { useHotkeys } from 'react-hotkeys-hook';

interface StatusProps {
};

function Vspace() {
  return <div style={{ paddingBottom: `5px` }}></div>;
}

async function stepOnce(processor: Processor) {
  await invoke('step_once');
  await processor.resynchronise();
}

export default function Status(props: StatusProps) {
  const processor = useContext(ProcessorContext);

  useHotkeys('f2', () => stepOnce(processor));

  const transport = <ButtonGroup>
    <Button variant="outline"><PlayIcon /></Button>
    <Button variant="outline" onClick={() => stepOnce(processor)}><StepForwardIcon /></Button>
    <Button variant="outline"><RefreshCwIcon /></Button>
  </ButtonGroup>;

  const registers = useContext(ProcessorContext).registers;
  const regs = <table className="registers">
    <tbody>
      <tr>
        <td className="regName">R0</td>
        <td className="regContents">{renderNumber(registers.regs[0])}</td>
      </tr>
      <tr>
        <td className="regName">R1</td>
        <td className="regContents">{renderNumber(registers.regs[1])}</td>
      </tr>
      <tr>
        <td className="regName">R2</td>
        <td className="regContents">{renderNumber(registers.regs[2])}</td>
      </tr>
      <tr>
        <td className="regName">R3</td>
        <td className="regContents">{renderNumber(registers.regs[3])}</td>
      </tr>
      <tr>
        <td className="regName">R4</td>
        <td className="regContents">{renderNumber(registers.regs[4])}</td>
      </tr>
      <tr>
        <td className="regName">R5</td>
        <td className="regContents">{renderNumber(registers.regs[5])}</td>
      </tr>
      <tr>
        <td className="regName">R6</td>
        <td className="regContents">{renderNumber(registers.regs[6])}</td>
      </tr>
      <tr>
        <td className="regName">R7</td>
        <td className="regContents">{renderNumber(registers.regs[7])}</td>
      </tr>
      <tr>
        <td className="regName">R8</td>
        <td className="regContents">{renderNumber(registers.regs[8])}</td>
      </tr>
      <tr>
        <td className="regName">R9</td>
        <td className="regContents">{renderNumber(registers.regs[9])}</td>
      </tr>
      <tr>
        <td className="regName">R10</td>
        <td className="regContents">{renderNumber(registers.regs[10])}</td>
      </tr>
      <tr>
        <td className="regName">R11</td>
        <td className="regContents">{renderNumber(registers.regs[11])}</td>
      </tr>
      <tr>
        <td className="regName">R12</td>
        <td className="regContents">{renderNumber(registers.regs[12])}</td>
      </tr>
      <tr>
        <td className="regName">SP</td>
        <td className="regContents">{renderNumber(registers.regs[13])}</td>
      </tr>
      <tr>
        <td className="regName">LR</td>
        <td className="regContents">{renderNumber(registers.regs[14])}</td>
      </tr>
      <tr>
        <td className="regName">PC</td>
        <td className="regContents">{renderNumber(registers.regs[15])}</td>
      </tr>
    </tbody>
  </table>;

  const cpsr = <span>Flags: {registers.regs[31].toString(16).toUpperCase().padStart(8, '0')}</span>;

  return <div className="status">
    {transport}
    <Vspace />
    {regs}
    <Vspace />
    {cpsr}
  </div>;
}
