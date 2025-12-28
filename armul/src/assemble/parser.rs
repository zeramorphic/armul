//! A parser for ARM assembly.

use crate::{
    assemble::{
        AssemblerError, LineError,
        syntax::{
            AsmInstr, AsmLine, AsmLineContents, DataOperand, Expression, MsrSource, Shift,
            ShiftAmount,
        },
    },
    instr::{Cond, DataOp, Psr, Register, ShiftType},
};

pub struct Parser<'a> {
    line_number: usize,
    remaining: &'a str,
}

pub type ParseResult<T> = Result<T, LineError>;

impl<'a> Parser<'a> {
    /// Only call this on fully uppercase input!
    pub fn new(src: &'a str) -> Self {
        Self {
            line_number: 1,
            remaining: src,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<AsmLine>, AssemblerError> {
        let mut result = Vec::new();
        loop {
            self.parse_whitespace_and_newlines();
            if self.remaining.is_empty() {
                return Ok(result);
            }
            let line_number = self.line_number;
            result.extend(
                self.parse_line(true)
                    .map_err(|error| AssemblerError { line_number, error })?,
            );
        }
    }

    pub fn parse_line(&mut self, allow_labels: bool) -> ParseResult<Vec<AsmLine>> {
        if let Ok(comment) = self.parse_comment() {
            if comment.is_empty() {
                return Ok(Vec::new());
            } else {
                return Ok(vec![AsmLine {
                    line_number: self.line_number,
                    contents: AsmLineContents::Empty,
                    comment,
                }]);
            }
        }

        let mnemonic = self.parse_mnemonic_or_label()?;

        if let Some(cond) = match_simple("BX", mnemonic) {
            let operand = self.parse_register()?;
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(cond, AsmInstr::BranchExchange { operand }),
                comment,
            }])
        } else if let Some((cond, link, ())) =
            match_mnemonic(&[("B", false), ("BL", true)], &[("", ())], mnemonic)
        {
            let target = self.parse_expression()?;
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(
                    cond,
                    AsmInstr::Branch {
                        link: *link,
                        target,
                    },
                ),
                comment,
            }])
        } else if let Some((cond, op, s)) = match_mnemonic(
            &[("MOV", DataOp::Mov), ("MVN", DataOp::Mvn)],
            &[("", false), ("S", true)],
            mnemonic,
        ) {
            let dest = self.parse_register()?;
            self.parse_comma()?;
            let op2 = self.parse_operand()?;
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(
                    cond,
                    AsmInstr::Data {
                        set_condition_codes: *s,
                        op: *op,
                        dest,
                        op1: Register::R0,
                        op2,
                    },
                ),
                comment,
            }])
        } else if let Some((cond, op, ())) = match_mnemonic(
            &[
                ("CMP", DataOp::Cmp),
                ("CMN", DataOp::Cmn),
                ("TEQ", DataOp::Teq),
                ("TST", DataOp::Tst),
            ],
            &[("", ())],
            mnemonic,
        ) {
            let op1 = self.parse_register()?;
            self.parse_comma()?;
            let op2 = self.parse_operand()?;
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(
                    cond,
                    AsmInstr::Data {
                        set_condition_codes: true,
                        op: *op,
                        dest: Register::R0,
                        op1,
                        op2,
                    },
                ),
                comment,
            }])
        } else if let Some((cond, op, s)) = match_mnemonic(
            &[
                ("AND", DataOp::And),
                ("EOR", DataOp::Eor),
                ("SUB", DataOp::Sub),
                ("RSB", DataOp::Rsb),
                ("ADD", DataOp::Add),
                ("ADC", DataOp::Adc),
                ("SBC", DataOp::Sbc),
                ("RSC", DataOp::Rsc),
                ("ORR", DataOp::Orr),
                ("BIC", DataOp::Bic),
            ],
            &[("", false), ("S", true)],
            mnemonic,
        ) {
            let dest = self.parse_register()?;
            self.parse_comma()?;
            let op1 = self.parse_register()?;
            self.parse_comma()?;
            let op2 = self.parse_operand()?;
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(
                    cond,
                    AsmInstr::Data {
                        set_condition_codes: *s,
                        op: *op,
                        dest,
                        op1,
                        op2,
                    },
                ),
                comment,
            }])
        } else if let Some(cond) = match_simple("MRS", mnemonic) {
            let target = self.parse_register()?;
            self.parse_comma()?;
            let psr = match self.parse_mnemonic_or_label()? {
                "CPSR" | "CPSR_ALL" => Psr::Cpsr,
                "SPSR" | "SPSR_ALL" => Psr::Spsr,
                _ => return Err(LineError::InvalidPsr),
            };
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(cond, AsmInstr::Mrs { psr, target }),
                comment,
            }])
        } else if let Some(cond) = match_simple("MSR", mnemonic) {
            let (psr, flags) = match self.parse_mnemonic_or_label()? {
                "CPSR" | "CPSR_ALL" => (Psr::Cpsr, false),
                "SPSR" | "SPSR_ALL" => (Psr::Spsr, false),
                "CPSR_FLG" => (Psr::Cpsr, true),
                "SPSR_FLG" => (Psr::Spsr, true),
                _ => return Err(LineError::InvalidPsr),
            };
            self.parse_comma()?;
            if flags {
                match self.parse_register() {
                    Ok(reg) => {
                        let comment = self.parse_comment()?;
                        Ok(vec![AsmLine {
                            line_number: self.line_number,
                            contents: AsmLineContents::Instr(
                                cond,
                                AsmInstr::Msr {
                                    psr,
                                    source: MsrSource::RegisterFlags(reg),
                                },
                            ),
                            comment,
                        }])
                    }
                    Err(_) => {
                        let exp = self.parse_expression()?;
                        let comment = self.parse_comment()?;
                        Ok(vec![AsmLine {
                            line_number: self.line_number,
                            contents: AsmLineContents::Instr(
                                cond,
                                AsmInstr::Msr {
                                    psr,
                                    source: MsrSource::Flags(exp),
                                },
                            ),
                            comment,
                        }])
                    }
                }
            } else {
                let reg = self.parse_register()?;
                let comment = self.parse_comment()?;
                Ok(vec![AsmLine {
                    line_number: self.line_number,
                    contents: AsmLineContents::Instr(
                        cond,
                        AsmInstr::Msr {
                            psr,
                            source: MsrSource::Register(reg),
                        },
                    ),
                    comment,
                }])
            }
        } else if let Some(cond) = match_simple("SWI", mnemonic) {
            self.parse_whitespace();
            let num = self.parse_number()? as u32;
            let comment = self.parse_comment()?;
            Ok(vec![AsmLine {
                line_number: self.line_number,
                contents: AsmLineContents::Instr(
                    cond,
                    AsmInstr::SoftwareInterrupt { comment: num },
                ),
                comment,
            }])
        } else if allow_labels {
            self.parse_whitespace();
            if self.parse_exact("EQU") {
                self.parse_whitespace();
                let expr = self.parse_expression()?;
                let comment = self.parse_comment()?;
                Ok(vec![AsmLine {
                    line_number: self.line_number,
                    contents: AsmLineContents::Equ(mnemonic.to_owned(), expr),
                    comment,
                }])
            } else {
                let line_number = self.line_number;
                self.parse_line(false).map(|mut x| {
                    x.insert(
                        0,
                        AsmLine {
                            line_number,
                            contents: AsmLineContents::Label(mnemonic.to_owned()),
                            comment: String::new(),
                        },
                    );
                    x
                })
            }
        } else {
            Err(LineError::UnrecognisedOpcode(mnemonic.to_owned()))
        }
    }

    fn parse_mnemonic_or_label(&mut self) -> ParseResult<&'a str> {
        self.parse_whitespace();
        let (mnemonic, tail) = match self
            .remaining
            .bytes()
            .enumerate()
            .find(|(_, b)| !b.is_ascii_alphanumeric() && ![b'-', b'_'].contains(b))
        {
            Some((i, _)) => self.remaining.split_at(i),
            None => (self.remaining, ""),
        };
        self.remaining = tail;
        if mnemonic.is_empty() {
            Err(LineError::ExpectedMnemonic(self.until_eol()))
        } else {
            Ok(mnemonic)
        }
    }

    fn parse_comment(&mut self) -> ParseResult<String> {
        self.parse_whitespace();
        if self.parse_exact(";") {
            if let Some((comment, tail)) = self.remaining.split_once('\n') {
                self.remaining = tail;
                self.line_number += 1;
                Ok(comment.to_owned())
            } else {
                let comment = self.remaining;
                self.remaining = "";
                Ok(comment.to_owned())
            }
        } else {
            // Assert we're at the end of a line.
            if self
                .remaining
                .chars()
                .next()
                .is_none_or(|x| ['\n', '\r'].contains(&x))
            {
                Ok(String::new())
            } else {
                Err(LineError::UnrecognisedAtEnd(self.until_eol()))
            }
        }
    }

    fn parse_operand(&mut self) -> ParseResult<DataOperand> {
        match self.parse_register() {
            Ok(reg) => Ok(DataOperand::Register(reg, self.parse_shift()?)),
            Err(_) => self.parse_expression().map(DataOperand::Constant),
        }
    }

    fn parse_shift(&mut self) -> ParseResult<Shift> {
        match self.parse_comma() {
            Ok(()) => {
                self.parse_whitespace();
                for (pattern, shift_type) in [
                    ("LSL", ShiftType::LogicalLeft),
                    ("ASL", ShiftType::LogicalLeft),
                    ("LSR", ShiftType::LogicalRight),
                    ("ASR", ShiftType::ArithmeticRight),
                    ("ROR", ShiftType::RotateRight),
                ] {
                    if self.parse_exact(pattern) {
                        self.parse_whitespace();
                        match self.parse_register() {
                            Ok(reg) => {
                                return Ok(Shift {
                                    shift_type,
                                    shift_amount: ShiftAmount::Register(reg),
                                });
                            }
                            Err(_) => {
                                return Ok(Shift {
                                    shift_type,
                                    shift_amount: ShiftAmount::Constant(self.parse_expression()?),
                                });
                            }
                        }
                    }
                }
                if self.parse_exact("RRX") {
                    Ok(Shift {
                        shift_type: ShiftType::RotateRightExtended,
                        shift_amount: ShiftAmount::Constant(Expression::Constant(1)),
                    })
                } else {
                    Err(LineError::ExpectedShift(self.until_eol()))
                }
            }
            Err(_) => Ok(Shift {
                shift_type: ShiftType::LogicalLeft,
                shift_amount: ShiftAmount::Constant(Expression::Constant(0)),
            }),
        }
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_expression_or()
    }

    #[allow(clippy::type_complexity)]
    fn parse_binary(
        &mut self,
        patterns: &[(
            &'static str,
            fn(Box<Expression>, Box<Expression>) -> Expression,
        )],
        lower: impl Fn(&mut Self) -> ParseResult<Expression>,
    ) -> ParseResult<Expression> {
        let lhs = lower(self)?;
        self.parse_whitespace();
        for (pattern, callback) in patterns {
            if self.parse_exact(pattern) {
                self.parse_whitespace();
                let rhs = lower(self)?;
                return Ok(callback(Box::new(lhs), Box::new(rhs)));
            }
        }
        Ok(lhs)
    }

    fn parse_expression_or(&mut self) -> ParseResult<Expression> {
        self.parse_binary(&[("OR", Expression::Or)], Self::parse_expression_shift)
    }

    fn parse_expression_shift(&mut self) -> ParseResult<Expression> {
        self.parse_binary(
            &[("LSL", Expression::Lsl), ("LSR", Expression::Lsr)],
            Self::parse_expression_atom,
        )
    }

    fn parse_expression_atom(&mut self) -> ParseResult<Expression> {
        self.parse_whitespace();
        if self.parse_exact("#") || self.remaining.starts_with(|c: char| c.is_ascii_digit()) {
            self.parse_number().map(Expression::Constant)
        } else {
            Ok(Expression::Label(
                self.parse_mnemonic_or_label()?.to_owned(),
            ))
        }
    }

    fn parse_comma(&mut self) -> ParseResult<()> {
        self.parse_whitespace();
        if self.parse_exact(",") {
            Ok(())
        } else {
            Err(LineError::ExpectedComma)
        }
    }

    /// Parses a number without the '#' prefix.
    fn parse_number(&mut self) -> ParseResult<i64> {
        if self.parse_exact("&") || self.parse_exact("0X") {
            self.parse_radix(16)
        } else if self.parse_exact("0B") {
            self.parse_radix(2)
        } else if self.parse_exact("0") {
            if self
                .remaining
                .chars()
                .next()
                .is_none_or(|x| !x.is_ascii_hexdigit())
            {
                Ok(0)
            } else {
                self.parse_radix(8)
            }
        } else {
            self.parse_radix(10)
        }
    }

    fn parse_radix(&mut self, radix: i64) -> ParseResult<i64> {
        let mut value = 0;
        let mut parsed_anything = false;
        while let Some(c) = self.remaining.chars().next() {
            let digit = if c.is_ascii_digit() {
                c as i64 - '0' as i64
            } else if ('A'..='F').contains(&c) {
                c as i64 - 'A' as i64
            } else if ('a'..='f').contains(&c) {
                c as i64 - 'a' as i64
            } else {
                break;
            };

            parsed_anything = true;
            if digit >= radix {
                return Err(LineError::AboveRadix);
            }
            value = value * radix + digit;
            self.remaining = &self.remaining[1..];
        }

        if parsed_anything {
            Ok(value)
        } else {
            Err(LineError::ExpectedNumber(self.until_eol()))
        }
    }

    fn parse_register(&mut self) -> ParseResult<Register> {
        self.parse_whitespace();
        for (s, reg) in [
            ("R0", Register::R0),
            ("R1", Register::R1),
            ("R2", Register::R2),
            ("R3", Register::R3),
            ("R4", Register::R4),
            ("R5", Register::R5),
            ("R6", Register::R6),
            ("R7", Register::R7),
            ("R8", Register::R8),
            ("R9", Register::R9),
            ("R10", Register::R10),
            ("R11", Register::R11),
            ("R12", Register::R12),
            ("R13", Register::R13),
            ("SP", Register::R13),
            ("R14", Register::R14),
            ("LR", Register::R14),
            ("R15", Register::R15),
            ("PC", Register::R15),
        ] {
            if self.parse_exact(s) {
                return Ok(reg);
            }
        }
        Err(LineError::ExpectedRegister)
    }

    fn parse_exact(&mut self, pattern: &str) -> bool {
        if let Some(new_tail) = self.remaining.strip_prefix(pattern) {
            self.remaining = new_tail;
            true
        } else {
            false
        }
    }

    fn parse_whitespace(&mut self) {
        while self
            .remaining
            .starts_with(|x: char| x.is_ascii_whitespace() && x != '\n')
        {
            self.remaining = &self.remaining[1..];
        }
    }

    fn parse_whitespace_and_newlines(&mut self) {
        while self
            .remaining
            .starts_with(|x: char| x.is_ascii_whitespace())
        {
            if self.remaining.bytes().next() == Some(b'\n') {
                self.line_number += 1;
            }
            self.remaining = &self.remaining[1..];
        }
    }

    fn until_eol(&self) -> String {
        self.remaining
            .lines()
            .next()
            .map(|x| x.to_owned())
            .unwrap_or_default()
    }
}

fn match_simple(prefix: &str, value: &str) -> Option<Cond> {
    if let Some(value) = value.strip_prefix(prefix)
        && let Ok(cond) = value.parse()
    {
        return Some(cond);
    }
    None
}

fn match_mnemonic<'a, T, U>(
    prefixes: &'a [(&str, T)],
    suffixes: &'a [(&str, U)],
    value: &str,
) -> Option<(Cond, &'a T, &'a U)> {
    for (prefix, t) in prefixes {
        if let Some(value) = value.strip_prefix(prefix) {
            for (suffix, u) in suffixes {
                if let Some(value) = value.strip_suffix(suffix)
                    && let Ok(cond) = value.parse()
                {
                    return Some((cond, t, u));
                }
            }
        }
    }
    None
}
