//! A model of the ARM7TDMI processor.

use crate::{
    instr::{Cond, DataOp, DataOperand, Instr, Register, Shift, ShiftAmount, ShiftType},
    memory::Memory,
    registers::Registers,
};

#[derive(Debug, Default)]
pub struct Processor {
    registers: Registers,
    memory: Memory,
}

impl Processor {
    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    pub fn poll(&mut self) -> ProcessorResult {
        todo!()
    }

    /// Immediately execute the instruction at the current program counter.
    pub fn try_execute(&mut self, listener: &mut impl ProcessorListener) -> ProcessorResult {
        let pc = self.registers.get(Register::R15);

        // Check that the program counter is aligned.
        if pc & 0b11 != 0 {
            return Err(ProcessorError::UnalignedPc);
        }

        let Some((cond, instr)) = Instr::decode(self.memory.get_word_aligned(pc)) else {
            return Err(ProcessorError::UnrecognisedInstruction);
        };

        // Check whether the condition code holds.
        if !self.registers.test_condition(cond) {
            // The condition did not hold.
            // According to page 10-19, unexecuted instructions
            // take one S-cycle.
            listener.cycle(Cycle::Seq, 1, pc);
        }

        match instr {
            Instr::BranchExchange { operand } => todo!(),
            Instr::Branch { link, offset } => todo!(),
            Instr::Data {
                set_condition_codes,
                op,
                dest,
                op1,
                op2,
            } => {
                let val1 = self.registers.get(op1);
                let (val2, barrel_carry) = self.evaluate_operand(op2)?;

                let mut carry = false;
                let result =
                    match op {
                        DataOp::And | DataOp::Tst => val1 & val2,
                        DataOp::Eor | DataOp::Teq => val1 ^ val2,
                        DataOp::Sub | DataOp::Cmp => {
                            if let Some(v) = val1.checked_sub(val2) {
                                v
                            } else {
                                carry = true;
                                val1.wrapping_sub(val2)
                            }
                        }
                        DataOp::Rsb => {
                            if let Some(v) = val2.checked_sub(val1) {
                                v
                            } else {
                                carry = true;
                                val2.wrapping_sub(val1)
                            }
                        }
                        DataOp::Add | DataOp::Cmn => {
                            if let Some(v) = val1.checked_add(val2) {
                                v
                            } else {
                                carry = true;
                                val1.wrapping_add(val2)
                            }
                        }
                        DataOp::Adc => {
                            if let Some(v) = val1.checked_add(val2).and_then(|x| x.checked_add(1)) {
                                v
                            } else {
                                carry = true;
                                val1.wrapping_add(val2).wrapping_add(1)
                            }
                        }
                        DataOp::Sbc => {
                            if let Some(v) = val1.checked_sub(val2).and_then(|x| {
                                x.checked_sub(if self.registers.carry() { 0 } else { 1 })
                            }) {
                                v
                            } else {
                                carry = true;
                                val1.wrapping_add(val2)
                                    .wrapping_add(1)
                                    .wrapping_sub(if self.registers.carry() { 0 } else { 1 })
                            }
                        }
                        DataOp::Rsc => {
                            if let Some(v) = val2.checked_sub(val1).and_then(|x| {
                                x.checked_sub(if self.registers.carry() { 0 } else { 1 })
                            }) {
                                v
                            } else {
                                carry = true;
                                val2.wrapping_add(val1)
                                    .wrapping_add(1)
                                    .wrapping_sub(if self.registers.carry() { 0 } else { 1 })
                            }
                        }
                        DataOp::Orr => val1 | val2,
                        DataOp::Mov => val2,
                        DataOp::Bic => val1 & !val2,
                        DataOp::Mvn => !val2,
                    };

                if set_condition_codes {
                    match op {
                        DataOp::And
                        | DataOp::Eor
                        | DataOp::Tst
                        | DataOp::Teq
                        | DataOp::Orr
                        | DataOp::Mov
                        | DataOp::Bic
                        | DataOp::Mvn => {
                            // This is a logical operation.
                            self.registers.set_carry(barrel_carry);
                            self.registers.set_zero(result == 0);
                            self.registers.set_negative(result & (1 << 31) != 0);
                        }
                        DataOp::Sub
                        | DataOp::Rsb
                        | DataOp::Add
                        | DataOp::Adc
                        | DataOp::Sbc
                        | DataOp::Rsc
                        | DataOp::Cmp
                        | DataOp::Cmn => {
                            // This is an arithmetic operation.
                            // TODO: How exactly does the overflow flag work?
                            todo!()
                        }
                    }
                }

                todo!()
            }
            Instr::Mrs { psr, target } => todo!(),
            Instr::Msr { psr, source } => todo!(),
            Instr::Multiply {
                set_condition_codes,
                dest,
                op1,
                op2,
                addend,
            } => todo!(),
            Instr::MultiplyLong {
                set_condition_codes,
                signed,
                accumulate,
                dest_hi,
                dest_lo,
                op1,
                op2,
            } => todo!(),
            Instr::SingleTransfer {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => todo!(),
            Instr::BlockTransfer {
                kind,
                write_back,
                offset_positive,
                pre_index,
                psr,
                base_register,
                registers,
            } => todo!(),
            Instr::Swap {
                byte,
                dest,
                source,
                base,
            } => todo!(),
            Instr::SoftwareInterrupt { comment } => todo!(),
        }
    }

    /// Evaluate the given operand to a data processing instruction.
    /// The output is given together with a carry out bit from the barrel shifter.
    /// If no shift operation was needed, we return the current value of the
    /// carry flag in the CPSR.
    fn evaluate_operand(&self, operand: DataOperand) -> Result<(u32, bool), ProcessorError> {
        match operand {
            DataOperand::Constant(c) => Ok((c, self.registers.carry())),
            DataOperand::Register(register, shift) => {
                self.apply_shift(self.registers.get(register), shift)
            }
        }
    }

    /// Perform the action of the barrel shifter.
    /// The result is a u32 output together with a carry out bit.
    /// The RRX (rotate right extended) shift type uses the C flag as a carry in.
    /// LSL #0 is a special case where the carry out bit is the same as the
    /// current C flag.
    fn apply_shift(&self, value: u32, shift: Shift) -> Result<(u32, bool), ProcessorError> {
        let shift_amount = match shift.shift_amount {
            ShiftAmount::Constant(n) => n,
            ShiftAmount::Register(Register::R15) => return Err(ProcessorError::PcUsedInShift),
            ShiftAmount::Register(register) => self.registers.get(register) as u8,
        };
        match (shift.shift_type, shift_amount) {
            (_, 0) => {
                // Note that special encodings such as LSR #0 have already been
                // decoded into their expanded forms.
                Ok((value, self.registers.carry()))
            }
            (ShiftType::LogicalLeft, 1..32) => Ok((
                value << shift_amount,
                value & (1 << (32 - shift_amount)) != 0,
            )),
            (ShiftType::LogicalLeft, 32) => Ok((0, value & 0b1 != 0)),
            (ShiftType::LogicalLeft, 33..) => Ok((0, false)),
            (ShiftType::LogicalRight, 1..32) => Ok((
                value >> shift_amount,
                value & (1 << (shift_amount - 1)) != 0,
            )),
            (ShiftType::LogicalRight, 32) => Ok((0, value & (1 << 31) != 0)),
            (ShiftType::LogicalRight, 33..) => Ok((0, false)),
            (ShiftType::ArithmeticRight, 1..32) => Ok((
                ((value as i32) >> shift_amount) as u32,
                value & (1 << (shift_amount - 1)) != 0,
            )),
            (ShiftType::ArithmeticRight, 32..) => {
                if value & (1 << 31) == 0 {
                    Ok((0, false))
                } else {
                    Ok((0xFFFFFFFF, true))
                }
            }
            (ShiftType::RotateRight, n) => {
                let n = (n - 1) % 32 + 1;
                // n is now in the range 1..=32.
                if n == 32 {
                    Ok((value.rotate_right(n as u32), value & (1 << (n - 1)) != 0))
                } else {
                    Ok((value, value & (1 << 31) != 0))
                }
            }
            (ShiftType::RotateRightExtended, _) => Ok((
                (value >> 1) + if self.registers.carry() { 1 << 31 } else { 0 },
                value & 0b1 != 0,
            )),
        }
    }
}

/// Provides instrumentation in a processor's behaviour.
pub trait ProcessorListener {
    /// A processor cycle (or several) were performed.
    /// For instrumentation purposes, we track the program counter
    /// at which the cycle took place.
    fn cycle(&mut self, cycle: Cycle, count: usize, pc: u32);
}

/// One of the four cycle types in the CPU.
pub enum Cycle {
    /// The processor accessed a portion of memory unrelated to the address
    /// used in the preceding cycle.
    /// We assume that these take roughly 2.5x the time of the other cycle types
    /// for the purposes of benchmarking.
    NonSeq,
    /// The processor accessed a memory location at the same address to last
    /// cycle, or a halfword or word afterwards.
    Seq,
    /// An internal cycle which does not require a memory transfer.
    Internal,
    /// The processor uses the data bus to communicate with a coprocessor.
    /// As coprocessor operations are not implemented by our emulator,
    /// these cycles will never occur.
    Coprocessor,
}

pub type ProcessorResult = Result<(), ProcessorError>;

/// The type of possible errors that can be encountered
/// while executing an instruction.
#[derive(Debug)]
pub enum ProcessorError {
    /// The program counter was not 4-byte aligned.
    UnalignedPc,
    /// The instruction at the program counter could not be decoded.
    UnrecognisedInstruction,
    /// The program counter register (PC, or R15) was used in a register
    /// specified shift amount.
    PcUsedInShift,
}

#[cfg(test)]
pub mod test {
    use crate::processor::Cycle;
    use crate::processor::ProcessorListener;

    #[derive(Default, Debug)]
    pub struct TestProcessorListener {
        n_cycles: usize,
        s_cycles: usize,
        i_cycles: usize,
    }

    impl ProcessorListener for TestProcessorListener {
        fn cycle(&mut self, cycle: Cycle, count: usize, _pc: u32) {
            match cycle {
                Cycle::NonSeq => self.n_cycles += count,
                Cycle::Seq => self.s_cycles += count,
                Cycle::Internal => self.i_cycles += count,
                Cycle::Coprocessor => {}
            }
        }
    }
}
