use num_traits::FromPrimitive;

use crate::instr::{
    Cond, DataOp, DataOperand, Instr, MsrSource, Psr, Register, Shift, ShiftAmount, ShiftType,
    TransferKind, TransferSize,
};

impl Instr {
    /// Attempt to decode the given 32-bit value as an instruction.
    /// If this instruction could not be decoded, return `None`.
    pub fn decode(instr: u32) -> Option<(Cond, Instr)> {
        // On condition 0b1111, return `None`.
        let cond = Cond::from_u32(instr >> 28)?;

        // Mask off the condition.
        let instr = instr & ((1 << 28) - 1);

        Instr::decode_no_cond(instr).map(|i| (cond, i))
    }

    /// Perform a decode, assuming that the top four bits are masked out.
    fn decode_no_cond(instr: u32) -> Option<Instr> {
        // First, test for the BX instruction since its pattern is very specific
        // and overlaps with other tests we'll do later.
        if instr >> 4 == 0b0001_0010_1111_1111_1111_0001 {
            return Some(Instr::BranchExchange {
                operand: Register::from_u4(instr, 0),
            });
        }

        // Test the first three bits of the instruction to determine its type.
        match instr >> 25 {
            0b000 | 0b001 => {
                // This is a data processing instruction or misc instruction.
                // To check which kind it is, we make use of the fact that
                // if bit 25 is set in a data processing instruction,
                // we're doing a shift, and therefore
                // either bit 4 is unset or bit 7 is unset.
                // Since bits 4 and 7 are both set for multiply/swap instructions,
                // this allows us to disambiguate the two possibilities.
                if instr & (1 << 25 | 1 << 7 | 1 << 4) == 1 << 7 | 1 << 4 {
                    // This is a non-data-processing instruction.
                    if instr & 0b110_0000 == 0 {
                        // This is multiply, multiply long, or single data swap.
                        if instr & (1 << 23) != 0 {
                            // This is multiply long.
                            Some(Instr::MultiplyLong {
                                set_condition_codes: instr & (1 << 20) != 0,
                                signed: instr & (1 << 22) != 0,
                                accumulate: instr & (1 << 21) != 0,
                                dest_hi: Register::from_u4(instr, 16),
                                dest_lo: Register::from_u4(instr, 12),
                                op1: Register::from_u4(instr, 0),
                                op2: Register::from_u4(instr, 8),
                            })
                        } else if instr & (1 << 24) != 0 {
                            // This is single data swap.
                            Some(Instr::Swap {
                                byte: instr & (1 << 22) != 0,
                                dest: Register::from_u4(instr, 12),
                                source: Register::from_u4(instr, 0),
                                base: Register::from_u4(instr, 16),
                            })
                        } else {
                            // This is multiply.
                            Some(Instr::Multiply {
                                set_condition_codes: instr & (1 << 20) != 0,
                                dest: Register::from_u4(instr, 16),
                                op1: Register::from_u4(instr, 0),
                                op2: Register::from_u4(instr, 8),
                                addend: if instr & (1 << 21) == 0 {
                                    Some(Register::from_u4(instr, 12))
                                } else {
                                    None
                                },
                            })
                        }
                    } else {
                        // This is halfword data transfer.
                        // Note that SH can never be 00.
                        Some(Instr::SingleTransfer {
                            kind: if instr & (1 << 20) == 0 {
                                TransferKind::Store
                            } else {
                                TransferKind::Load
                            },
                            size: if instr & (1 << 6) == 0 {
                                TransferSize::HalfWord
                            } else if instr & (1 << 5) == 0 {
                                TransferSize::SignExtendedByte
                            } else {
                                TransferSize::SignExtendedHalfWord
                            },
                            write_back: instr & (1 << 21) != 0,
                            offset_positive: instr & (1 << 23) != 0,
                            pre_index: instr & (1 << 24) != 0,
                            data_register: Register::from_u4(instr, 12),
                            base_register: Register::from_u4(instr, 16),
                            offset: DataOperand::Register(
                                Register::from_u4(instr, 0),
                                Shift {
                                    shift_type: ShiftType::LogicalLeft,
                                    shift_amount: ShiftAmount::Constant(0),
                                },
                            ),
                        })
                    }
                } else {
                    // This is a data-processing or PSR transfer instruction.

                    // Note that the MSR/MRS instructions would otherwise
                    // be interpreted as `TEQ/TST/CMP/CMN` instructions with
                    // the `S` bit unset, but these instructions would be
                    // pointless so the space is reused for PSR instructions.

                    // Some extra unnecessary bits are not checked.

                    if instr & (0b1_1011_1111 << 16) == 0b1_0000_1111 << 16 {
                        // This is an MRS instruction.
                        Some(Instr::Mrs {
                            psr: if instr & (1 << 22) == 0 {
                                Psr::Cpsr
                            } else {
                                Psr::Spsr
                            },
                            target: Register::from_u4(instr, 12),
                        })
                    } else if instr & (0b1_1011_1111_1111 << 12) == 0b1_0010_1000_1111 << 12 {
                        // This is an MSR flag instruction.
                        Some(Instr::Msr {
                            psr: if instr & (1 << 22) == 0 {
                                Psr::Cpsr
                            } else {
                                Psr::Spsr
                            },
                            source: if instr & (1 << 25) == 0 {
                                // The source operand is a register.
                                MsrSource::RegisterFlags(Register::from_u4(instr, 0))
                            } else {
                                // The source operand is an immediate value.
                                let imm = instr & 0xFF;
                                let rotate = (instr >> 8) & 0xF;
                                MsrSource::Flags(imm.rotate_right(rotate * 2))
                            },
                        })
                    } else if instr & (0b1_1011_0000_1111 << 12) == 0b1_0010_0000_1111 << 12 {
                        // This is an MSR register instruction.
                        // Note that we don't check bits 16..13 because
                        // the docs [here](https://mgba-emu.github.io/gbatek/#armopcodespsrtransfermrsmsr)
                        // say those bits are variable.
                        Some(Instr::Msr {
                            psr: if instr & (1 << 22) == 0 {
                                Psr::Cpsr
                            } else {
                                Psr::Spsr
                            },
                            source: MsrSource::Register(Register::from_u4(instr, 0)),
                        })
                    } else {
                        // This is a data instruction.
                        let op2 = if instr & (1 << 25) == 0 {
                            // Shifted register operand.
                            Instr::decode_shifted_register(instr)
                        } else {
                            // Immediate operand.
                            let imm = instr & 0xFF;
                            let rotate = (instr >> 8) & 0xF;
                            DataOperand::Constant(imm.rotate_right(rotate * 2))
                        };
                        Some(Instr::Data {
                            set_condition_codes: instr & (1 << 20) != 0,
                            op: DataOp::from_u32((instr >> 21) & 0b1111).unwrap(),
                            dest: Register::from_u4(instr, 12),
                            op1: Register::from_u4(instr, 16),
                            op2,
                        })
                    }
                }
            }
            0b010 | 0b011 => {
                // This is a word/byte single data transfer instruction.
                Some(Instr::SingleTransfer {
                    kind: if instr & (1 << 20) == 0 {
                        TransferKind::Store
                    } else {
                        TransferKind::Load
                    },
                    size: if instr & (1 << 22) == 0 {
                        TransferSize::Word
                    } else {
                        TransferSize::Byte
                    },
                    write_back: instr & (1 << 21) != 0,
                    offset_positive: instr & (1 << 23) != 0,
                    pre_index: instr & (1 << 24) != 0,
                    data_register: Register::from_u4(instr, 12),
                    base_register: Register::from_u4(instr, 16),
                    offset: if instr & (1 << 25) == 0 {
                        // The offset is an immediate value.
                        DataOperand::Constant(instr & 0xFFF)
                    } else {
                        // The offset is a shifted register.
                        Instr::decode_shifted_register(instr)
                    },
                })
            }
            0b100 => {
                // This is a block data transfer instruction.
                Some(Instr::BlockTransfer {
                    kind: if instr & (1 << 20) == 0 {
                        TransferKind::Store
                    } else {
                        TransferKind::Load
                    },
                    write_back: instr & (1 << 21) != 0,
                    offset_positive: instr & (1 << 23) != 0,
                    pre_index: instr & (1 << 24) != 0,
                    psr: instr & (1 << 22) != 0,
                    base_register: Register::from_u4(instr, 16),
                    registers: instr as u16,
                })
            }
            0b101 => {
                // This is a branch instruction.
                let base_offset = (instr & ((1 << 24) - 1)) << 2;
                // Sign-extend the shifted offset to 32 bits.
                let offset = if instr & (1 << 23) == 0 {
                    base_offset as i32
                } else {
                    (base_offset | !((1 << 26) - 1)) as i32
                };
                Some(Instr::Branch {
                    link: instr & (1 << 24) != 0,
                    offset,
                })
            }
            0b111 if instr & (1 << 25) != 0 => {
                // This is a software interrupt.
                let comment = instr & ((1 << 24) - 1);
                Some(Instr::SoftwareInterrupt { comment })
            }
            _ => {
                // This is a coprocessor instruction, which is unsupported.
                None
            }
        }
    }

    /// Decode the shift register data in bits 11..0.
    fn decode_shifted_register(instr: u32) -> DataOperand {
        let register = Register::from_u4(instr, 0);
        let mut shift_type = ShiftType::from_u32((instr >> 5) & 0b11).unwrap();
        if instr & (1 << 4) == 0 {
            // Shift by a constant.
            let mut shift_amount = (instr >> 7) & 0b11111;
            if shift_amount == 0 {
                match shift_type {
                    ShiftType::LogicalRight | ShiftType::ArithmeticRight => shift_amount = 32,
                    ShiftType::RotateRight => shift_type = ShiftType::RotateRightExtended,
                    _ => {}
                }
            }
            DataOperand::Register(
                register,
                Shift {
                    shift_type,
                    shift_amount: ShiftAmount::Constant(shift_amount as u8),
                },
            )
        } else {
            // Shift by a register.
            // Bit 7 is unset.
            let shift_by = Register::from_u4(instr, 8);
            DataOperand::Register(
                register,
                Shift {
                    shift_type,
                    shift_amount: ShiftAmount::Register(shift_by),
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instr::Instr;

    #[test]
    fn test() {
        let instrs = [
            0xEAFFFFFE, 0xEA000004, 0xE3510000, 0x0A000002, 0xEB000008, 0xE2811001, 0x3BFFFFFF,
        ];
        let instrs = instrs.map(Instr::decode);
        for instr in instrs {
            if let Some((c, i)) = instr {
                println!("{}", i.display(c));
            } else {
                panic!("---")
            }
        }
    }
}
