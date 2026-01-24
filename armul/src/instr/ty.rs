//! Defines the ARM instruction set.

use std::{fmt::Display, str::FromStr};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Serialize;
use serde_repr::Serialize_repr;

/// Enumerates the registers that can be directly referenced in code.
/// In reality there are a total of 37 registers.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, Serialize_repr,
)]
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

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A condition to execute an instruction on.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromPrimitive, Serialize_repr,
)]
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

impl FromStr for Cond {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EQ" | "eq" => Ok(Cond::EQ),
            "NE" | "ne" => Ok(Cond::NE),
            "CS" | "cs" => Ok(Cond::CS),
            "CC" | "cc" => Ok(Cond::CC),
            "MI" | "mi" => Ok(Cond::MI),
            "PL" | "pl" => Ok(Cond::PL),
            "VS" | "vs" => Ok(Cond::VS),
            "VC" | "vc" => Ok(Cond::VC),
            "HI" | "hi" => Ok(Cond::HI),
            "LS" | "ls" => Ok(Cond::LS),
            "GE" | "ge" => Ok(Cond::GE),
            "LT" | "lt" => Ok(Cond::LT),
            "GT" | "gt" => Ok(Cond::GT),
            "LE" | "le" => Ok(Cond::LE),
            "AL" | "al" | "" => Ok(Cond::AL),
            _ => Err(()),
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
        /// Where to transfer from.
        psr: Psr,
        target: Register,
    },
    /// Move to Status from Register (MSR).
    Msr {
        /// Where to transfer to.
        psr: Psr,
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
        /// - a 12-bit unsigned constant;
        /// - a shifted register not using a register-specified shift amount.
        offset: TransferOperand,
    },
    /// Single Data Transfer Special (LDRH, LDRSB, LDRSH, STRH).
    SingleTransferSpecial {
        kind: TransferKind,
        /// Sign-extended transfers are only valid in loads.
        size: TransferSizeSpecial,
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
        /// - an unsigned 8-bit constant;
        /// - a register.
        offset: SpecialOperand,
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
    Constant(RotatedConstant),
    /// The second operand is contained in a register,
    /// possibly shifted in some way.
    Register(Register, Shift),
}

impl Display for DataOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOperand::Constant(i) => write!(f, "#{i}"),
            DataOperand::Register(register, shift) => write!(f, "{register}{shift}"),
        }
    }
}

impl DataOperand {
    pub fn is_register_specified_shift(self) -> bool {
        matches!(
            self,
            DataOperand::Register(
                _,
                Shift {
                    shift_type: _,
                    shift_amount: ShiftAmount::Register(_),
                },
            )
        )
    }
}

/// The last operand used in a single transfer instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOperand {
    /// The operand is a constant 12-bit value.
    Constant(u16),
    /// The operand is contained in a register.
    /// Register-specified shifts are not allowed.
    Register(Register, Shift),
}

impl Display for TransferOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferOperand::Constant(i) => write!(f, "#{i}"),
            TransferOperand::Register(register, shift) => write!(f, "{register}{shift}"),
        }
    }
}

/// The last operand used in a special single transfer instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialOperand {
    /// The operand is a constant 8-bit value.
    Constant(u8),
    /// The operand is contained in a register.
    Register(Register),
}

impl Display for SpecialOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialOperand::Constant(i) => write!(f, "#{i}"),
            SpecialOperand::Register(register) => write!(f, "{register}"),
        }
    }
}

/// A 32-bit value encoded as a bit-rotated 8-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotatedConstant {
    pub immediate: u8,
    /// `immediate` is rotated right by twice this value.
    pub half_rotate: u8,
}

impl Display for RotatedConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.half_rotate == 0 {
            write!(f, "{}", self.immediate)
        } else {
            write!(f, "{},ROR {}", self.immediate, self.half_rotate * 2)
        }
    }
}

impl RotatedConstant {
    /// Attempt to encode this 32-bit value in only 12 bits.
    pub fn encode(value: u32) -> Option<Self> {
        // The algorithm is very simple: attempt to rotate left by
        // all possible values (0, 2, ..., 30), and see if any of the
        // results fit into 8 bits.
        for half_rotate in 0..16 {
            let immediate = value.rotate_left(half_rotate * 2);
            if immediate <= 0xFF {
                return Some(Self {
                    immediate: immediate as u8,
                    half_rotate: half_rotate as u8,
                });
            }
        }
        None
    }

    /// Returns the result of evaluating this constant,
    /// as well as the barrel shifter's carry out.
    pub fn value(self) -> (u32, bool) {
        let result = (self.immediate as u32).rotate_right(self.half_rotate as u32 * 2);
        (result, result & (1 << 31) != 0)
    }
}

/// The possible ways to shift the second operand
/// of a data-processing instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "type", content = "value")]
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
            ShiftAmount::Register(register) => write!(f, "{register}"),
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
}

impl Display for TransferSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferSize::Byte => write!(f, "B"),
            TransferSize::Word => Ok(()),
        }
    }
}

/// How much data is to be transferred by a transfer instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferSizeSpecial {
    HalfWord,
    SignExtendedByte,
    SignExtendedHalfWord,
}

impl Display for TransferSizeSpecial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferSizeSpecial::HalfWord => write!(f, "H"),
            TransferSizeSpecial::SignExtendedByte => write!(f, "SB"),
            TransferSizeSpecial::SignExtendedHalfWord => write!(f, "SH"),
        }
    }
}

/// Program status register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, Serialize)]
pub enum Psr {
    Cpsr,
    Spsr,
}

impl Display for Psr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Psr::Cpsr => write!(f, "CPSR"),
            Psr::Spsr => write!(f, "SPSR"),
        }
    }
}
