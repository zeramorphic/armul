//! Abstract syntax for ARM assembly.

use crate::instr::{Cond, DataOp, Psr, Register, ShiftType, TransferKind, TransferSize};

#[derive(Debug)]
pub struct AsmLine {
    pub line_number: usize,
    pub contents: AsmLineContents,
    pub comment: String,
}

#[derive(Debug)]
pub enum AsmLineContents {
    Label(String),
    Instr(Cond, AsmInstr),
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
        size: TransferSize,
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
        comment: u32,
    },
}

#[derive(Debug)]
pub enum DataOperand {
    Constant(Expression),
    Register(Register, Shift),
}

impl DataOperand {
    pub fn is_register_specified_shift(&self) -> bool {
        match self {
            DataOperand::Constant(_) => false,
            DataOperand::Register(_, _) => true,
        }
    }
}

#[derive(Debug)]
pub struct Shift {
    pub shift_type: ShiftType,
    pub shift_amount: ShiftAmount,
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

#[derive(Debug)]
pub enum Expression {
    Constant(i64),
    Label(String),
    Add(Box<Expression>, Box<Expression>),
}
