//! Abstract syntax for ARM assembly.

use std::fmt::Display;

use crate::instr::{
    Cond, DataOp, Psr, Register, ShiftType, TransferKind, TransferSize, TransferSizeSpecial,
};

#[derive(Debug)]
pub struct AsmLine {
    pub line_number: usize,
    pub contents: AsmLineContents,
    pub comment: String,
}

#[derive(Debug)]
pub enum AsmLineContents {
    Empty,
    Label(String),
    Instr(Cond, AsmInstr),
    Equ(String, Expression),
    DefWord(Expression),
}

/// An instruction that might contain expressions or labels.
/// See [armul::instr::Instr] for more information and documentation.
#[derive(Debug)]
pub enum AsmInstr {
    BranchExchange {
        operand: Register,
    },
    Branch {
        link: bool,
        target: Expression,
    },
    Adr {
        dest: Register,
        expr: Expression,
    },
    Data {
        set_condition_codes: bool,
        op: DataOp,
        dest: Register,
        op1: Register,
        op2: DataOperand,
    },
    Mrs {
        psr: Psr,
        target: Register,
    },
    Msr {
        psr: Psr,
        source: MsrSource,
    },
    Multiply {
        set_condition_codes: bool,
        dest: Register,
        op1: Register,
        op2: Register,
        addend: Option<Register>,
    },
    MultiplyLong {
        set_condition_codes: bool,
        signed: bool,
        accumulate: bool,
        dest_hi: Register,
        dest_lo: Register,
        op1: Register,
        op2: Register,
    },
    SingleTransfer {
        kind: TransferKind,
        size: AnyTransferSize,
        write_back: bool,
        offset_positive: bool,
        pre_index: bool,
        data_register: Register,
        base_register: Register,
        offset: DataOperand,
    },
    BlockTransfer {
        kind: TransferKind,
        write_back: bool,
        offset_positive: bool,
        pre_index: bool,
        psr: bool,
        base_register: Register,
        registers: u16,
    },
    Swap {
        byte: bool,
        dest: Register,
        source: Register,
        base: Register,
    },
    SoftwareInterrupt {
        comment: Expression,
    },
}

#[derive(Debug)]
pub enum DataOperand {
    Constant(Expression),
    Register(Register, Shift),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnyTransferSize {
    Normal(TransferSize),
    Special(TransferSizeSpecial),
}

impl Display for AnyTransferSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyTransferSize::Normal(transfer_size) => write!(f, "{transfer_size}"),
            AnyTransferSize::Special(transfer_size_special) => write!(f, "{transfer_size_special}"),
        }
    }
}

#[derive(Debug)]
pub struct Shift {
    pub shift_type: ShiftType,
    pub shift_amount: ShiftAmount,
}

impl Default for Shift {
    fn default() -> Self {
        Self {
            shift_type: ShiftType::LogicalLeft,
            shift_amount: ShiftAmount::Constant(Expression::Constant(0)),
        }
    }
}

#[derive(Debug)]
pub enum ShiftAmount {
    Constant(Expression),
    Register(Register),
}

#[derive(Debug)]
pub enum MsrSource {
    /// Transfer entirely from a register.
    Register(Register),
    /// Transfer only the flag bits from a register.
    RegisterFlags(Register),
    /// Transfer the flag bits of the given 32-bit value.
    Flags(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(u32),
    Label(String),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Lsl(Box<Expression>, Box<Expression>),
    Lsr(Box<Expression>, Box<Expression>),
    Asr(Box<Expression>, Box<Expression>),
    Ror(Box<Expression>, Box<Expression>),
}
