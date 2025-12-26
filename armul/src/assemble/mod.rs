use std::collections::BTreeMap;

use crate::instr::{Cond, Instr};

mod assembler;
mod parser;
mod syntax;

#[derive(Debug)]
pub struct AssemblerOutput {
    labels: BTreeMap<String, u32>,
    instrs: Vec<(Cond, Instr)>,
    warnings: Vec<AssemblerWarning>,
    passes: usize,
}

#[derive(Debug)]
pub struct AssemblerError {
    pub line_number: usize,
    pub error: LineError,
}

#[derive(Debug)]
pub enum LineError {
    ExpectedComma,
    ExpectedRegister,
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
    )
}

#[cfg(test)]
mod tests {
    use crate::assemble::{AssemblerError, assemble};

    #[test]
    fn test_assemble() -> Result<(), AssemblerError> {
        let assembled = assemble(include_str!("../../test/divide.s"))?;
        println!("{assembled:#?}");
        for (cond, instr) in assembled.instrs {
            println!("{}", instr.display(cond));
        }
        panic!();
        Ok(())
    }
}
