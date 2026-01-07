use std::{collections::BTreeMap, fmt::Display};

mod assembler;
mod parser;
mod syntax;

use assembler::HealStrategy;

#[derive(Debug)]
pub struct AssemblerOutput {
    pub labels: BTreeMap<String, u32>,
    pub instrs: Vec<u32>,
    pub warnings: Vec<AssemblerWarning>,
    pub passes: usize,
}

#[derive(Debug)]
pub struct AssemblerError {
    pub line_number: usize,
    pub error: LineError,
}

#[derive(Debug)]
pub enum LineError {
    ParseError(String),
    LabelNotFound(String),
    ShiftOutOfRange,
    MisalignedBranchOffset,
    OffsetOutOfRange,
    ImmediateOutOfRange(u32),
    InvalidShiftType,
    InvalidStoreSize,
    AddressTooComplex,
    TooManyPasses,
}

impl Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineError::ParseError(s) => write!(f, "{s}"),
            LineError::LabelNotFound(label) => write!(f, "label '{label}' not found"),
            LineError::ShiftOutOfRange => write!(f, "shift out of range"),
            LineError::MisalignedBranchOffset => write!(f, "branch offset was not 4-byte aligned"),
            LineError::OffsetOutOfRange => write!(f, "offset out of range"),
            LineError::ImmediateOutOfRange(n) => write!(f, "value {n} out of range"),
            LineError::InvalidShiftType => write!(f, "invalid shift type"),
            LineError::InvalidStoreSize => write!(f, "invalid store size"),
            LineError::AddressTooComplex => write!(f, "address too complex for this instruction"),
            LineError::TooManyPasses => {
                write!(f, "too many passes were needed to assemble; aborting")
            }
        }
    }
}

#[derive(Debug)]
pub struct AssemblerWarning {
    pub line_number: usize,
    pub warning: LineWarning,
}

#[derive(Debug)]
pub enum LineWarning {}

pub fn assemble(src: &str) -> Result<AssemblerOutput, Vec<AssemblerError>> {
    crate::assemble::assembler::assemble(
        crate::assemble::parser::parse(src)?,
        if src.lines().any(|line| line.trim() == "; HEAL OFF") {
            HealStrategy::Off
        } else if src.lines().any(|line| line.trim() == "; HEAL SIMPLE") {
            HealStrategy::Simple
        } else {
            HealStrategy::Advanced(crate::instr::Register::R12)
        },
    )
    .map_err(|e| vec![e])
}
