//! Defines the ARM instruction set.

/// Enumerates the registers that can be directly referenced in code.
/// In reality there are a total of 37 registers.
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

/// A condition to execute an instruction on.
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
        dest: Register,
        op1: Register,
        /// The second operand can either be a constant or a register,
        /// possibly bit-shifted left or right in some way.
        op2: DataOperand,
    },
    /// Move to Register from State (MRS).
    Mrs {
        /// Where to transfer from:
        /// - if false: the current program state register (CPSR);
        /// - if true: the saved program state register (SPSR).
        saved: bool,
        target: Register,
    },
    /// Move to State from Register (MSR).
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

/// The second operand used in a data-processing instruction.
pub enum DataOperand {
    /// The second operand is a constant value.
    /// Not all 32-bit constants can be represented in this way.
    Constant(u32),
    /// The second operand is contained in a register,
    /// possibly shifted in some way.
    Register(Register, Shift),
}

/// The possible ways to shift the second operand
/// of a data-processing instruction.
pub struct Shift {
    pub shift_type: ShiftType,
    pub shift_amount: ShiftAmount,
}

#[repr(u8)]
pub enum ShiftType {
    /// Arithmetic left is the same as logical left.
    LogicalLeft,
    LogicalRight,
    ArithmeticRight,
    /// Rotating right by a constant amount 0 is the notation used to encode the
    /// "rotate right extended" procedure, rotating right by one bit position of
    /// the 33-bit quantity obtained by prepending the carry flag of the CPSR
    /// to the register to be shifted.
    RotateRight,
}

pub enum ShiftAmount {
    /// Shift by the given 5-bit unsigned integer.
    Constant(u8),
    /// Shift by the amount specified in the bottom byte of the given register.
    Register(Register),
}

/// The source to transfer into a PSR.
pub enum MsrSource {
    /// Transfer entirely from a register.
    Register(Register),
    /// Transfer only the flag bits from a register.
    RegisterFlags(Register),
    /// Transfer the flag bits of the given 32-bit value.
    Flags(u32),
}

/// Whether a data transfer is a store (0) or a load (1).
#[repr(u8)]
pub enum TransferKind {
    Store,
    Load,
}

/// How much data is to be transferred by a transfer instruction.
pub enum TransferSize {
    Byte,
    Word,
    HalfWord,
    SignExtendedByte,
    SignExtendedHalfWord,
}
