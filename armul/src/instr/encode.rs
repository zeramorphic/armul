use crate::{
    assemble::LineError,
    instr::{Cond, DataOp, DataOperand, Instr, MsrSource, Psr, Register, Shift, ShiftType},
};

use super::ShiftAmount;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealStrategy {
    NoHealing,
    SimpleHealing,
    /// An advanced healing strategy that lets us use a dummy register.
    AdvancedHealing(Register),
}

/// This constant/shifted register can either be encoded as a 12-bit value or
/// is put into the healing register using a given sequence of instructions.
pub struct OperandEncoding {
    /// The value of bits 25 and 11..0.
    pub value: u32,
    /// The instructions to prepend to make this operand have the right behaviour.
    pub instrs: Vec<u32>,
}

impl Instr {
    /// Attempt to encode the given instruction as a 32-bit integer.
    /// If healing is enabled, we will try to fix out-of-range problems
    /// by adding extra instructions.
    pub fn encode(self, cond: Cond, heal: HealStrategy) -> Result<Vec<u32>, LineError> {
        self.encode_no_cond(heal).map(|mut xs| {
            for x in &mut xs {
                *x |= (cond as u32) << 28;
            }
            xs
        })
    }

    /// Encode an instruction into the bottom 28 bits of a 32-bit integer,
    /// or possibly a sequence of such integers if healing was desired.
    fn encode_no_cond(self, heal: HealStrategy) -> Result<Vec<u32>, LineError> {
        match self {
            Instr::BranchExchange { operand } => {
                Ok(vec![0b1_0010_1111_1111_1111_0001_0000 | operand as u32])
            }
            Instr::Branch { link, offset } => {
                // Check that the offset is in bounds.
                if offset % 4 != 0 {
                    Err(LineError::MisalignedBranchOffset)
                } else if !(-(1 << 24)..(1 << 24)).contains(&(offset >> 2)) {
                    Err(LineError::OffsetOutOfRange)
                } else {
                    Ok(vec![
                        0b101 << 25
                        | (if link { 1 << 24 } else { 0 })
                        // Truncate to 24 significant bits.
                        | (((offset / 4) as u32) << 8 >> 8),
                    ])
                }
            }
            Instr::Data {
                set_condition_codes: false,
                op: DataOp::Mov,
                dest,
                op1: _,
                op2: DataOperand::Constant(c),
            } => Ok(Instr::fill_register(c, dest)),
            Instr::Data {
                set_condition_codes,
                op,
                dest,
                op1,
                op2,
            } => {
                let mut operand = Instr::encode_data_operand(op2, heal)?;
                operand.instrs.insert(
                    0,
                    (op as u32) << 21
                        | (if set_condition_codes { 1 << 20 } else { 0 })
                        | (op1 as u32) << 16
                        | (dest as u32) << 12
                        | operand.value,
                );
                Ok(operand.instrs)
            }
            Instr::Mrs { psr, target } => todo!(),
            Instr::Msr { psr, source } => {
                let signature = 0b1_0010_1000_1111 << 12;
                let dest = match psr {
                    Psr::Cpsr => 0,
                    Psr::Spsr => 1 << 22,
                };
                let source = match source {
                    MsrSource::Register(register) => (1 << 16) | register as u32,
                    MsrSource::RegisterFlags(register) => register as u32,
                    // The flags are encoded in a simple way:
                    // the uper four bits are encoded as the lower four bits
                    // of the immediate value, which are ROR'd by 4 places.
                    MsrSource::Flags(flags) => (1 << 25) | (1 << 9) | (flags >> 28),
                };
                Ok(vec![signature | dest | source])
            }
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
            Instr::SoftwareInterrupt { comment } => Ok(vec![0b1111 << 24 | comment & 0x00FFFFFF]),
        }
    }

    /// Encodes a data operand in bits 25 and 11..0.
    fn encode_data_operand(
        operand: DataOperand,
        heal: HealStrategy,
    ) -> Result<OperandEncoding, LineError> {
        match operand {
            DataOperand::Constant(constant) => Ok(Instr::encode_constant(constant, heal)?),
            DataOperand::Register(register, shift) => Ok(OperandEncoding {
                value: register as u32 | Instr::encode_shift(shift)?,
                instrs: Vec::new(),
            }),
        }
    }

    /// Encodes a shift in bits 11..4.
    fn encode_shift(mut shift: Shift) -> Result<u32, LineError> {
        match shift.shift_amount {
            ShiftAmount::Constant(0) => {
                // Any shift by zero is encoded as LSL #0.
                // This is because the bit fields for (e.g.) LSR #0 are overloaded.
                Ok(0)
            }
            ShiftAmount::Constant(mut shift_amount) => {
                if shift.shift_type == ShiftType::RotateRightExtended {
                    shift.shift_type = ShiftType::RotateRight;
                    shift_amount = 0;
                } else if shift_amount == 32
                    && matches!(
                        shift.shift_type,
                        ShiftType::LogicalRight | ShiftType::ArithmeticRight
                    )
                {
                    shift_amount = 0;
                } else if shift_amount >= 32 {
                    return Err(LineError::ShiftOutOfRange);
                }
                Ok((shift_amount as u32) << 7 | (shift.shift_type as u32) << 5)
            }
            ShiftAmount::Register(register) => {
                if shift.shift_type == ShiftType::RotateRightExtended {
                    Err(LineError::InvalidShiftType)
                } else {
                    Ok((register as u32) << 8 | (shift.shift_type as u32) << 5 | (1 << 4))
                }
            }
        }
    }

    /// Attempts to encode a 32-bit value as a 12-bit value.
    /// This is accomplished by treating the lower 8 bits as an unsigned value,
    /// which is zero extended to 32 bits and then rotated right by twice the
    /// value in the upper 4 bits.
    pub fn encode_constant(value: u32, heal: HealStrategy) -> Result<OperandEncoding, LineError> {
        // The algorithm is very simple: attempt to rotate left by
        // all possible values (0, 2, ..., 30), and see if any of the
        // results fit into 8 bits.
        for shift in (0..16).map(|x| x * 2) {
            let immediate = value.rotate_left(shift);
            if immediate <= 0xFF {
                // The shift value is already doubled, so we left-shift by 7 not 8.
                return Ok(OperandEncoding {
                    value: immediate | (shift << 7) | (1 << 25),
                    instrs: Vec::new(),
                });
            }
        }
        if let HealStrategy::AdvancedHealing(reg) = heal {
            // Fix the out-of-range error using this dummy register.
            Ok(OperandEncoding {
                value: reg as u32,
                instrs: Instr::fill_register(value, reg),
            })
        } else {
            Err(LineError::ImmediateOutOfRange(value))
        }
    }

    /// Return instructions that fill the given register with the prescribed value,
    /// using all healing strategies.
    pub fn fill_register(value: u32, register: Register) -> Vec<u32> {
        println!("value = {value} = {value:X}");
        // Try a direct move strategy first as in encode_constant.
        for shift in (0..16).map(|x| x * 2) {
            let immediate = value.rotate_left(shift);
            if immediate <= 0xFF {
                // Move directly.
                return vec![
                    (1 << 25)
                        | (DataOp::Mov as u32) << 21
                        | (register as u32) << 12
                        | (shift << 7)
                        | immediate,
                ];
            }
        }

        // Try a negated move next.
        for shift in (0..16).map(|x| x * 2) {
            let immediate = (!value).rotate_left(shift);
            if immediate <= 0xFF {
                // Move directly.
                return vec![
                    (1 << 25)
                        | (DataOp::Mvn as u32) << 21
                        | (register as u32) << 12
                        | (shift << 7)
                        | immediate,
                ];
            }
        }

        // Slice off the lowest significant byte (or 7 bits if misaligned) and try again.
        let trailing_zeros = (value.trailing_zeros() / 2) * 2;
        let shift = trailing_zeros + 8;
        println!("trailing {trailing_zeros}, shift {shift}");
        let mut instrs = Instr::fill_register(value >> shift << shift, register);
        // Now do `orr Rd, Rd, (extra)` to fill the remaining bits.
        instrs.push(
            (1 << 25)
                | (DataOp::Orr as u32) << 21
                | (register as u32) << 16
                | (register as u32) << 12
                // We need the & 0b11111 for the case where trailing_zeros is 0.
                | ((16 - trailing_zeros / 2) & 0b1111) << 8
                | ((value & (0xFF << trailing_zeros)) >> trailing_zeros),
        );
        instrs
    }
}
