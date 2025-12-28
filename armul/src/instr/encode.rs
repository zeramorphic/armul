use crate::{
    assemble::LineError,
    instr::{Cond, DataOperand, Instr, MsrSource, Psr, RotatedConstant, Shift, ShiftType},
};

use super::ShiftAmount;

impl Instr {
    /// Encode the given instruction as a 32-bit integer.
    pub fn encode(self, cond: Cond) -> Result<u32, LineError> {
        self.encode_no_cond().map(|x| x | (cond as u32) << 28)
    }

    /// Encode an instruction into the bottom 28 bits of a 32-bit integer.
    fn encode_no_cond(self) -> Result<u32, LineError> {
        match self {
            Instr::BranchExchange { operand } => {
                Ok(0b1_0010_1111_1111_1111_0001_0000 | operand as u32)
            }
            Instr::Branch { link, offset } => {
                // Check that the offset is in bounds.
                if offset % 4 != 0 {
                    Err(LineError::MisalignedBranchOffset)
                } else if !(-(1 << 24)..(1 << 24)).contains(&(offset >> 2)) {
                    Err(LineError::OffsetOutOfRange)
                } else {
                    Ok(0b101 << 25
                        | (if link { 1 << 24 } else { 0 })
                        // Truncate to 24 significant bits.
                        | (((offset / 4) as u32) << 8 >> 8))
                }
            }
            Instr::Data {
                set_condition_codes,
                op,
                dest,
                op1,
                op2,
            } => Ok((op as u32) << 21
                | (if set_condition_codes { 1 << 20 } else { 0 })
                | (op1 as u32) << 16
                | (dest as u32) << 12
                | Instr::encode_data_operand(op2)?),
            Instr::Mrs { psr, target } => Ok(0b100001111 << 16
                | match psr {
                    Psr::Cpsr => 0,
                    Psr::Spsr => 1 << 22,
                }
                | (target as u32) << 12),
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
                Ok(signature | dest | source)
            }
            Instr::Multiply { .. } => todo!(),
            Instr::MultiplyLong { .. } => todo!(),
            Instr::SingleTransfer { .. } => todo!(),
            Instr::BlockTransfer { .. } => todo!(),
            Instr::Swap { .. } => todo!(),
            Instr::SoftwareInterrupt { comment } => Ok(0b1111 << 24 | comment & 0x00FFFFFF),
        }
    }

    /// Encodes a data operand in bits 25 and 11..0.
    fn encode_data_operand(operand: DataOperand) -> Result<u32, LineError> {
        match operand {
            DataOperand::Constant(constant) => Ok(Instr::encode_constant(constant)),
            DataOperand::Register(register, shift) => {
                Ok(register as u32 | Instr::encode_shift(shift)?)
            }
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

    fn encode_constant(value: RotatedConstant) -> u32 {
        (value.immediate as u32) | ((value.half_rotate as u32) << 8) | (1 << 25)
    }
}
