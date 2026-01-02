import React from 'react';
import './Status.css';
import { invoke } from '@tauri-apps/api/core';
import { renderNumber } from './MemoryRow';

interface StatusProps {
  generation: number,
};

interface Registers {
  regs: number[],
}

function Vspace() {
  return <div style={{ paddingBottom: `5px` }}></div>;
}

export default function Status(props: StatusProps) {
  const [generation, setGeneration] = React.useState(props.generation);
  const [registers, setRegisters] = React.useState<Registers>({ regs: Array(37).fill(0) });

  if (generation !== props.generation) {
    setGeneration(props.generation);
  }

  React.useEffect(() => {
    (async () => {
      console.log("Gen bump!");
      // Race conditions don't matter here because as soon as the data updates,
      // we get given a new generation.
      setRegisters(await invoke('registers'));
    })();
  }, [generation]);

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
    {regs}
    <Vspace />
    {cpsr}
  </div>;
}
