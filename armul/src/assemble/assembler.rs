//! Assembles parsed assembly into real 32-bit instructions.

use std::{collections::BTreeMap, ops::Mul};

use crate::{
    assemble::{
        AssemblerError, AssemblerOutput, LineError,
        syntax::{self, AnyTransferSize, AsmInstr, AsmLine, AsmLineContents, Expression},
    },
    instr::{
        self, DataOp, Instr, Register, RotatedConstant, Shift, SpecialOperand, TransferKind,
        TransferSizeSpecial,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealStrategy {
    Off,
    Simple,
    /// An advanced healing strategy that lets us use a dummy register.
    Advanced(Register),
}

pub fn assemble(
    lines: Vec<AsmLine>,
    heal: HealStrategy,
) -> Result<AssemblerOutput, AssemblerError> {
    // Create a mapping of labels to their absolute addresses.
    // For the moment let's just say that every label is mapped to 0.
    let labels = lines
        .iter()
        .filter_map(|line| match &line.contents {
            AsmLineContents::Label(label) => Some(label),
            AsmLineContents::Equ(label, _) => Some(label),
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
    let mut i = 0;
    loop {
        output.instrs = Vec::new();
        output.warnings = Vec::new();
        output.passes += 1;
        if !single_pass(&lines, heal, &mut output)? {
            break;
        }
        i += 1;
        if i > 10 {
            return Err(AssemblerError {
                line_number: 0,
                error: LineError::TooManyPasses,
            });
        }
    }
    Ok(output)
}

/// Returns true if anything in the assembler's output changed
/// since last pass.
fn single_pass(
    lines: &[AsmLine],
    heal: HealStrategy,
    output: &mut AssemblerOutput,
) -> Result<bool, AssemblerError> {
    let mut program_counter = 0u32;
    let mut anything_changed = false;
    for line in lines {
        match &line.contents {
            AsmLineContents::Empty => {}
            AsmLineContents::Label(label) => {
                let entry = output.labels.entry(label.to_owned()).or_default();
                if *entry != program_counter {
                    anything_changed = true;
                    *entry = program_counter;
                }
            }
            AsmLineContents::Instr(cond, asm_instr) => {
                let instrs =
                    assemble_instr(line.line_number, heal, program_counter, asm_instr, output)?;
                program_counter += 4 * instrs.len() as u32;
                output.instrs.extend(
                    instrs
                        .into_iter()
                        .map(|i| i.encode(*cond))
                        .collect::<Result<Vec<u32>, LineError>>()
                        .map_err(|error| AssemblerError {
                            line_number: line.line_number,
                            error,
                        })?,
                );
            }
            AsmLineContents::Equ(name, expression) => {
                let value = expression.evaluate(line.line_number, output)?;
                let entry = output.labels.entry(name.to_owned()).or_default();
                if *entry != value {
                    anything_changed = true;
                    *entry = value;
                }
            }
            AsmLineContents::DefWord(expression) => {
                let value = expression.evaluate(line.line_number, output)?;
                program_counter += 4;
                output.instrs.push(value);
            }
        }
    }
    Ok(anything_changed)
}

fn assemble_instr(
    line_number: usize,
    heal: HealStrategy,
    program_counter: u32,
    asm_instr: &AsmInstr,
    output: &mut AssemblerOutput,
) -> Result<Vec<Instr>, AssemblerError> {
    match asm_instr {
        AsmInstr::BranchExchange { operand } => {
            Ok(vec![Instr::BranchExchange { operand: *operand }])
        }
        AsmInstr::Branch { link, target } => {
            let address = target.evaluate(line_number, output)?;
            let offset = (address as i32).wrapping_sub(program_counter as i32 + 8);
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
                offset,
            }])
        }
        AsmInstr::Adr {
            long: _,
            dest,
            expr,
        } => assemble_instr(
            line_number,
            heal,
            program_counter,
            &AsmInstr::Data {
                set_condition_codes: false,
                op: instr::DataOp::Mov,
                dest: *dest,
                op1: instr::Register::R0,
                op2: syntax::DataOperand::Constant(expr.clone()),
            },
            output,
        ),
        AsmInstr::Data {
            set_condition_codes,
            op,
            dest,
            op1,
            op2,
        } => with_operand(line_number, output, heal, op2, |op2| Instr::Data {
            set_condition_codes: *set_condition_codes,
            op: *op,
            dest: *dest,
            op1: *op1,
            op2,
        }),
        AsmInstr::Mrs { psr, target } => Ok(vec![Instr::Mrs {
            psr: *psr,
            target: *target,
        }]),
        AsmInstr::Msr { psr, source } => Ok(vec![Instr::Msr {
            psr: *psr,
            source: match source {
                syntax::MsrSource::Register(register) => instr::MsrSource::Register(*register),
                syntax::MsrSource::RegisterFlags(register) => {
                    instr::MsrSource::RegisterFlags(*register)
                }
                syntax::MsrSource::Flags(expression) => {
                    instr::MsrSource::Flags(expression.evaluate(line_number, output)?)
                }
            },
        }]),
        AsmInstr::Multiply {
            set_condition_codes,
            dest,
            op1,
            op2,
            addend,
        } => Ok(vec![Instr::Multiply {
            set_condition_codes: *set_condition_codes,
            dest: *dest,
            op1: *op1,
            op2: *op2,
            addend: *addend,
        }]),
        AsmInstr::MultiplyLong {
            set_condition_codes,
            signed,
            accumulate,
            dest_hi,
            dest_lo,
            op1,
            op2,
        } => Ok(vec![Instr::MultiplyLong {
            set_condition_codes: *set_condition_codes,
            signed: *signed,
            accumulate: *accumulate,
            dest_hi: *dest_hi,
            dest_lo: *dest_lo,
            op1: *op1,
            op2: *op2,
        }]),
        AsmInstr::SingleTransfer {
            kind,
            size: AnyTransferSize::Normal(size),
            write_back,
            offset_positive,
            pre_index,
            data_register,
            base_register,
            offset,
        } => with_transfer_operand(line_number, output, heal, offset, |offset| {
            Instr::SingleTransfer {
                kind: *kind,
                size: *size,
                write_back: *write_back,
                offset_positive: *offset_positive,
                pre_index: *pre_index,
                data_register: *data_register,
                base_register: *base_register,
                offset,
            }
        }),
        AsmInstr::SingleTransfer {
            kind,
            size: AnyTransferSize::Special(size),
            write_back,
            offset_positive,
            pre_index,
            data_register,
            base_register,
            offset,
        } => {
            if *kind == TransferKind::Store && *size != TransferSizeSpecial::HalfWord {
                return Err(AssemblerError {
                    line_number,
                    error: LineError::InvalidStoreSize,
                });
            }
            let mut instrs = Vec::new();
            let offset = match offset {
                syntax::DataOperand::Constant(expression) => {
                    let value = expression.evaluate(line_number, output)?;
                    if value <= 0xFF {
                        SpecialOperand::Constant(value as u8)
                        // TODO: What about negative offsets?
                    } else if let HealStrategy::Advanced(register) = heal {
                        instrs.extend(fill_register(value, register));
                        SpecialOperand::Register(register)
                    } else {
                        return Err(AssemblerError {
                            line_number,
                            error: LineError::AddressTooComplex,
                        });
                    }
                }
                syntax::DataOperand::Register(register, shift) => {
                    let shift_amount = match &shift.shift_amount {
                        syntax::ShiftAmount::Constant(expression) => {
                            expression.evaluate(line_number, output)?
                        }
                        syntax::ShiftAmount::Register(_) => {
                            return Err(AssemblerError {
                                line_number,
                                error: LineError::AddressTooComplex,
                            });
                        }
                    };
                    if shift_amount == 0 {
                        SpecialOperand::Register(*register)
                    } else {
                        return Err(AssemblerError {
                            line_number,
                            error: LineError::AddressTooComplex,
                        });
                    }
                }
            };
            instrs.push(Instr::SingleTransferSpecial {
                kind: *kind,
                size: *size,
                write_back: *write_back,
                offset_positive: *offset_positive,
                pre_index: *pre_index,
                data_register: *data_register,
                base_register: *base_register,
                offset,
            });
            Ok(instrs)
        }
        AsmInstr::BlockTransfer { .. } => todo!(),
        AsmInstr::Swap { .. } => todo!(),
        AsmInstr::SoftwareInterrupt { comment } => Ok(vec![Instr::SoftwareInterrupt {
            comment: comment.evaluate(line_number, output)?,
        }]),
    }
}

fn with_operand(
    line_number: usize,
    output: &AssemblerOutput,
    heal: HealStrategy,
    op: &syntax::DataOperand,
    instr: impl FnOnce(instr::DataOperand) -> Instr,
) -> Result<Vec<Instr>, AssemblerError> {
    match op {
        syntax::DataOperand::Constant(expression) => {
            let value = expression.evaluate(line_number, output)?;
            // Attempt to encode this 32-bit value in just 12 bits.
            let (mut instrs, operand) = encode_constant(line_number, heal, value)?;
            instrs.push(instr(operand));
            Ok(instrs)
        }
        syntax::DataOperand::Register(register, shift) => {
            Ok(vec![instr(instr::DataOperand::Register(
                *register,
                Shift {
                    shift_type: shift.shift_type,
                    shift_amount: match &shift.shift_amount {
                        syntax::ShiftAmount::Constant(expression) => instr::ShiftAmount::Constant(
                            expression.evaluate(line_number, output)? as u8,
                        ),
                        syntax::ShiftAmount::Register(register) => {
                            instr::ShiftAmount::Register(*register)
                        }
                    },
                },
            ))])
        }
    }
}

fn with_transfer_operand(
    line_number: usize,
    output: &AssemblerOutput,
    heal: HealStrategy,
    op: &syntax::DataOperand,
    instr: impl FnOnce(instr::TransferOperand) -> Instr,
) -> Result<Vec<Instr>, AssemblerError> {
    match op {
        syntax::DataOperand::Constant(expression) => {
            let value = expression.evaluate(line_number, output)?;
            if value < 1 << 12 {
                Ok(vec![instr(instr::TransferOperand::Constant(value as u16))])
            } else if let HealStrategy::Advanced(register) = heal {
                let mut instrs = fill_register(value, register);
                instrs.push(instr(instr::TransferOperand::Register(
                    register,
                    Shift {
                        shift_type: instr::ShiftType::LogicalLeft,
                        shift_amount: instr::ShiftAmount::Constant(0),
                    },
                )));
                Ok(instrs)
            } else {
                Err(AssemblerError {
                    line_number,
                    error: LineError::ImmediateOutOfRange(value),
                })
            }
        }
        syntax::DataOperand::Register(register, shift) => {
            Ok(vec![instr(instr::TransferOperand::Register(
                *register,
                Shift {
                    shift_type: shift.shift_type,
                    shift_amount: match &shift.shift_amount {
                        syntax::ShiftAmount::Constant(expression) => instr::ShiftAmount::Constant(
                            expression.evaluate(line_number, output)? as u8,
                        ),
                        syntax::ShiftAmount::Register(register) => {
                            instr::ShiftAmount::Register(*register)
                        }
                    },
                },
            ))])
        }
    }
}

/// Return instructions that fill the given register with the prescribed value,
/// using all healing strategies.
///
/// TODO: What if the register is R15?
#[must_use]
pub fn fill_register(value: u32, register: Register) -> Vec<Instr> {
    // Try a direct move strategy first as in encode_constant.
    if let Some(constant) = RotatedConstant::encode(value) {
        return vec![Instr::Data {
            set_condition_codes: false,
            op: DataOp::Mov,
            dest: register,
            op1: Register::R0,
            op2: instr::DataOperand::Constant(constant),
        }];
    }

    // Try a negated move next.
    if let Some(constant) = RotatedConstant::encode(!value) {
        return vec![Instr::Data {
            set_condition_codes: false,
            op: DataOp::Mvn,
            dest: register,
            op1: Register::R0,
            op2: instr::DataOperand::Constant(constant),
        }];
    }

    // Slice off the lowest significant byte (or 7 bits if misaligned) and try again.
    let trailing_zeros = (value.trailing_zeros() / 2) * 2;
    let shift = trailing_zeros + 8;
    let mut instrs = fill_register(value >> shift << shift, register);
    // Now do `orr Rd, Rd, (extra)` to fill the remaining bits.
    instrs.push(Instr::Data {
        set_condition_codes: false,
        op: DataOp::Orr,
        dest: register,
        op1: register,
        op2: instr::DataOperand::Constant(RotatedConstant {
            immediate: ((value & (0xFF << trailing_zeros)) >> trailing_zeros) as u8,
            half_rotate: ((16 - trailing_zeros / 2) & 0b1111) as u8,
        }),
    });
    instrs
}

fn encode_constant(
    line_number: usize,
    heal: HealStrategy,
    value: u32,
) -> Result<(Vec<Instr>, instr::DataOperand), AssemblerError> {
    if let Some(constant) = RotatedConstant::encode(value) {
        Ok((Vec::new(), instr::DataOperand::Constant(constant)))
    } else if let HealStrategy::Advanced(reg) = heal {
        Ok((
            fill_register(value, reg),
            instr::DataOperand::Register(
                reg,
                Shift {
                    shift_type: instr::ShiftType::LogicalLeft,
                    shift_amount: instr::ShiftAmount::Constant(0),
                },
            ),
        ))
    } else {
        Err(AssemblerError {
            line_number,
            error: LineError::ImmediateOutOfRange(value),
        })
    }
}

impl Expression {
    pub fn evaluate(
        &self,
        line_number: usize,
        output: &AssemblerOutput,
    ) -> Result<u32, AssemblerError> {
        match self {
            Expression::Constant(x) => Ok(*x),
            Expression::Label(label) => match output.labels.get(label) {
                Some(address) => Ok(*address),
                None => Err(AssemblerError {
                    line_number,
                    error: LineError::LabelNotFound(label.to_owned()),
                }),
            },
            Expression::Mul(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .wrapping_mul(rhs.evaluate(line_number, output)?)),
            Expression::Div(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .wrapping_div(rhs.evaluate(line_number, output)?)),
            Expression::Add(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .wrapping_add(rhs.evaluate(line_number, output)?)),
            Expression::Sub(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .wrapping_sub(rhs.evaluate(line_number, output)?)),
            Expression::Or(lhs, rhs) => {
                Ok(lhs.evaluate(line_number, output)? | rhs.evaluate(line_number, output)?)
            }
            Expression::Lsl(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .wrapping_shl(rhs.evaluate(line_number, output)?)),
            Expression::Lsr(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .wrapping_shr(rhs.evaluate(line_number, output)?)),
            Expression::Asr(lhs, rhs) => Ok((lhs.evaluate(line_number, output)? as i32
                >> rhs.evaluate(line_number, output)?)
                as u32),
            Expression::Ror(lhs, rhs) => Ok(lhs
                .evaluate(line_number, output)?
                .rotate_right(rhs.evaluate(line_number, output)?)),
        }
    }
}
