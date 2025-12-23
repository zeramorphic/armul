//! Defines the ARM instruction set.

use std::fmt::Display;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

/// Enumerates the registers that can be directly referenced in code.
/// In reality there are a total of 37 registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive)]
#[repr(u8)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    /// Also used for the stack pointer `SP`.
    R13,
    /// Also used for the link register `LR`.
    R14,
    /// Also used for the program counter `PC`.
    R15,
}

impl Register {
    pub fn from_u4(value: u32, offset: usize) -> Register {
        Register::from_u32((value >> offset) & 0xF).unwrap()
    }
}

/// A condition to execute an instruction on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive)]
#[repr(u8)]
pub enum Cond {
    /// Z set (equal)
    EQ,
    /// Z clear (not equal)
    NE,
    /// C set (unsigned higher or same)
    CS,
    /// C clear (unsigned lower)
    CC,
    /// N set (negative)
    MI,
    /// N clear (positive or zero)
    PL,
    /// V set (overflow)
    VS,
    /// V clear (no overflow)
    VC,
    /// C set and Z clear (unsigned higher)
    HI,
    /// C clear or Z set (unsigned lower or same)
    LS,
    /// N equals V (greater or equal)
    GE,
    /// N not equal to V (less than)
    LT,
    /// Z clear AND (N equals V) (greater than)
    GT,
    /// Z set OR (N not equal to V) (less than or equal)
    LE,
    /// (ignored) (always)
    AL,
}

impl Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cond::AL => Ok(()),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// The list of instructions implemented in hardware in ARM7TDMI.
/// - This type is used for conversion to and from a u32 representation,
///   not for use in the front end of an assembler.
/// - This does not include virtual instructions such as `NOP` and `ADR(L)`.
/// - This does not include conditions.
/// - These instructions may include unencodable operations, such as data
///   manipulation calls with unencodable constants.
///
/// Additionally,
/// - Coprocessor operations are not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    /// Branch and Exchange (BX).
    ///
    /// Performs a branch by copying the contents of a register into the program counter.
    /// If the operand is R15, the behaviour is undefined.
    ///
    /// *Timing:* 2S + 1N cycles.
    BranchExchange { operand: Register },
    /// Branch (B), and Branch with Link (BL).
    ///
    /// Sets the program counter to the given offset from the PC.
    /// The offset is encoded as a two's-complement 24-bit number,
    /// which is multiplied by four before being used.
    /// Note that due to instruction pipelining, when this instruction is executed,
    /// the PC is already two instructions ahead.
    ///
    /// If the link flag is set, this also writes the program counter corresponding
    /// to the immediately following instruction into the link register `LR`.
    /// When the subroutine returns, it should set the `PC` to this value.
    ///
    /// *Timing:* 2S + 1N cycles.
    Branch { link: bool, offset: i32 },
    /// General data-processing instructions
    /// (ADD, EOR, SUB, RSB, ADD, ADC, SBC, RSC, TST, TEQ, CMP, CMN, ORR, MOV, BIC, MVN).
    ///
    /// *Timing:*
    /// - normal: 1S
    /// - register-specified shift in op2: 1S + 1I
    /// - PC written: 2S + 1N
    /// - register-specified shift and PC written: 2S + 1N + 1I
    ///
    /// This timing takes into account the pipeline flush (1N + 1S) that occurs
    /// when the program counter is overwritten.
    /// To simplify, this means that a data-processing instruction takes 1S
    /// to compute, but:
    /// - register-specified shifts incur a 1I cost; and
    /// - PC overwrites cause a pipeline flush, which costs 1N + 1S.
    Data {
        /// Whether the condition codes should be set after executing this instruction.
        set_condition_codes: bool,
        op: DataOp,
        dest: Register,
        op1: Register,
        /// The second operand can either be a constant or a register,
        /// possibly bit-shifted left or right in some way.
        op2: DataOperand,
    },
    /// Move to Register from Status (MRS).
    Mrs {
        /// Where to transfer from:
        /// - if false: the current program state register (CPSR);
        /// - if true: the saved program state register (SPSR).
        saved: bool,
        target: Register,
    },
    /// Move to Status from Register (MSR).
    Msr {
        /// Where to transfer to:
        /// - if false: the current program state register (CPSR);
        /// - if true: the saved program state register (SPSR).
        saved: bool,
        source: MsrSource,
    },
    /// Multiply (MUL) and Multiply-Accumulate (MLA).
    Multiply {
        /// Whether the condition codes should be set after executing this instruction.
        set_condition_codes: bool,
        dest: Register,
        op1: Register,
        op2: Register,
        /// If this is set, this is a Multiply-Accumulate (MLA) instruction,
        /// and the contents of this register are added to the product of
        /// op1 with op2.
        addend: Option<Register>,
    },
    /// Multiply Long (MULL) and Multiply-Accumuate Long (MLAL).
    MultiplyLong {
        /// Whether the condition codes should be set after executing this instruction.
        set_condition_codes: bool,
        /// Whether to treat all operands as signed 32-bit values and
        /// the result as a signed 64-bit value; otherwise we treat the operands
        /// as unsigned 32-bit values and the result as an unsigned 64-bit value.
        signed: bool,
        /// If this is true, we additionally treat the destination as a 64-bit
        /// operand to be added to the result.
        accumulate: bool,
        /// The high (most significant) 32 bits of the result.
        dest_hi: Register,
        /// The low (least significant) 32 bits of the result.
        dest_lo: Register,
        op1: Register,
        op2: Register,
    },
    /// Single Data Transfer (LDR, STR).
    SingleTransfer {
        kind: TransferKind,
        /// Sign-extended transfers are only valid in loads.
        size: TransferSize,
        /// If this is true, the computed address is
        /// written back into the base register.
        write_back: bool,
        /// If this is true, the offset is considered to be positive.
        /// Otherwise, it is considered to be negative.
        offset_positive: bool,
        /// If this is true, the offset is added before the transfer.
        pre_index: bool,
        /// The register to read from or write to (depending on the transfer kind).
        data_register: Register,
        /// The base register to use for computing the memory location to use.
        base_register: Register,
        /// The offset to use for this instruction.
        /// Some of these are unrepresentable.
        /// The valid operands are:
        /// - an unsigned 12-bit constant;
        /// - a shifted register not using a register-specified shift amount.
        offset: DataOperand,
    },
    /// Block Data Transfer (LDM, STM).
    BlockTransfer {
        kind: TransferKind,
        /// If this is true, the computed address is
        /// written back into the base register.
        write_back: bool,
        /// If this is true, the offset is considered to be positive.
        /// Otherwise, it is considered to be negative.
        offset_positive: bool,
        /// If this is true, the offset is added before the transfer.
        pre_index: bool,
        /// If this is true, load the PSR or force user mode.
        psr: bool,
        /// The base register to use for computing the memory location to use.
        base_register: Register,
        /// A bit field corresponding to the set of registers to use.
        registers: u16,
    },
    /// Single Data Swap (SWP).
    Swap {
        /// If this is true, only swap a byte; otherwise, swap a word.
        byte: bool,
        dest: Register,
        source: Register,
        base: Register,
    },
    /// Software Interrupt (SWI).
    SoftwareInterrupt {
        /// The payload to pass to the software interrupt handler.
        comment: u32,
    },
}

/// The possible data operations to use in a data-processing instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive)]
#[repr(u8)]
pub enum DataOp {
    /// Returns op1 bitwise AND op2.
    And,
    /// Returns op1 bitwise XOR op2.
    Eor,
    /// Returns op1 - op2.
    Sub,
    /// Returns op2 - op1.
    Rsb,
    /// Returns op1 + op2.
    Add,
    /// Returns op1 + op2 + carry.
    Adc,
    /// Returns op1 - op2 + carry - 1.
    Sbc,
    /// Returns op2 - op1 + carry - 1.
    Rsc,
    /// As `And`, but result is not written.
    Tst,
    /// As `Eor`, but result is not written.
    Teq,
    /// As `Sub`, but result is not written.
    Cmp,
    /// As `Add`, but result is not written.
    Cmn,
    /// Returns op1 bitwise OR op2.
    Orr,
    /// Returns op2; op1 is ignored.
    Mov,
    /// Returns op1 bitwise AND NOT op2 (bit clear).
    Bic,
    /// Returns bitwise NOT op2; op1 is ignored.
    Mvn,
}

impl Display for DataOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOp::And => write!(f, "AND"),
            DataOp::Eor => write!(f, "EOR"),
            DataOp::Sub => write!(f, "SUB"),
            DataOp::Rsb => write!(f, "RSB"),
            DataOp::Add => write!(f, "ADD"),
            DataOp::Adc => write!(f, "ADC"),
            DataOp::Sbc => write!(f, "SBC"),
            DataOp::Rsc => write!(f, "RSC"),
            DataOp::Tst => write!(f, "TST"),
            DataOp::Teq => write!(f, "TEQ"),
            DataOp::Cmp => write!(f, "CMP"),
            DataOp::Cmn => write!(f, "CMN"),
            DataOp::Orr => write!(f, "ORR"),
            DataOp::Mov => write!(f, "MOV"),
            DataOp::Bic => write!(f, "BIC"),
            DataOp::Mvn => write!(f, "MVN"),
        }
    }
}

/// The second operand used in a data-processing instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataOperand {
    /// The second operand is a constant value.
    /// Not all 32-bit constants can be represented in this way.
    Constant(u32),
    /// The second operand is contained in a register,
    /// possibly shifted in some way.
    Register(Register, Shift),
}

impl Display for DataOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOperand::Constant(i) => write!(f, "#{i}"),
            DataOperand::Register(register, shift) => write!(f, "{register:?}{shift}"),
        }
    }
}

/// The possible ways to shift the second operand
/// of a data-processing instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shift {
    pub shift_type: ShiftType,
    pub shift_amount: ShiftAmount,
}

impl Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.shift_type == ShiftType::RotateRightExtended {
            write!(f, ",RRX")
        } else if self.shift_amount == ShiftAmount::Constant(0) {
            Ok(())
        } else {
            write!(f, ",{} {}", self.shift_type, self.shift_amount)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive)]
#[repr(u8)]
pub enum ShiftType {
    /// Arithmetic left is the same as logical left.
    LogicalLeft,
    LogicalRight,
    ArithmeticRight,
    RotateRight,
    /// Rotate right by one bit position the 33-bit quantity obtained
    /// by appending the CPSR carry flag to the most significant end
    /// of the argument. Shift amount is ignored.
    RotateRightExtended,
}

impl Display for ShiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftType::LogicalLeft => write!(f, "LSL"),
            ShiftType::LogicalRight => write!(f, "LSR"),
            ShiftType::ArithmeticRight => write!(f, "ASR"),
            ShiftType::RotateRight => write!(f, "ROR"),
            ShiftType::RotateRightExtended => write!(f, "RRX"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShiftAmount {
    /// Shift by the given 5-bit unsigned integer.
    Constant(u8),
    /// Shift by the amount specified in the bottom byte of the given register.
    Register(Register),
}

impl Display for ShiftAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftAmount::Constant(i) => write!(f, "#{i}"),
            ShiftAmount::Register(register) => write!(f, "{register:?}"),
        }
    }
}

/// The source to transfer into a PSR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsrSource {
    /// Transfer entirely from a register.
    Register(Register),
    /// Transfer only the flag bits from a register.
    RegisterFlags(Register),
    /// Transfer the flag bits of the given 32-bit value.
    Flags(u32),
}

/// Whether a data transfer is a store (0) or a load (1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TransferKind {
    Store,
    Load,
}

/// How much data is to be transferred by a transfer instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferSize {
    Byte,
    Word,
    HalfWord,
    SignExtendedByte,
    SignExtendedHalfWord,
}

impl Display for TransferSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferSize::Byte => write!(f, "B"),
            TransferSize::Word => Ok(()),
            TransferSize::HalfWord => write!(f, "H"),
            TransferSize::SignExtendedByte => write!(f, "SB"),
            TransferSize::SignExtendedHalfWord => write!(f, "SH"),
        }
    }
}

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
                            saved: instr & (1 << 22) != 0,
                            target: Register::from_u4(instr, 12),
                        })
                    } else if instr & (0b1_1011_1111_1111 << 12) == 0b1_0010_1000_1111 << 12 {
                        // This is an MSR flag instruction.
                        Some(Instr::Msr {
                            saved: instr & (1 << 22) != 0,
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
                            saved: instr & (1 << 22) != 0,
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

    pub fn write(&self, cond: Cond, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Instr::BranchExchange { operand } => {
                write!(f, "BX{cond} {operand:?}")?;
            }
            Instr::Branch { link, offset } => {
                write!(f, "B")?;
                if *link {
                    write!(f, "L")?;
                }
                write!(f, "{cond} PC+#{offset}")?;
            }
            Instr::Data {
                set_condition_codes,
                op,
                dest,
                op1,
                op2,
            } => {
                write!(f, "{op}{cond}")?;
                if *set_condition_codes
                    && !matches!(op, DataOp::Cmp | DataOp::Cmn | DataOp::Teq | DataOp::Tst)
                {
                    write!(f, "S")?;
                }
                match op {
                    DataOp::Mov | DataOp::Mvn => {
                        write!(f, " {dest:?}")?;
                    }
                    DataOp::Cmp | DataOp::Cmn | DataOp::Teq | DataOp::Tst => {
                        write!(f, " {op1:?}")?;
                    }
                    _ => {
                        write!(f, " {dest:?},{op1:?}")?;
                    }
                }
                write!(f, ",{op2}")?;
            }
            Instr::Mrs { saved, target } => {
                write!(f, "MRS{cond} {target:?},")?;
                if *saved {
                    write!(f, "SPSR")?;
                } else {
                    write!(f, "CPSR")?;
                }
            }
            Instr::Msr { saved, source } => {
                write!(f, "MSR{cond} ")?;
                if *saved {
                    write!(f, "SPSR")?;
                } else {
                    write!(f, "CPSR")?;
                }
                match source {
                    MsrSource::Register(register) => {
                        write!(f, ",{register:?}")?;
                    }
                    MsrSource::RegisterFlags(register) => {
                        write!(f, "_flg,{register:?}")?;
                    }
                    MsrSource::Flags(c) => {
                        write!(f, "_flg,#{c}")?;
                    }
                }
            }
            Instr::Multiply {
                set_condition_codes,
                dest,
                op1,
                op2,
                addend,
            } => match addend {
                Some(addend) => {
                    write!(f, "MLA{cond}")?;
                    if *set_condition_codes {
                        write!(f, "S")?;
                    }
                    write!(f, "{dest:?},{op1:?},{op2:?},{addend:?}")?;
                }
                None => {
                    write!(f, "MUL{cond}")?;
                    if *set_condition_codes {
                        write!(f, "S")?;
                    }
                    write!(f, "{dest:?},{op1:?},{op2:?}")?;
                }
            },
            Instr::MultiplyLong {
                set_condition_codes,
                signed,
                accumulate,
                dest_hi,
                dest_lo,
                op1,
                op2,
            } => {
                if *signed {
                    write!(f, "S")?;
                } else {
                    write!(f, "U")?;
                }
                if *accumulate {
                    write!(f, "MLAL")?;
                } else {
                    write!(f, "MULL")?;
                }
                write!(f, "{cond}")?;
                if *set_condition_codes {
                    write!(f, "S")?;
                }
                write!(f, " {dest_lo:?},{dest_hi:?},{op1:?},{op2:?}")?;
            }
            Instr::SingleTransfer {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => {
                match kind {
                    TransferKind::Store => {
                        write!(f, "STR")?;
                    }
                    TransferKind::Load => {
                        write!(f, "LDR")?;
                    }
                }
                write!(f, "{cond}{size}")?;
                if *write_back && !*pre_index {
                    write!(f, "T")?;
                }
                write!(f, " {data_register:?},[{base_register:?}")?;
                if !*pre_index {
                    write!(f, "]")?;
                }
                match offset {
                    DataOperand::Constant(0) => {}
                    DataOperand::Constant(i) => {
                        write!(f, ",#")?;
                        if !*offset_positive {
                            write!(f, "-")?;
                        }
                        write!(f, "{i}")?;
                    }
                    DataOperand::Register(register, shift) => {
                        write!(f, ",")?;
                        if !*offset_positive {
                            write!(f, "-")?;
                        }
                        write!(f, "{register:?}{shift}")?;
                    }
                }
                if *pre_index {
                    write!(f, "]")?;
                    if *write_back {
                        write!(f, "!")?;
                    }
                }
            }
            Instr::BlockTransfer {
                kind,
                write_back,
                offset_positive,
                pre_index,
                psr,
                base_register,
                registers,
            } => {
                match kind {
                    TransferKind::Store => {
                        write!(f, "STM")?;
                    }
                    TransferKind::Load => {
                        write!(f, "LDM")?;
                    }
                }
                write!(f, "{cond}")?;
                let offset = match (kind, pre_index, *offset_positive) {
                    (TransferKind::Store, true, true) => "FA",
                    (TransferKind::Store, true, false) => "FD",
                    (TransferKind::Store, false, true) => "EA",
                    (TransferKind::Store, false, false) => "ED",
                    (TransferKind::Load, true, true) => "ED",
                    (TransferKind::Load, true, false) => "EA",
                    (TransferKind::Load, false, true) => "FD",
                    (TransferKind::Load, false, false) => "FA",
                };
                write!(f, "{offset} {base_register:?}")?;
                if *write_back {
                    write!(f, "!")?;
                }
                write!(f, ",{{")?;
                for (ix, i) in (0..16).filter(|i| (registers & (1 << i)) != 0).enumerate() {
                    if ix != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "R{i}")?;
                }
                write!(f, "}}")?;
                if *psr {
                    write!(f, "^")?;
                }
            }
            Instr::Swap {
                byte,
                dest,
                source,
                base,
            } => {
                write!(f, "SWP{cond}")?;
                if *byte {
                    write!(f, "B")?;
                }
                write!(f, " {dest:?},{source:?},[{base:?}]")?;
            }
            Instr::SoftwareInterrupt { comment } => {
                write!(f, "SWI{cond} {comment}")?;
            }
        }
        Ok(())
    }

    pub fn display(&self, cond: Cond) -> String {
        let mut w = String::new();
        self.write(cond, &mut w).unwrap();
        w
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
