use crate::instr::{Cond, DataOp, DataOperand, Instr, MsrSource, Psr, TransferKind};

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
                    write!(f, "{dest},{op1},{op2},{addend}")?;
                }
                None => {
                    write!(f, "MUL{cond}")?;
                    if *set_condition_codes {
                        write!(f, "S")?;
                    }
                    write!(f, "{dest},{op1},{op2}")?;
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
                match offset {
                    DataOperand::Constant(i) if i.value().0 == 0 => {}
                    DataOperand::Constant(i) => {
                        write!(f, ",#")?;
                        if !*offset_positive {
                            write!(f, "-")?;
                        }
                        write!(f, "{i}")?;
                    }
                    DataOperand::Register(register, shift) => {
                        write!(f, ",")?;
                        if !*offset_positive {
                            write!(f, "-")?;
                        }
                        write!(f, "{register}{shift}")?;
                    }
                }
                if *pre_index {
                    write!(f, "]")?;
                    if *write_back {
                        write!(f, "!")?;
                    }
                }
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
