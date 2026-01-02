use serde::Serialize;

use crate::{
    assemble::AssemblerOutput,
    instr::{
        Cond, DataOp, DataOperand, Instr, MsrSource, Psr, Register, Shift, ShiftAmount,
        TransferKind,
    },
};

/// Information about a line of assembly or disassembled code.
#[derive(Debug, Serialize)]
pub struct LineInfo {
    /// The raw 32-bit value that this line contains.
    value: u32,
    /// The decoded instruction, if there was one.
    instr: Option<PrettyInstr>,
}

impl LineInfo {
    /// Generate the line info for the given value, given the symbol table information in the assembler output.
    pub fn new(
        address: u32,
        value: u32,
        assembled: Option<&AssemblerOutput>,
        disassemble: bool,
    ) -> Self {
        LineInfo {
            value,
            instr: if disassemble {
                Instr::decode(value).map(|(cond, instr)| PrettyInstr::new(address, cond, instr))
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PrettyInstr {
    opcode_prefix: String,
    cond: String,
    opcode_suffix: String,
    args: Vec<PrettyArgument>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum PrettyArgument {
    Register {
        register: Register,
        negative: bool,
        write_back: bool,
    },
    Psr {
        psr: Psr,
        flag: bool,
    },
    Shift(Shift),
    Constant {
        value: u32,
        style: ConstantStyle,
    },
    RegisterSet {
        registers: Vec<Register>,
        caret: bool,
    },
}

impl PrettyArgument {
    fn from_data_operand(value: DataOperand) -> Vec<Self> {
        match value {
            DataOperand::Constant(rotated_constant) => vec![PrettyArgument::Constant {
                value: rotated_constant.value().0,
                style: ConstantStyle::Unknown,
            }],
            DataOperand::Register(register, shift) => match shift.shift_amount {
                ShiftAmount::Constant(0) => vec![PrettyArgument::Register {
                    register,
                    negative: false,
                    write_back: false,
                }],
                _ => vec![
                    PrettyArgument::Register {
                        register,
                        negative: false,
                        write_back: false,
                    },
                    PrettyArgument::Shift(shift),
                ],
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ConstantStyle {
    Address,
    UnsignedDecimal,
    Unknown,
}

impl PrettyInstr {
    pub fn new(address: u32, cond: Cond, instr: Instr) -> Self {
        let (opcode_prefix, opcode_suffix) = match instr {
            Instr::BranchExchange { .. } => ("BX".to_owned(), "".to_owned()),
            Instr::Branch { link: false, .. } => ("B".to_owned(), "".to_owned()),
            Instr::Branch { link: true, .. } => ("B".to_owned(), "".to_owned()),
            Instr::Data {
                set_condition_codes,
                op,
                ..
            } => (
                op.to_string(),
                if set_condition_codes
                    && !matches!(op, DataOp::Teq | DataOp::Tst | DataOp::Cmp | DataOp::Cmn)
                {
                    "S".to_owned()
                } else {
                    "".to_owned()
                },
            ),
            Instr::Mrs { .. } => ("MRS".to_owned(), "".to_owned()),
            Instr::Msr { .. } => ("MSR".to_owned(), "".to_owned()),
            Instr::Multiply {
                set_condition_codes,
                addend,
                ..
            } => (
                match addend.is_some() {
                    true => "MLA".to_owned(),
                    false => "MUL".to_owned(),
                },
                if set_condition_codes {
                    "S".to_owned()
                } else {
                    "".to_owned()
                },
            ),
            Instr::MultiplyLong {
                set_condition_codes,
                signed,
                accumulate,
                ..
            } => {
                let lhs = match (signed, accumulate) {
                    (true, true) => "SMLAL",
                    (true, false) => "SMULL",
                    (false, true) => "UMLAL",
                    (false, false) => "UMULL",
                };
                (
                    lhs.to_owned(),
                    if set_condition_codes {
                        "S".to_owned()
                    } else {
                        "".to_owned()
                    },
                )
            }
            Instr::SingleTransfer {
                kind,
                size,
                write_back,
                pre_index,
                ..
            } => (
                match kind {
                    TransferKind::Store => "STR".to_owned(),
                    TransferKind::Load => "LDR".to_owned(),
                },
                format!(
                    "{size}{}",
                    if write_back && !pre_index {
                        "T".to_owned()
                    } else {
                        "".to_owned()
                    }
                ),
            ),
            Instr::SingleTransferSpecial {
                kind,
                size,
                write_back,
                pre_index,
                ..
            } => (
                match kind {
                    TransferKind::Store => "STR".to_owned(),
                    TransferKind::Load => "LDR".to_owned(),
                },
                format!(
                    "{size}{}",
                    if write_back && !pre_index {
                        "T".to_owned()
                    } else {
                        "".to_owned()
                    }
                ),
            ),
            Instr::BlockTransfer {
                kind,
                offset_positive,
                pre_index,
                ..
            } => (
                match kind {
                    TransferKind::Store => "STM".to_owned(),
                    TransferKind::Load => "LDM".to_owned(),
                },
                match (kind, pre_index, offset_positive) {
                    (TransferKind::Store, true, true) => "FA",
                    (TransferKind::Store, true, false) => "FD",
                    (TransferKind::Store, false, true) => "EA",
                    (TransferKind::Store, false, false) => "ED",
                    (TransferKind::Load, true, true) => "ED",
                    (TransferKind::Load, true, false) => "EA",
                    (TransferKind::Load, false, true) => "FD",
                    (TransferKind::Load, false, false) => "FA",
                }
                .to_owned(),
            ),
            Instr::Swap { byte, .. } => (
                "SWP".to_owned(),
                if byte { "B".to_owned() } else { "".to_owned() },
            ),
            Instr::SoftwareInterrupt { .. } => ("SWI".to_owned(), "".to_owned()),
        };

        let args = match instr {
            Instr::BranchExchange { operand } => vec![PrettyArgument::Register {
                register: operand,
                negative: false,
                write_back: false,
            }],
            Instr::Branch { offset, .. } => {
                let absolute_address = address.wrapping_add_signed(offset).wrapping_add(8);
                vec![PrettyArgument::Constant {
                    value: absolute_address,
                    style: ConstantStyle::Address,
                }]
            }
            Instr::Data {
                op, dest, op1, op2, ..
            } => {
                let mut args = Vec::new();
                if !matches!(op, DataOp::Cmp | DataOp::Cmn | DataOp::Teq | DataOp::Tst) {
                    args.push(PrettyArgument::Register {
                        register: dest,
                        negative: false,
                        write_back: false,
                    });
                }
                if !matches!(op, DataOp::Mov | DataOp::Mvn) {
                    args.push(PrettyArgument::Register {
                        register: op1,
                        negative: false,
                        write_back: false,
                    });
                }
                args.extend(PrettyArgument::from_data_operand(op2));
                args
            }
            Instr::Mrs { psr, target } => vec![
                PrettyArgument::Psr { psr, flag: false },
                PrettyArgument::Register {
                    register: target,
                    negative: false,
                    write_back: false,
                },
            ],
            Instr::Msr { psr, source } => vec![
                PrettyArgument::Psr {
                    psr,
                    flag: matches!(source, MsrSource::RegisterFlags(_) | MsrSource::Flags(_)),
                },
                match source {
                    MsrSource::Register(register) | MsrSource::RegisterFlags(register) => {
                        PrettyArgument::Register {
                            register,
                            negative: false,
                            write_back: false,
                        }
                    }
                    MsrSource::Flags(value) => PrettyArgument::Constant {
                        value,
                        style: ConstantStyle::Unknown,
                    },
                },
            ],
            Instr::Multiply {
                dest,
                op1,
                op2,
                addend,
                ..
            } => match addend {
                Some(addend) => vec![
                    PrettyArgument::Register {
                        register: dest,
                        negative: false,
                        write_back: false,
                    },
                    PrettyArgument::Register {
                        register: op1,
                        negative: false,
                        write_back: false,
                    },
                    PrettyArgument::Register {
                        register: op2,
                        negative: false,
                        write_back: false,
                    },
                    PrettyArgument::Register {
                        register: addend,
                        negative: false,
                        write_back: false,
                    },
                ],
                None => vec![
                    PrettyArgument::Register {
                        register: dest,
                        negative: false,
                        write_back: false,
                    },
                    PrettyArgument::Register {
                        register: op1,
                        negative: false,
                        write_back: false,
                    },
                    PrettyArgument::Register {
                        register: op2,
                        negative: false,
                        write_back: false,
                    },
                ],
            },
            Instr::MultiplyLong {
                dest_hi,
                dest_lo,
                op1,
                op2,
                ..
            } => vec![
                PrettyArgument::Register {
                    register: dest_hi,
                    negative: false,
                    write_back: false,
                },
                PrettyArgument::Register {
                    register: dest_lo,
                    negative: false,
                    write_back: false,
                },
                PrettyArgument::Register {
                    register: op1,
                    negative: false,
                    write_back: false,
                },
                PrettyArgument::Register {
                    register: op2,
                    negative: false,
                    write_back: false,
                },
            ],
            Instr::SingleTransfer {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => Vec::new(),
            Instr::SingleTransferSpecial {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => Vec::new(),
            Instr::BlockTransfer {
                write_back,
                psr,
                base_register,
                registers,
                ..
            } => vec![
                PrettyArgument::Register {
                    register: base_register,
                    negative: false,
                    write_back,
                },
                PrettyArgument::RegisterSet {
                    registers: (0..16)
                        .filter(|i| (registers & (1 << i)) != 0)
                        .map(|x| Register::from_u4(x, 0))
                        .collect(),
                    caret: psr,
                },
            ],
            Instr::Swap {
                byte,
                dest,
                source,
                base,
            } => Vec::new(),
            Instr::SoftwareInterrupt { comment } => vec![PrettyArgument::Constant {
                value: comment,
                style: ConstantStyle::UnsignedDecimal,
            }],
        };

        Self {
            opcode_prefix,
            cond: cond.to_string(),
            opcode_suffix,
            args,
        }
    }
}
