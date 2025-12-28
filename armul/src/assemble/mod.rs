use std::collections::BTreeMap;

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
    ExpectedComma(String),
    ExpectedRegister(String),
    UnrecognisedOpcode(String),
    ExpectedMnemonic(String),
    UnrecognisedAtEnd(String),
    ExpectedNumber(String),
    AboveRadix,
    ExpectedShift(String),
    LabelNotFound(String),
    ShiftOutOfRange,
    MisalignedBranchOffset,
    OffsetOutOfRange,
    ImmediateOutOfRange(u32),
    InvalidShiftType,
    InvalidPsr,
}

#[derive(Debug)]
pub struct AssemblerWarning {
    pub line_number: usize,
    pub warning: LineWarning,
}

#[derive(Debug)]
pub enum LineWarning {}

pub fn assemble(src: &str) -> Result<AssemblerOutput, AssemblerError> {
    crate::assemble::assembler::assemble(
        crate::assemble::parser::Parser::new(&src.to_uppercase()).parse()?,
        if src.lines().any(|line| line.trim() == "; HEAL OFF") {
            HealStrategy::Off
        } else if src.lines().any(|line| line.trim() == "; HEAL SIMPLE") {
            HealStrategy::Simple
        } else {
            HealStrategy::Advanced(crate::instr::Register::R12)
        },
    )
}
