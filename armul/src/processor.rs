//! A model of the ARM7TDMI processor.

use crate::{
    instr::{DataOp, DataOperand, Instr, MsrSource, Psr, Register, Shift, ShiftAmount, ShiftType},
    memory::Memory,
    mode::Mode,
    registers::Registers,
};

#[derive(Debug, Default)]
pub struct Processor {
    registers: Registers,
    memory: Memory,
    state: ProcessorState,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorState {
    #[default]
    Running,
    Stopped,
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

    pub fn state(&self) -> ProcessorState {
        self.state
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
            return Ok(());
        }

        match instr {
            Instr::BranchExchange { operand } => {
                self.execute_branch_exchange(pc, operand, listener)
            }
            Instr::Branch { link, offset } => self.execute_branch(pc, link, offset, listener),
            Instr::Data {
                set_condition_codes,
                op,
                dest,
                op1,
                op2,
            } => {
                self.execute_data_processing(pc, set_condition_codes, op, dest, op1, op2, listener)
            }
            Instr::Mrs { .. } => todo!(),
            Instr::Msr { psr, source } => self.execute_msr(pc, psr, source, listener),
            Instr::Multiply { .. } => todo!(),
            Instr::MultiplyLong { .. } => todo!(),
            Instr::SingleTransfer { .. } => todo!(),
            Instr::BlockTransfer { .. } => todo!(),
            Instr::Swap { .. } => todo!(),
            Instr::SoftwareInterrupt { comment } => match comment {
                2 => {
                    // Halt the processor.
                    self.state = ProcessorState::Stopped;
                    Ok(())
                }
                _ => Err(ProcessorError::InvalidSwi),
            },
        }
    }

    #[inline]
    fn execute_branch_exchange(
        &mut self,
        pc: u32,
        operand: Register,
        listener: &mut impl ProcessorListener,
    ) -> ProcessorResult {
        listener.cycle(Cycle::Seq, 1, pc);
        let new_pc = self.registers.get(operand);
        if new_pc & 0b11 != 0 {
            // We don't emulate THUMB instructions.
            return Err(ProcessorError::UnalignedPc);
        }
        *self.registers.get_mut(Register::R15) = new_pc;
        listener.pipeline_flush(pc);
        Ok(())
    }

    #[inline]
    fn execute_branch(
        &mut self,
        pc: u32,
        link: bool,
        offset: i32,
        listener: &mut impl ProcessorListener,
    ) -> ProcessorResult {
        listener.cycle(Cycle::Seq, 1, pc);
        if link {
            // Write the address of the next instruction into R14 (LR).
            *self.registers.get_mut(Register::R14) =
                self.registers.get(Register::R15).wrapping_add(4);
        }
        let pc_reg = self.registers.get_mut(Register::R15);
        // Only add 4 bytes instead of the actual PC offset (8 bytes)
        // because we're about to auto-increment the PC anyway at the
        // end of this execution step.
        *pc_reg = pc_reg.wrapping_add(4).wrapping_add_signed(offset);
        listener.pipeline_flush(pc);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn execute_data_processing(
        &mut self,
        pc: u32,
        set_condition_codes: bool,
        op: DataOp,
        dest: Register,
        op1: Register,
        op2: DataOperand,
        listener: &mut impl ProcessorListener,
    ) -> ProcessorResult {
        listener.cycle(Cycle::Seq, 1, pc);
        let pc_offset = if op2.is_register_specified_shift() {
            listener.cycle(Cycle::Internal, 1, pc);
            12
        } else {
            8
        };
        let mut val1 = self.registers.get_pc_offset(op1, pc_offset);
        let (mut val2, barrel_carry) = self.evaluate_operand(op2, pc_offset)?;

        let carry_value = if self.registers.carry() { 1 } else { 0 };
        let mut carry = false;
        let result = match op {
            DataOp::And | DataOp::Tst => val1 & val2,
            DataOp::Eor | DataOp::Teq => val1 ^ val2,
            DataOp::Sub | DataOp::Cmp => {
                // We implement subtraction by using the fact that
                // a - b is the same as a + ~b + 1.
                // We reassign to val2 to get correct behaviour of flags.
                val2 = !val2;
                if let Some(v) = val1.checked_add(val2).and_then(|x| x.checked_add(1)) {
                    v
                } else {
                    carry = true;
                    val1.wrapping_add(val2).wrapping_add(1)
                }
            }
            DataOp::Rsb => {
                val1 = !val1;
                if let Some(v) = val2.checked_add(val1).and_then(|x| x.checked_add(1)) {
                    v
                } else {
                    carry = true;
                    val2.wrapping_add(val1).wrapping_add(1)
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
                if let Some(v) = val1
                    .checked_add(val2)
                    .and_then(|x| x.checked_add(carry_value))
                {
                    v
                } else {
                    carry = true;
                    val1.wrapping_add(val2).wrapping_add(carry_value)
                }
            }
            DataOp::Sbc => {
                // val1 - val2 + carry - 1
                // is the same as val1 + ~val2 + carry.
                val2 = !val2;
                if let Some(v) = val1
                    .checked_add(val2)
                    .and_then(|x| x.checked_add(carry_value))
                {
                    v
                } else {
                    carry = true;
                    val1.wrapping_add(val2).wrapping_add(carry_value)
                }
            }
            DataOp::Rsc => {
                val1 = !val1;
                if let Some(v) = val1
                    .checked_add(val2)
                    .and_then(|x| x.checked_add(carry_value))
                {
                    v
                } else {
                    carry = true;
                    val1.wrapping_add(val2).wrapping_add(carry_value)
                }
            }
            DataOp::Orr => val1 | val2,
            DataOp::Mov => val2,
            DataOp::Bic => val1 & !val2,
            DataOp::Mvn => !val2,
        };

        println!(
            "Data operation: {op} {op1}={val1} {op2}={val2} {carry} {result} (flags = {set_condition_codes})"
        );

        if set_condition_codes {
            if dest == Register::R15 {
                match self.registers.mode().and_then(|m| Psr::Spsr.physical(m)) {
                    Some(spsr) => *self.registers.cpsr_mut() = self.registers.get_physical(spsr),
                    None => return Err(ProcessorError::NoSpsr),
                }
            } else {
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
                        self.registers.set_overflow(
                            (val1 & (1 << 31) == val2 & (1 << 31))
                                && (val1 & (1 << 31) != result & (1 << 31)),
                        );
                        self.registers.set_carry(carry);
                        self.registers.set_zero(result == 0);
                        self.registers.set_negative(result & (1 << 31) != 0);
                    }
                }
            }
        }

        match op {
            DataOp::Tst | DataOp::Teq | DataOp::Cmp | DataOp::Cmn => {}
            _ => {
                *self.registers_mut().get_mut(dest) = if dest == Register::R15 {
                    // We need to decrement the PC by 4 bytes to
                    // take the auto-increment into account.
                    listener.pipeline_flush(pc);
                    result.wrapping_sub(4)
                } else {
                    result
                };
            }
        }

        Ok(())
    }

    #[inline]
    fn execute_msr(
        &mut self,
        pc: u32,
        psr: Psr,
        source: MsrSource,
        listener: &mut impl ProcessorListener,
    ) -> ProcessorResult {
        listener.cycle(Cycle::Seq, 1, pc);
        let mode = self.registers.mode().unwrap_or(Mode::Usr);
        match source {
            MsrSource::Register(register) => {
                let value = self.registers.get(register);
                let target = self
                    .registers
                    .get_physical_mut(psr.physical(mode).ok_or(ProcessorError::NoSpsr)?);
                if mode == Mode::Usr {
                    *target = (*target & 0x0FFFFFFF) | (value & 0xF0000000);
                } else {
                    *target = value;
                }
                Ok(())
            }
            MsrSource::RegisterFlags(register) => {
                let value = self.registers.get(register);
                let target = self
                    .registers
                    .get_physical_mut(psr.physical(mode).ok_or(ProcessorError::NoSpsr)?);
                *target = (*target & 0x0FFFFFFF) | (value & 0xF0000000);
                Ok(())
            }
            MsrSource::Flags(flags) => {
                let target = self
                    .registers
                    .get_physical_mut(psr.physical(mode).ok_or(ProcessorError::NoSpsr)?);
                *target = (*target & 0x0FFFFFFF) | (flags & 0xF0000000);
                Ok(())
            }
        }
    }

    /// Evaluate the given operand to a data processing instruction.
    /// The output is given together with a carry out bit from the barrel shifter.
    /// If no shift operation was needed, we return the current value of the
    /// carry flag in the CPSR.
    ///
    /// If the register was used to specify the shift amount, the PC will be
    /// 12 bytes ahead of the current instruction. Else it will be 8 bytes ahead.
    fn evaluate_operand(
        &self,
        operand: DataOperand,
        pc_offset: u32,
    ) -> Result<(u32, bool), ProcessorError> {
        match operand {
            DataOperand::Constant(c) => Ok(c.value()),
            DataOperand::Register(register, shift) => self.apply_shift(
                self.registers.get_pc_offset(register, pc_offset),
                shift,
                pc_offset,
            ),
        }
    }

    /// Perform the action of the barrel shifter.
    /// The result is a u32 output together with a carry out bit.
    /// The RRX (rotate right extended) shift type uses the C flag as a carry in.
    /// LSL #0 is a special case where the carry out bit is the same as the
    /// current C flag.
    fn apply_shift(
        &self,
        value: u32,
        shift: Shift,
        pc_offset: u32,
    ) -> Result<(u32, bool), ProcessorError> {
        let shift_amount = match shift.shift_amount {
            ShiftAmount::Constant(n) => n,
            ShiftAmount::Register(Register::R15) => return Err(ProcessorError::PcUsedInShift),
            ShiftAmount::Register(register) => {
                self.registers.get_pc_offset(register, pc_offset) as u8
            }
        };
        match (shift.shift_type, shift_amount) {
            (ShiftType::RotateRightExtended, _) => Ok((
                (value >> 1) + if self.registers.carry() { 1 << 31 } else { 0 },
                value & 0b1 != 0,
            )),
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
                    Ok((value, value & (1 << 31) != 0))
                } else {
                    Ok((value.rotate_right(n as u32), value & (1 << (n - 1)) != 0))
                }
            }
        }
    }
}

/// Provides instrumentation in a processor's behaviour.
pub trait ProcessorListener {
    /// A processor cycle (or several) were performed.
    /// For instrumentation purposes, we track the program counter
    /// at which the cycle took place.
    fn cycle(&mut self, cycle: Cycle, count: usize, pc: u32);
    /// Simulate a pipeline flush.
    /// This takes 1S + 1N cycles to recover.
    fn pipeline_flush(&mut self, pc: u32);
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
    /// The SPSR was accessed, but one was not present in the current mode.
    NoSpsr,
    /// An invalid software interrupt was issued.
    InvalidSwi,
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

        fn pipeline_flush(&mut self, _pc: u32) {
            self.n_cycles += 1;
            self.s_cycles += 1;
        }
    }
}
