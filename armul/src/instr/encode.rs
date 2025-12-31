use crate::{
    assemble::LineError,
    instr::{
        Cond, DataOperand, Instr, MsrSource, Psr, RotatedConstant, Shift, ShiftType,
        SpecialOperand, TransferKind, TransferOperand, TransferSize, TransferSizeSpecial,
    },
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
            Instr::Multiply {
                set_condition_codes,
                dest,
                op1,
                op2,
                addend,
            } => Ok((if set_condition_codes { 1 << 20 } else { 0 })
                | (dest as u32) << 16
                | addend
                    .map(|addend| (1 << 21) | (addend as u32) << 12)
                    .unwrap_or(0)
                | (op2 as u32) << 8
                | 0b1001 << 4
                | (op1 as u32)),
            Instr::MultiplyLong {
                set_condition_codes,
                signed,
                accumulate,
                dest_hi,
                dest_lo,
                op1,
                op2,
            } => Ok(1 << 23
                | (if signed { 1 << 22 } else { 0 })
                | (if accumulate { 1 << 21 } else { 0 })
                | (if set_condition_codes { 1 << 20 } else { 0 })
                | (dest_hi as u32) << 16
                | (dest_lo as u32) << 12
                | (op2 as u32) << 8
                | 0b1001 << 4
                | (op1 as u32)),
            Instr::SingleTransfer {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => Ok((1 << 26)
                | (if pre_index { 1 << 24 } else { 0 })
                | (if offset_positive { 1 << 23 } else { 0 })
                | (if size == TransferSize::Byte {
                    1 << 22
                } else {
                    0
                })
                | (if write_back && pre_index { 1 << 21 } else { 0 })
                | (match kind {
                    TransferKind::Store => 0,
                    TransferKind::Load => 1 << 20,
                })
                | (base_register as u32) << 16
                | (data_register as u32) << 12
                | Instr::encode_transfer_operand(offset)?),
            Instr::SingleTransferSpecial {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => Ok((if pre_index { 1 << 24 } else { 0 })
                | (if offset_positive { 1 << 23 } else { 0 })
                | (if write_back && pre_index { 1 << 21 } else { 0 })
                | (match kind {
                    TransferKind::Store => 0,
                    TransferKind::Load => 1 << 20,
                })
                | (base_register as u32) << 16
                | (data_register as u32) << 12
                | (match size {
                    TransferSizeSpecial::HalfWord => 0b1011_0000,
                    TransferSizeSpecial::SignExtendedByte => 0b1101_0000,
                    TransferSizeSpecial::SignExtendedHalfWord => 0b1111_0000,
                })
                | Instr::encode_special_operand(offset)),
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

    /// Encodes a transfer operand in bits 25 and 11..0.
    fn encode_transfer_operand(operand: TransferOperand) -> Result<u32, LineError> {
        match operand {
            TransferOperand::Constant(value) => {
                if value < 1 << 12 {
                    Ok(value as u32)
                } else {
                    Err(LineError::ImmediateOutOfRange(value as u32))
                }
            }
            TransferOperand::Register(register, shift) => {
                Ok(1 << 25 | register as u32 | Instr::encode_shift(shift)?)
            }
        }
    }

    /// Encodes a special operand in bits 22, 11..8, 3..0.
    fn encode_special_operand(operand: SpecialOperand) -> u32 {
        match operand {
            SpecialOperand::Constant(value) => {
                1 << 22 | ((value >> 4) as u32) << 8 | (value & 0xF) as u32
            }
            SpecialOperand::Register(register) => register as u32,
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
