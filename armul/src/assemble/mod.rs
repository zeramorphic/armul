use std::collections::BTreeMap;

mod assembler;
mod parser;
mod syntax;

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
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        assemble::{AssemblerError, assemble},
        instr::Instr,
    };

    #[test]
    fn test_assemble() -> Result<(), AssemblerError> {
        let assembled = assemble(include_str!("../../test/divide.s"))?;
        println!("{assembled:#?}");
        for x in assembled.instrs {
            let instr = Instr::decode(x).map(|(cond, instr)| instr.display(cond));
            println!("{x:0>8X}: {}", instr.as_deref().unwrap_or("???"));
            assert!(instr.is_some());
        }
        Ok(())
    }
}
