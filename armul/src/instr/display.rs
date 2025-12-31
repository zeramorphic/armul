use std::fmt::Display;

use crate::instr::{
    Cond, DataOp, DataOperand, Instr, MsrSource, Psr, TransferKind, TransferOperand,
};

use super::SpecialOperand;

impl Instr {
    pub fn write(&self, cond: Cond, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Instr::BranchExchange { operand } => {
                write!(f, "BX{cond} {operand}")?;
            }
            Instr::Branch { link, offset } => {
                write!(f, "B")?;
                if *link {
                    write!(f, "L")?;
                }
                write!(f, "{cond} PC+#{offset}")?;
            }
            Instr::Data {
                set_condition_codes,
                op,
                dest,
                op1,
                op2,
            } => {
                write!(f, "{op}{cond}")?;
                if *set_condition_codes
                    && !matches!(op, DataOp::Cmp | DataOp::Cmn | DataOp::Teq | DataOp::Tst)
                {
                    write!(f, "S")?;
                }
                match op {
                    DataOp::Mov | DataOp::Mvn => {
                        write!(f, " {dest}")?;
                    }
                    DataOp::Cmp | DataOp::Cmn | DataOp::Teq | DataOp::Tst => {
                        write!(f, " {op1}")?;
                    }
                    _ => {
                        write!(f, " {dest},{op1}")?;
                    }
                }
                write!(f, ",{op2}")?;
            }
            Instr::Mrs { psr, target } => {
                write!(f, "MRS{cond} {target},")?;
                match psr {
                    Psr::Cpsr => write!(f, "CPSR")?,
                    Psr::Spsr => write!(f, "SPSR")?,
                }
            }
            Instr::Msr { psr, source } => {
                write!(f, "MSR{cond} ")?;
                match psr {
                    Psr::Cpsr => write!(f, "CPSR")?,
                    Psr::Spsr => write!(f, "SPSR")?,
                }
                match source {
                    MsrSource::Register(register) => {
                        write!(f, ",{register}")?;
                    }
                    MsrSource::RegisterFlags(register) => {
                        write!(f, "_flg,{register}")?;
                    }
                    MsrSource::Flags(c) => {
                        write!(f, "_flg,#{c}")?;
                    }
                }
            }
            Instr::Multiply {
                set_condition_codes,
                dest,
                op1,
                op2,
                addend,
            } => match addend {
                Some(addend) => {
                    write!(f, "MLA{cond}")?;
                    if *set_condition_codes {
                        write!(f, "S")?;
                    }
                    write!(f, " {dest},{op1},{op2},{addend}")?;
                }
                None => {
                    write!(f, "MUL{cond}")?;
                    if *set_condition_codes {
                        write!(f, "S")?;
                    }
                    write!(f, " {dest},{op1},{op2}")?;
                }
            },
            Instr::MultiplyLong {
                set_condition_codes,
                signed,
                accumulate,
                dest_hi,
                dest_lo,
                op1,
                op2,
            } => {
                if *signed {
                    write!(f, "S")?;
                } else {
                    write!(f, "U")?;
                }
                if *accumulate {
                    write!(f, "MLAL")?;
                } else {
                    write!(f, "MULL")?;
                }
                write!(f, "{cond}")?;
                if *set_condition_codes {
                    write!(f, "S")?;
                }
                write!(f, " {dest_lo},{dest_hi},{op1},{op2}")?;
            }
            Instr::SingleTransfer {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => {
                write_single_transfer(
                    cond,
                    f,
                    kind,
                    size,
                    write_back,
                    pre_index,
                    data_register,
                    base_register,
                    match offset {
                        TransferOperand::Constant(0) => "".to_owned(),
                        TransferOperand::Constant(i) => {
                            if *offset_positive {
                                format!(",#{i}")
                            } else {
                                format!(",#-{i}")
                            }
                        }
                        TransferOperand::Register(register, shift) => {
                            if *offset_positive {
                                format!(",{register}{shift}")
                            } else {
                                format!(",-{register}{shift}")
                            }
                        }
                    },
                )?;
            }
            Instr::SingleTransferSpecial {
                kind,
                size,
                write_back,
                offset_positive,
                pre_index,
                data_register,
                base_register,
                offset,
            } => {
                write_single_transfer(
                    cond,
                    f,
                    kind,
                    size,
                    write_back,
                    pre_index,
                    data_register,
                    base_register,
                    match offset {
                        SpecialOperand::Constant(0) => "".to_owned(),
                        SpecialOperand::Constant(i) => {
                            if *offset_positive {
                                format!(",#{i}")
                            } else {
                                format!(",#-{i}")
                            }
                        }
                        SpecialOperand::Register(register) => {
                            if *offset_positive {
                                format!(",{register}")
                            } else {
                                format!(",-{register}")
                            }
                        }
                    },
                )?;
            }
            Instr::BlockTransfer {
                kind,
                write_back,
                offset_positive,
                pre_index,
                psr,
                base_register,
                registers,
            } => {
                match kind {
                    TransferKind::Store => {
                        write!(f, "STM")?;
                    }
                    TransferKind::Load => {
                        write!(f, "LDM")?;
                    }
                }
                write!(f, "{cond}")?;
                let offset = match (kind, pre_index, *offset_positive) {
                    (TransferKind::Store, true, true) => "FA",
                    (TransferKind::Store, true, false) => "FD",
                    (TransferKind::Store, false, true) => "EA",
                    (TransferKind::Store, false, false) => "ED",
                    (TransferKind::Load, true, true) => "ED",
                    (TransferKind::Load, true, false) => "EA",
                    (TransferKind::Load, false, true) => "FD",
                    (TransferKind::Load, false, false) => "FA",
                };
                write!(f, "{offset} {base_register}")?;
                if *write_back {
                    write!(f, "!")?;
                }
                write!(f, ",{{")?;
                for (ix, i) in (0..16).filter(|i| (registers & (1 << i)) != 0).enumerate() {
                    if ix != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "R{i}")?;
                }
                write!(f, "}}")?;
                if *psr {
                    write!(f, "^")?;
                }
            }
            Instr::Swap {
                byte,
                dest,
                source,
                base,
            } => {
                write!(f, "SWP{cond}")?;
                if *byte {
                    write!(f, "B")?;
                }
                write!(f, " {dest},{source},[{base}]")?;
            }
            Instr::SoftwareInterrupt { comment } => {
                write!(f, "SWI{cond} {comment}")?;
            }
        }
        Ok(())
    }

    pub fn display(&self, cond: Cond) -> String {
        let mut w = String::new();
        self.write(cond, &mut w).unwrap();
        w
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
fn write_single_transfer(
    cond: Cond,
    f: &mut impl std::fmt::Write,
    kind: &TransferKind,
    size: &impl Display,
    write_back: &bool,
    pre_index: &bool,
    data_register: &super::Register,
    base_register: &super::Register,
    offset: String,
) -> Result<(), std::fmt::Error> {
    match kind {
        TransferKind::Store => {
            write!(f, "STR")?;
        }
        TransferKind::Load => {
            write!(f, "LDR")?;
        }
    }
    write!(f, "{cond}{size}")?;
    if *write_back && !*pre_index {
        write!(f, "T")?;
    }
    write!(f, " {data_register},[{base_register}")?;
    if !*pre_index {
        write!(f, "]")?;
    }
    write!(f, "{offset}")?;
    if *pre_index {
        write!(f, "]")?;
        if *write_back {
            write!(f, "!")?;
        }
    };
    Ok(())
}
