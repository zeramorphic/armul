//! Assembles parsed assembly into real 32-bit instructions.

use std::{any, collections::BTreeMap};

use crate::{
    assemble::{
        AssemblerError, AssemblerOutput, LineError,
        syntax::{self, AsmInstr, AsmLine, AsmLineContents, Expression},
    },
    instr::{self, Cond, Instr, Shift},
};

pub fn assemble(lines: Vec<AsmLine>) -> Result<AssemblerOutput, AssemblerError> {
    // Create a mapping of labels to their absolute addresses.
    // For the moment let's just say that every label is mapped to 0.
    let labels = lines
        .iter()
        .filter_map(|line| match &line.contents {
            AsmLineContents::Label(label) => Some(label),
            _ => None,
        })
        .map(|x| (x.to_owned(), 0))
        .collect::<BTreeMap<String, u32>>();
    // Attempt to assemble the input given this mapping of labels.
    // Repeat using the updated mapping of labels until the mapping doesn't change.
    // This approach allows for a certain amount of dynamic error correction
    // to be done on users' code, for example allowing out-of-range values
    // by adding extra instructions to construct them.
    let mut output = AssemblerOutput {
        labels,
        instrs: Vec::new(),
        warnings: Vec::new(),
        passes: 0,
    };
    loop {
        output.instrs = Vec::new();
        output.warnings = Vec::new();
        output.passes += 1;
        if !single_pass(&lines, &mut output)? {
            break;
        }
    }
    Ok(output)
}

/// Returns true if anything in the assembler's output changed
/// since last pass.
fn single_pass(lines: &[AsmLine], output: &mut AssemblerOutput) -> Result<bool, AssemblerError> {
    let mut program_counter = 0u32;
    let mut anything_changed = false;
    for line in lines {
        match &line.contents {
            AsmLineContents::Label(label) => {
                let entry = output.labels.entry(label.to_owned()).or_default();
                if *entry != program_counter {
                    anything_changed = true;
                    *entry = program_counter;
                }
            }
            AsmLineContents::Instr(cond, asm_instr) => {
                let instrs = assemble_instr(line.line_number, program_counter, asm_instr, output)?;
                program_counter += 4 * instrs.len() as u32;
                output.instrs.extend(instrs.into_iter().map(|i| (*cond, i)));
            }
        }
    }
    Ok(anything_changed)
}

fn assemble_instr(
    line_number: usize,
    program_counter: u32,
    asm_instr: &AsmInstr,
    output: &mut AssemblerOutput,
) -> Result<Vec<Instr>, AssemblerError> {
    match asm_instr {
        AsmInstr::BranchExchange { operand } => todo!(),
        AsmInstr::Branch { link, target } => {
            let address = target.evaluate(line_number, output)?;
            let offset = address - (program_counter as i64 + 8);
            // Check that the offset is 4 * some signed 24-bit value.
            if offset % 4 != 0 {
                return Err(AssemblerError {
                    line_number,
                    error: LineError::MisalignedBranchOffset,
                });
            }
            if !(-(1 << 24)..(1 << 24)).contains(&(offset >> 2)) {
                return Err(AssemblerError {
                    line_number,
                    error: LineError::OffsetOutOfRange,
                });
            }
            Ok(vec![Instr::Branch {
                link: *link,
                offset: offset as i32,
            }])
        }
        AsmInstr::Data {
            set_condition_codes,
            op,
            dest,
            op1,
            op2,
        } => {
            let offset = if op2.is_register_specified_shift() {
                12
            } else {
                8
            };
            with_operand(line_number, output, op2, |op2| Instr::Data {
                set_condition_codes: *set_condition_codes,
                op: *op,
                dest: *dest,
                op1: *op1,
                op2,
            })
        }
        AsmInstr::Mrs { psr, target } => todo!(),
        AsmInstr::Msr { psr, source } => todo!(),
        AsmInstr::Multiply {
            set_condition_codes,
            dest,
            op1,
            op2,
            addend,
        } => todo!(),
        AsmInstr::MultiplyLong {
            set_condition_codes,
            signed,
            accumulate,
            dest_hi,
            dest_lo,
            op1,
            op2,
        } => todo!(),
        AsmInstr::SingleTransfer {
            kind,
            size,
            write_back,
            offset_positive,
            pre_index,
            data_register,
            base_register,
            offset,
        } => todo!(),
        AsmInstr::BlockTransfer {
            kind,
            write_back,
            offset_positive,
            pre_index,
            psr,
            base_register,
            registers,
        } => todo!(),
        AsmInstr::Swap {
            byte,
            dest,
            source,
            base,
        } => todo!(),
        AsmInstr::SoftwareInterrupt { comment } => todo!(),
    }
}

fn with_operand(
    line_number: usize,
    output: &AssemblerOutput,
    op: &syntax::DataOperand,
    instr: impl FnOnce(instr::DataOperand) -> Instr,
) -> Result<Vec<Instr>, AssemblerError> {
    match op {
        syntax::DataOperand::Constant(expression) => {
            let value = expression.evaluate(line_number, output)?;
            // For now let's not do any checking to determine whether this value is
            // even representable using the 8-bit + 4-bit-rotate system.
            Ok(vec![instr(instr::DataOperand::Constant(value as u32))])
        }
        syntax::DataOperand::Register(register, shift) => {
            Ok(vec![instr(instr::DataOperand::Register(
                *register,
                Shift {
                    shift_type: shift.shift_type,
                    shift_amount: match &shift.shift_amount {
                        syntax::ShiftAmount::Constant(expression) => {
                            let value = expression.evaluate(line_number, output)?;
                            // This value had better be a 5-bit unsigned integer.
                            if !(0..=0b11111).contains(&value) {
                                return Err(AssemblerError {
                                    line_number,
                                    error: LineError::ShiftOutOfRange,
                                });
                            }
                            instr::ShiftAmount::Constant(value as u8)
                        }
                        syntax::ShiftAmount::Register(register) => {
                            instr::ShiftAmount::Register(*register)
                        }
                    },
                },
            ))])
        }
    }
}

impl Expression {
    pub fn evaluate(
        &self,
        line_number: usize,
        output: &AssemblerOutput,
    ) -> Result<i64, AssemblerError> {
        match self {
            Expression::Constant(x) => Ok(*x),
            Expression::Label(label) => match output.labels.get(label) {
                Some(address) => Ok(*address as i64),
                None => Err(AssemblerError {
                    line_number,
                    error: LineError::LabelNotFound(label.to_owned()),
                }),
            },
            Expression::Add(expression, expression1) => todo!(),
        }
    }
}
