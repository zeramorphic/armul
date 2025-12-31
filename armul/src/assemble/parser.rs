//! A parser for ARM assembly.

use std::{
    cell::Cell,
    fmt::{Debug, Display},
    rc::Rc,
};

use chumsky::{
    input::{Stream, ValueInput},
    pratt::{infix, left},
    prelude::*,
};
use logos::Logos;

use crate::{
    assemble::{
        AssemblerError, LineError,
        syntax::{
            AnyTransferSize, AsmInstr, AsmLine, AsmLineContents, DataOperand, Expression,
            MsrSource, Shift, ShiftAmount,
        },
    },
    instr::{
        Cond, DataOp, Psr, Register, ShiftType, TransferKind, TransferSize, TransferSizeSpecial,
    },
};

pub fn parse(src: &str) -> Result<Vec<AsmLine>, Vec<AssemblerError>> {
    let token_iter = Token::lexer(src).spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok.disambiguate(), span.into()),
        Err(err) => (Token::Error(err), span.into()),
    });

    let token_stream =
        Stream::from_iter(token_iter).map((0..src.len()).into(), |(t, s): (_, _)| (t, s));

    let line_indices = src
        .char_indices()
        .filter(|(_, c)| *c == '\n')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    parser(&line_indices, &Default::default())
        .parse(token_stream)
        .into_result()
        .map_err(|errs| {
            errs.into_iter()
                .map(|err| {
                    let line = line_number(&line_indices, *err.span());
                    let col = err.span().start.saturating_sub(
                        line_indices
                            .get(line - 2)
                            .copied()
                            .unwrap_or(line_indices.last().copied().unwrap_or_default()),
                    ) + 1;
                    AssemblerError {
                        line_number: line,
                        error: LineError::ParseError(format!("{line}:{col}: {err}")),
                    }
                })
                .collect()
        })
}

#[derive(Logos, Clone, PartialEq)]
#[logos(error(LexError, LexError::from_lexer))]
#[logos(subpattern numbertail = r"[_0-9a-fA-F]*")]
enum Token<'a> {
    Error(LexError),

    #[regex(r"[a-zA-Z][a-zA-Z0-9\-_]*")]
    Name(&'a str),

    Register(Register),
    Opcode((Cond, Opcode)),
    /// The bool is whether a `_flg` suffix was present.
    Psr((Psr, bool)),

    #[regex("[0-9](?&numbertail)", |lex| lex.slice().parse::<u32>())]
    Integer(u32),

    #[regex("-[0-9](?&numbertail)", |lex| lex.slice().parse::<i32>())]
    NegativeInteger(i32),

    #[regex("0[xX](?&numbertail)", |lex| u32::from_str_radix(&lex.slice()[2..], 16))]
    HexInteger(u32),

    #[regex("0[oO](?&numbertail)", |lex| u32::from_str_radix(&lex.slice()[2..], 8))]
    OctalInteger(u32),

    #[regex("0[bB](?&numbertail)", |lex| u32::from_str_radix(&lex.slice()[2..], 2))]
    BinaryInteger(u32),

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LSquare,
    #[token("]")]
    RSquare,
    #[token(",")]
    Comma,
    #[token("#")]
    Hash,
    #[token("!")]
    Exclamation,

    #[regex(r"[ \t\f]+")]
    Whitespace,

    #[regex("\n")]
    Newline,

    #[regex(r";[^\n]*", allow_greedy = true)]
    Comment(&'a str),
}

impl<'a> Token<'a> {
    fn disambiguate(self) -> Token<'a> {
        fn disambiguate_register(name: &str) -> Option<Register> {
            match name {
                "r0" => Some(Register::R0),
                "r1" => Some(Register::R1),
                "r2" => Some(Register::R2),
                "r3" => Some(Register::R3),
                "r4" => Some(Register::R4),
                "r5" => Some(Register::R5),
                "r6" => Some(Register::R6),
                "r7" => Some(Register::R7),
                "r8" => Some(Register::R8),
                "r9" => Some(Register::R9),
                "r10" => Some(Register::R10),
                "r11" => Some(Register::R11),
                "r12" => Some(Register::R12),
                "r13" => Some(Register::R13),
                "sp" => Some(Register::R13),
                "r14" => Some(Register::R14),
                "lr" => Some(Register::R14),
                "r15" => Some(Register::R15),
                "pc" => Some(Register::R15),
                _ => None,
            }
        }

        #[rustfmt::skip]
        fn disambiguate_mnemonic(name: &str) -> Option<(Cond, Opcode)> {
            for (prefix, suffix, opcode) in [
                ("bx", "", Opcode::BranchExchange),
                ("b", "", Opcode::Branch { link: false }),
                ("bl", "", Opcode::Branch { link: true }),
                ("adr", "", Opcode::Adr),
                ("adrl", "", Opcode::Adr),
                ("nop", "", Opcode::Nop),
                ("and", "", Opcode::Data(false, DataOp::And)),
                ("and", "s", Opcode::Data(true, DataOp::And)),
                ("eor", "", Opcode::Data(false, DataOp::Eor)),
                ("eor", "s", Opcode::Data(true, DataOp::Eor)),
                ("sub", "", Opcode::Data(false, DataOp::Sub)),
                ("sub", "s", Opcode::Data(true, DataOp::Sub)),
                ("rsb", "", Opcode::Data(false, DataOp::Rsb)),
                ("rsb", "s", Opcode::Data(true, DataOp::Rsb)),
                ("add", "", Opcode::Data(false, DataOp::Add)),
                ("add", "s", Opcode::Data(true, DataOp::Add)),
                ("adc", "", Opcode::Data(false, DataOp::Adc)),
                ("adc", "s", Opcode::Data(true, DataOp::Adc)),
                ("sbc", "", Opcode::Data(false, DataOp::Sbc)),
                ("sbc", "s", Opcode::Data(true, DataOp::Sbc)),
                ("rsc", "", Opcode::Data(false, DataOp::Rsc)),
                ("rsc", "s", Opcode::Data(true, DataOp::Rsc)),
                ("tst", "", Opcode::Data(true, DataOp::Tst)),
                ("teq", "", Opcode::Data(true, DataOp::Teq)),
                ("cmp", "", Opcode::Data(true, DataOp::Cmp)),
                ("cmn", "", Opcode::Data(true, DataOp::Cmn)),
                ("orr", "", Opcode::Data(false, DataOp::Orr)),
                ("orr", "s", Opcode::Data(true, DataOp::Orr)),
                ("mov", "", Opcode::Data(false, DataOp::Mov)),
                ("mov", "s", Opcode::Data(true, DataOp::Mov)),
                ("bic", "", Opcode::Data(false, DataOp::Bic)),
                ("bic", "s", Opcode::Data(true, DataOp::Bic)),
                ("mvn", "", Opcode::Data(false, DataOp::Mvn)),
                ("mvn", "s", Opcode::Data(true, DataOp::Mvn)),
                ("lsl", "", Opcode::Shift(false, ShiftType::LogicalLeft)),
                ("lsl", "s", Opcode::Shift(true, ShiftType::LogicalLeft)),
                ("asl", "", Opcode::Shift(false, ShiftType::LogicalLeft)),
                ("asl", "s", Opcode::Shift(true, ShiftType::LogicalLeft)),
                ("lsr", "", Opcode::Shift(false, ShiftType::LogicalRight)),
                ("lsr", "s", Opcode::Shift(true, ShiftType::LogicalRight)),
                ("asr", "", Opcode::Shift(false, ShiftType::ArithmeticRight)),
                ("asr", "s", Opcode::Shift(true, ShiftType::ArithmeticRight)),
                ("ror", "", Opcode::Shift(false, ShiftType::RotateRight)),
                ("ror", "s", Opcode::Shift(true, ShiftType::RotateRight)),
                ("rrx", "", Opcode::Shift(false, ShiftType::RotateRightExtended)),
                ("rrx", "s", Opcode::Shift(true, ShiftType::RotateRightExtended)),
                ("mrs", "", Opcode::Mrs),
                ("msr", "", Opcode::Msr),
                ("mul", "", Opcode::Mul(false, false)),
                ("mul", "s", Opcode::Mul(true, false)),
                ("mla", "", Opcode::Mul(false, true)),
                ("mla", "s", Opcode::Mul(true, true)),
                ("umull", "", Opcode::MulLong(false, false, false)),
                ("umull", "s", Opcode::MulLong(true, false, false)),
                ("umlal", "", Opcode::MulLong(false, false, true)),
                ("umlal", "s", Opcode::MulLong(true, false, true)),
                ("smull", "", Opcode::MulLong(false, true, false)),
                ("smull", "s", Opcode::MulLong(true, true, false)),
                ("smlal", "", Opcode::MulLong(false, true, true)),
                ("smlal", "s", Opcode::MulLong(true, true, true)),
                ("ldr", "", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Normal(TransferSize::Word), false)),
                ("ldr", "b", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Normal(TransferSize::Byte), false)),
                ("ldr", "t", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Normal(TransferSize::Word), true)),
                ("ldr", "bt", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Normal(TransferSize::Byte), true)),
                ("ldr", "h", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Special(TransferSizeSpecial::HalfWord), false)),
                ("ldr", "sh", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Special(TransferSizeSpecial::SignExtendedHalfWord), false)),
                ("ldr", "sb", Opcode::SingleTransfer(TransferKind::Load, AnyTransferSize::Special(TransferSizeSpecial::SignExtendedByte), false)),
                ("str", "", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Normal(TransferSize::Word), false)),
                ("str", "b", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Normal(TransferSize::Byte), false)),
                ("str", "t", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Normal(TransferSize::Word), true)),
                ("str", "bt", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Normal(TransferSize::Byte), true)),
                ("str", "h", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Special(TransferSizeSpecial::HalfWord), false)),
                ("str", "sh", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Special(TransferSizeSpecial::SignExtendedHalfWord), false)),
                ("str", "sb", Opcode::SingleTransfer(TransferKind::Store, AnyTransferSize::Special(TransferSizeSpecial::SignExtendedByte), false)),
                ("ldm", "fd", Opcode::BlockTransfer(TransferKind::Load, true, false)),
                ("ldm", "ed", Opcode::BlockTransfer(TransferKind::Load, true, true)),
                ("ldm", "fa", Opcode::BlockTransfer(TransferKind::Load, false, false)),
                ("ldm", "ea", Opcode::BlockTransfer(TransferKind::Load, false, true)),
                ("ldm", "ia", Opcode::BlockTransfer(TransferKind::Load, true, false)),
                ("ldm", "ib", Opcode::BlockTransfer(TransferKind::Load, true, true)),
                ("ldm", "da", Opcode::BlockTransfer(TransferKind::Load, false, false)),
                ("ldm", "db", Opcode::BlockTransfer(TransferKind::Load, false, true)),
                ("stm", "fd", Opcode::BlockTransfer(TransferKind::Store, true, false)),
                ("stm", "ed", Opcode::BlockTransfer(TransferKind::Store, true, true)),
                ("stm", "fa", Opcode::BlockTransfer(TransferKind::Store, false, false)),
                ("stm", "ea", Opcode::BlockTransfer(TransferKind::Store, false, true)),
                ("stm", "ia", Opcode::BlockTransfer(TransferKind::Store, true, false)),
                ("stm", "ib", Opcode::BlockTransfer(TransferKind::Store, true, true)),
                ("stm", "da", Opcode::BlockTransfer(TransferKind::Store, false, false)),
                ("stm", "db", Opcode::BlockTransfer(TransferKind::Store, false, true)),
                ("swp", "", Opcode::Swap(false)),
                ("swp", "b", Opcode::Swap(true)),
                ("swi", "", Opcode::Swi),
                ("equ", "", Opcode::Equ),
                ("dw", "", Opcode::DefW),
                ("defw", "", Opcode::DefW),
            ] {
                if let Some(tail) = name.strip_prefix(prefix)
                    && let Some(cond) = tail.strip_suffix(suffix)
                    && let Ok(cond) = cond.parse()
                {
                    return Some((cond, opcode));
                }
            }
            None
        }

        fn disambiguate_psr(name: &str) -> Option<(Psr, bool)> {
            match name {
                "cpsr" | "cpsr_all" => Some((Psr::Cpsr, false)),
                "cpsr_flg" => Some((Psr::Cpsr, true)),
                "spsr" | "spsr_all" => Some((Psr::Spsr, false)),
                "spsr_flg" => Some((Psr::Spsr, true)),
                _ => None,
            }
        }

        match self {
            Token::Name(name) => {
                let lower = name.to_lowercase();
                if let Some(reg) = disambiguate_register(&lower) {
                    Token::Register(reg)
                } else if let Some(mnemonic) = disambiguate_mnemonic(&lower) {
                    Token::Opcode(mnemonic)
                } else if let Some(value) = disambiguate_psr(&lower) {
                    Token::Psr(value)
                } else {
                    self
                }
            }
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Opcode {
    BranchExchange,
    Branch {
        link: bool,
    },
    Adr,
    Nop,
    Data(bool, DataOp),
    Shift(bool, ShiftType),
    Mrs,
    Msr,
    /// Set condition codes; accumulate.
    Mul(bool, bool),
    /// Set condition codes; signed; accumulate.
    MulLong(bool, bool, bool),
    /// The bool is for forced writeback (the T flag).
    SingleTransfer(TransferKind, AnyTransferSize, bool),
    /// The bool flags are positive offset and pre index.
    BlockTransfer(TransferKind, bool, bool),
    /// The bool is whether to swap a byte.
    Swap(bool),
    Swi,
    Equ,
    DefW,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Opcode::BranchExchange => write!(f, "BX"),
            Opcode::Branch { link: false } => write!(f, "B"),
            Opcode::Branch { link: true } => write!(f, "BL"),
            Opcode::Data(set_condition_codes, op) => {
                write!(f, "{op}")?;
                if set_condition_codes {
                    write!(f, "S")
                } else {
                    Ok(())
                }
            }
            Opcode::Shift(set_condition_codes, shift_type) => {
                write!(f, "{shift_type}")?;
                if set_condition_codes {
                    write!(f, "S")
                } else {
                    Ok(())
                }
            }
            Opcode::Adr => write!(f, "ADR"),
            Opcode::Nop => write!(f, "NOP"),
            Opcode::Mrs => write!(f, "MRS"),
            Opcode::Msr => write!(f, "MSR"),
            Opcode::Mul(set_condition_codes, accumulate) => {
                if accumulate {
                    write!(f, "MLA")?;
                } else {
                    write!(f, "MUL")?;
                }
                if set_condition_codes {
                    write!(f, "S")
                } else {
                    Ok(())
                }
            }
            Opcode::MulLong(set_condition_codes, signed, accumulate) => {
                if signed {
                    write!(f, "S")?;
                } else {
                    write!(f, "U")?;
                }
                if accumulate {
                    write!(f, "MLA")?;
                } else {
                    write!(f, "MUL")?;
                }
                if set_condition_codes {
                    write!(f, "S")
                } else {
                    Ok(())
                }
            }
            Opcode::SingleTransfer(transfer_kind, transfer_size, t) => {
                match transfer_kind {
                    TransferKind::Store => write!(f, "STR")?,
                    TransferKind::Load => write!(f, "LDR")?,
                };
                write!(f, "{transfer_size}")?;
                if t { write!(f, "T") } else { Ok(()) }
            }
            Opcode::BlockTransfer(transfer_kind, pos_offset, pre_index) => {
                match transfer_kind {
                    TransferKind::Store => write!(f, "STM")?,
                    TransferKind::Load => write!(f, "LDM")?,
                };
                if pre_index {
                    write!(f, "E")?;
                } else {
                    write!(f, "F")?;
                }
                if pos_offset {
                    write!(f, "D")
                } else {
                    write!(f, "A")
                }
            }
            Opcode::Swap(byte) => {
                write!(f, "SWP")?;
                if byte { write!(f, "B") } else { Ok(()) }
            }
            Opcode::Swi => write!(f, "SWI"),
            Opcode::Equ => write!(f, "EQU"),
            Opcode::DefW => write!(f, "DEFW"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
enum LexError {
    Error(String),
    #[default]
    Other,
}

/// Error type returned by calling `lex.slice().parse()` to u8.
impl From<std::num::ParseIntError> for LexError {
    fn from(err: std::num::ParseIntError) -> Self {
        use std::num::IntErrorKind::*;
        match err.kind() {
            PosOverflow | NegOverflow => LexError::Error("overflow error".to_owned()),
            _ => LexError::Error("invalid integer".to_owned()),
        }
    }
}

impl LexError {
    fn from_lexer<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Self {
        LexError::Error(format!(
            "invalid character {:?}",
            lex.slice().chars().next().unwrap()
        ))
    }
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::Error(s) => write!(f, "{s}"),
            LexError::Other => write!(f, "error"),
        }
    }
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Error(err) => write!(f, "{err}"),
            Token::Integer(i) => write!(f, "{i}"),
            Token::NegativeInteger(i) => write!(f, "{i}"),
            Token::HexInteger(i) => write!(f, "{i}"),
            Token::OctalInteger(i) => write!(f, "{i}"),
            Token::BinaryInteger(i) => write!(f, "{i}"),
            Token::Name(x) => write!(f, "{x}"),
            Token::Register(register) => write!(f, "{register}"),
            Token::Opcode((cond, opcode)) => write!(f, "{opcode} ({cond})"),
            Token::Psr((psr, false)) => write!(f, "{psr}"),
            Token::Psr((psr, true)) => write!(f, "{psr}_flg"),
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LSquare => write!(f, "["),
            Token::RSquare => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Hash => write!(f, "#"),
            Token::Exclamation => write!(f, "!"),
            Token::Whitespace => write!(f, "whitespace"),
            Token::Newline => write!(f, "newline"),
            Token::Comment(_) => write!(f, "comment"),
        }
    }
}

fn line_number(line_indices: &[usize], span: SimpleSpan) -> usize {
    line_indices
        .binary_search(&span.start)
        .unwrap_or_else(|x| x)
        + 1
}

#[derive(Default, Clone, Copy)]
struct LabelGenerator(u32);

fn generate_label(generator: &Rc<Cell<LabelGenerator>>) -> String {
    let index = generator.get().0;
    generator.set(LabelGenerator(index + 1));
    format!("__generatedlabel_{index}")
}

fn parser<'tokens, 'src: 'tokens, I>(
    line_indices: &[usize],
    generator: &Rc<Cell<LabelGenerator>>,
) -> impl Parser<'tokens, I, Vec<AsmLine>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    line_contents(generator)
        .or_not()
        .map(|x| x.unwrap_or_default())
        .spanned()
        .then(select! { Token::Comment(x) => x }.or_not())
        .then_ignore(just(Token::Newline).repeated().at_least(1))
        .map(
            |(contents, mut comment): (Spanned<Vec<AsmLineContents>>, Option<&str>)| {
                let line_number = line_number(line_indices, contents.span);
                if contents.is_empty() {
                    vec![AsmLine {
                        line_number,
                        contents: AsmLineContents::Empty,
                        comment: comment.unwrap_or_default().to_owned(),
                    }]
                } else {
                    contents
                        .inner
                        .into_iter()
                        .map(|contents| AsmLine {
                            line_number,
                            contents,
                            comment: comment.take().unwrap_or_default().to_owned(),
                        })
                        .collect()
                }
            },
        )
        .repeated()
        .collect::<Vec<_>>()
        .map(|x| x.into_iter().flatten().collect())
}

fn line_contents<'tokens, 'src: 'tokens, I>(
    generator: &Rc<Cell<LabelGenerator>>,
) -> impl Parser<'tokens, I, Vec<AsmLineContents>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    let label = select! { Token::Name(label) => label };
    let mnemonic = select! { Token::Opcode(mnemonic) => mnemonic };
    let args = argument()
        .separated_by(
            whitespace()
                .then_ignore(just(Token::Comma))
                .then_ignore(whitespace()),
        )
        .collect::<Vec<_>>();

    label
        .or_not()
        .then_ignore(whitespace())
        .then((mnemonic.then_ignore(whitespace()).then(args)).or_not())
        .try_map(|(label, instr), span| match instr {
            Some(((cond, opcode), args)) => process_instruction(opcode, args, span, generator)
                .map(|instr| (label, Some((cond, instr)))),
            None => Ok((label, None)),
        })
        .try_map(|(label, instr), span| process_line_contents(label, instr, span))
}

fn process_line_contents(
    label: Option<&str>,
    instr: Option<(Cond, Processed)>,
    span: SimpleSpan,
) -> Result<Vec<AsmLineContents>, Rich<'_, Token<'_>>> {
    match (label, instr) {
        (None, None) => Ok(Vec::new()),
        (label, Some((cond, Processed::Instr(instr)))) => {
            let mut result = Vec::new();
            if let Some(label) = label {
                result.push(AsmLineContents::Label(label.to_owned()))
            }
            result.push(AsmLineContents::Instr(cond, instr));
            Ok(result)
        }
        (label, Some((cond, Processed::DefW(expr)))) => {
            let mut result = Vec::new();
            if let Some(label) = label {
                result.push(AsmLineContents::Label(label.to_owned()))
            }
            if cond != Cond::AL {
                return Err(Rich::custom(span, "'defw' cannot have a condition flag"));
            }
            result.push(AsmLineContents::DefWord(expr));
            Ok(result)
        }
        (None, Some((_, Processed::Equ(_)))) => Err(Rich::custom(span, "'equ' needs a label")),
        (Some(label), Some((cond, Processed::Equ(expr)))) => {
            if cond != Cond::AL {
                return Err(Rich::custom(span, "'equ' cannot have a condition flag"));
            }
            Ok(vec![AsmLineContents::Equ(label.to_owned(), expr)])
        }
        (mut label, Some((cond, Processed::Vec(items)))) => {
            let mut result = Vec::new();
            for item in items {
                result.extend(process_line_contents(
                    label.take(),
                    Some((cond, item)),
                    span,
                )?);
            }
            if let Some(label) = label {
                result.push(AsmLineContents::Label(label.to_owned()))
            }
            Ok(result)
        }
        (label, Some((_, Processed::Label(second_label)))) => {
            let mut result = Vec::new();
            if let Some(label) = label {
                result.push(AsmLineContents::Label(label.to_owned()))
            }
            result.push(AsmLineContents::Label(second_label.to_owned()));
            Ok(result)
        }
        (Some(label), None) => Ok(vec![AsmLineContents::Label(label.to_owned())]),
    }
}

#[derive(Debug)]
enum Argument {
    Register(Register),
    /// The bool is whether the sign was positive.
    SignedRegister(bool, Register),
    Psr {
        psr: Psr,
        flag: bool,
    },
    Shift(Shift),
    Expression(Expression),
    /// `[Rd{,operand}*]{!}`
    Address {
        base_register: Register,
        operands: Vec<Argument>,
        write_back: bool,
    },
}

fn argument<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Argument, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|arg| {
        choice((
            register().map(Argument::Register),
            custom(|inp| {
                let checkpoint = inp.save();
                let sign = match inp.next() {
                    Some(Token::Sub) => Some(false),
                    Some(Token::Add) => Some(true),
                    _ => None,
                };
                if let Some(sign) = sign
                    && let Some(Token::Register(reg)) = inp.next()
                {
                    return Ok(Argument::SignedRegister(sign, reg));
                }
                let span = inp.span_since(checkpoint.cursor());
                inp.rewind(checkpoint);
                Err(Rich::custom(span, "expected signed register"))
            }),
            shift().map(Argument::Shift),
            expression().map(Argument::Expression),
            select! {
                Token::Psr((psr, flag)) => (psr, flag)
            }
            .map(|(psr, flag)| Argument::Psr { psr, flag }),
            just(Token::LSquare)
                .ignore_then(register())
                .then(
                    just(Token::Comma)
                        .ignore_then(
                            arg.padded_by(whitespace())
                                .separated_by(just(Token::Comma))
                                .collect::<Vec<_>>(),
                        )
                        .or_not(),
                )
                .then_ignore(just(Token::RSquare))
                .then(just(Token::Exclamation).or_not().map(|x| x.is_some()))
                .map(|((base, operands), write_back)| Argument::Address {
                    base_register: base,
                    operands: operands.unwrap_or_default(),
                    write_back,
                }),
        ))
    })
}

fn shift<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Shift, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Opcode((
        Cond::AL,
        Opcode::Shift(false, ShiftType::RotateRightExtended),
    )))
    .map(|_| Shift {
        shift_type: ShiftType::RotateRightExtended,
        shift_amount: ShiftAmount::Constant(Expression::Constant(1)),
    })
    .or(
        select! { Token::Opcode((Cond::AL, Opcode::Shift(false, t))) if t != ShiftType::RotateRightExtended => t }
            .then_ignore(whitespace().or_not())
            .then(choice((
                register().map(ShiftAmount::Register),
                expression().map(ShiftAmount::Constant),
            )))
            .map(|(shift_type, shift_amount)| Shift {
                shift_type,
                shift_amount,
            }),
    )
}

fn register<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Register, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token::Register(r) => r
    }
}

fn expression<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expression, extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    recursive(|e| {
        let number = select! {
            Token::Integer(i) => Expression::Constant(i),
            Token::NegativeInteger(i) => Expression::Constant(i as u32),
            Token::HexInteger(i) => Expression::Constant(i),
            Token::OctalInteger(i) => Expression::Constant(i),
            Token::BinaryInteger(i) => Expression::Constant(i),
        };
        let atom = choice((
            just(Token::Hash).or_not().ignore_then(number),
            select! { Token::Name(name) => Expression::Label(name.to_owned()) },
            just(Token::LParen)
                .ignore_then(e)
                .then_ignore(just(Token::RParen)),
        ));
        atom.padded_by(whitespace().or_not()).pratt((
            infix(left(3), just(Token::Mul), |l, _, r, _| {
                Expression::Mul(Box::new(l), Box::new(r))
            }),
            infix(left(3), just(Token::Div), |l, _, r, _| {
                Expression::Div(Box::new(l), Box::new(r))
            }),
            infix(left(4), just(Token::Add), |l, _, r, _| {
                Expression::Add(Box::new(l), Box::new(r))
            }),
            infix(left(4), just(Token::Sub), |l, _, r, _| {
                Expression::Sub(Box::new(l), Box::new(r))
            }),
            infix(
                left(5),
                select! { Token::Opcode((Cond::AL, Opcode::Shift(false, s))) if s != ShiftType::RotateRightExtended => s },
                |l, s, r, _| match s {
                    ShiftType::LogicalLeft => Expression::Lsl(Box::new(l), Box::new(r)),
                    ShiftType::LogicalRight => Expression::Lsr(Box::new(l), Box::new(r)),
                    ShiftType::ArithmeticRight => Expression::Asr(Box::new(l), Box::new(r)),
                    ShiftType::RotateRight => Expression::Ror(Box::new(l), Box::new(r)),
                    _ => unreachable!(),
                },
            ),
            infix(left(10), just(Token::Name("or")), |l, _, r, _| {
                Expression::Or(Box::new(l), Box::new(r))
            }),
        ))
    })
}

fn whitespace<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (), extra::Err<Rich<'tokens, Token<'src>>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    just(Token::Whitespace).or_not().ignored()
}

enum Processed {
    Label(String),
    Instr(AsmInstr),
    Equ(Expression),
    DefW(Expression),
    Vec(Vec<Processed>),
}

fn process_instruction<'tokens, 'src: 'tokens>(
    opcode: Opcode,
    mut args: Vec<Argument>,
    span: SimpleSpan,
    generator: &Rc<Cell<LabelGenerator>>,
) -> Result<Processed, Rich<'tokens, Token<'src>>> {
    match opcode {
        Opcode::BranchExchange => {
            let [operand] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match operand {
                Argument::Register(operand) => {
                    Ok(Processed::Instr(AsmInstr::BranchExchange { operand }))
                }
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <offset>"))),
            }
        }
        Opcode::Branch { link } => {
            let [target] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match target {
                Argument::Expression(target) => {
                    Ok(Processed::Instr(AsmInstr::Branch { link, target }))
                }
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <offset>"))),
            }
        }
        Opcode::Adr => {
            let [dest, expr] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
            match (dest, expr) {
                (Argument::Register(dest), Argument::Expression(expr)) => {
                    Ok(Processed::Instr(AsmInstr::Adr {
                        long: false,
                        dest,
                        expr,
                    }))
                }
                _ => Err(Rich::custom(
                    span,
                    format!("syntax: {opcode} Rd,<expression>"),
                )),
            }
        }
        Opcode::Nop => {
            let [] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 0 arguments"))?;
            // A simple NOP instruction.
            Ok(Processed::Instr(AsmInstr::Data {
                set_condition_codes: false,
                op: DataOp::Mov,
                dest: Register::R8,
                op1: Register::R0,
                op2: DataOperand::Register(Register::R8, Shift::default()),
            }))
        }
        Opcode::Data(set_condition_codes, op) => {
            let last_operand = args
                .pop()
                .ok_or_else(|| Rich::custom(span, "expected operands"))?;
            let op2 = match last_operand {
                Argument::Register(register) => DataOperand::Register(register, Shift::default()),
                Argument::Shift(shift) => match args.pop() {
                    Some(Argument::Register(reg)) => DataOperand::Register(reg, shift),
                    _ => return Err(Rich::custom(span, "shift must follow register")),
                },
                Argument::Expression(expression) => DataOperand::Constant(expression),
                _ => {
                    return Err(Rich::custom(
                        span,
                        "unexpected operand to data processing instruction",
                    ));
                }
            };
            match op.kind() {
                DataOpKind::NoDest => {
                    let [op1] = args
                        .try_into()
                        .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
                    match op1 {
                        Argument::Register(op1) => Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op,
                            dest: Register::R0,
                            op1,
                            op2,
                        })),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rn,<Op2>"))),
                    }
                }
                DataOpKind::NoOp1 => {
                    let [dest] = args
                        .try_into()
                        .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
                    match dest {
                        Argument::Register(dest) => Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op,
                            dest,
                            op1: Register::R0,
                            op2,
                        })),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<Op2>"))),
                    }
                }
                DataOpKind::ThreeArg => match TryInto::<[Argument; 1]>::try_into(args) {
                    Ok([op1]) => match op1 {
                        Argument::Register(op1) => Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op,
                            dest: op1,
                            op1,
                            op2,
                        })),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<Op2>"))),
                    },
                    Err(args) => {
                        let [dest, op1] = args
                            .try_into()
                            .map_err(|_| Rich::custom(span, "expected 3 arguments"))?;
                        match (dest, op1) {
                            (Argument::Register(dest), Argument::Register(op1)) => {
                                Ok(Processed::Instr(AsmInstr::Data {
                                    set_condition_codes,
                                    op,
                                    dest,
                                    op1,
                                    op2,
                                }))
                            }
                            _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rn,<Op2>"))),
                        }
                    }
                },
            }
        }
        Opcode::Shift(set_condition_codes, ShiftType::RotateRightExtended) => {
            match TryInto::<[Argument; 1]>::try_into(args) {
                Ok([op1]) => match op1 {
                    Argument::Register(op1) => Ok(Processed::Instr(AsmInstr::Data {
                        set_condition_codes,
                        op: DataOp::Mov,
                        dest: op1,
                        op1: Register::R0,
                        op2: DataOperand::Register(
                            op1,
                            Shift {
                                shift_type: ShiftType::RotateRightExtended,
                                shift_amount: ShiftAmount::Constant(Expression::Constant(1)),
                            },
                        ),
                    })),
                    _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd"))),
                },
                Err(args) => {
                    let [dest, op1] = args
                        .try_into()
                        .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
                    match (dest, op1) {
                        (Argument::Register(dest), Argument::Register(op1)) => {
                            Ok(Processed::Instr(AsmInstr::Data {
                                set_condition_codes,
                                op: DataOp::Mov,
                                dest,
                                op1: Register::R0,
                                op2: DataOperand::Register(
                                    op1,
                                    Shift {
                                        shift_type: ShiftType::RotateRightExtended,
                                        shift_amount: ShiftAmount::Constant(Expression::Constant(
                                            1,
                                        )),
                                    },
                                ),
                            }))
                        }
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rn"))),
                    }
                }
            }
        }
        Opcode::Shift(set_condition_codes, shift_type) => {
            match TryInto::<[Argument; 2]>::try_into(args) {
                Ok([op1, shift]) => match (op1, shift) {
                    (Argument::Register(op1), Argument::Expression(shift_amount)) => {
                        Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op: DataOp::Mov,
                            dest: op1,
                            op1: Register::R0,
                            op2: DataOperand::Register(
                                op1,
                                Shift {
                                    shift_type,
                                    shift_amount: ShiftAmount::Constant(shift_amount),
                                },
                            ),
                        }))
                    }
                    (Argument::Register(op1), Argument::Register(shift_amount)) => {
                        Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op: DataOp::Mov,
                            dest: op1,
                            op1: Register::R0,
                            op2: DataOperand::Register(
                                op1,
                                Shift {
                                    shift_type,
                                    shift_amount: ShiftAmount::Register(shift_amount),
                                },
                            ),
                        }))
                    }
                    _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<Op2>"))),
                },
                Err(args) => {
                    let [dest, op1, shift] = args
                        .try_into()
                        .map_err(|_| Rich::custom(span, "expected 3 arguments"))?;
                    match (dest, op1, shift) {
                        (
                            Argument::Register(dest),
                            Argument::Register(op1),
                            Argument::Expression(shift_amount),
                        ) => Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op: DataOp::Mov,
                            dest,
                            op1: Register::R0,
                            op2: DataOperand::Register(
                                op1,
                                Shift {
                                    shift_type,
                                    shift_amount: ShiftAmount::Constant(shift_amount),
                                },
                            ),
                        })),
                        (
                            Argument::Register(dest),
                            Argument::Register(op1),
                            Argument::Register(shift_amount),
                        ) => Ok(Processed::Instr(AsmInstr::Data {
                            set_condition_codes,
                            op: DataOp::Mov,
                            dest,
                            op1: Register::R0,
                            op2: DataOperand::Register(
                                op1,
                                Shift {
                                    shift_type,
                                    shift_amount: ShiftAmount::Register(shift_amount),
                                },
                            ),
                        })),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rn,<Op2>"))),
                    }
                }
            }
        }
        Opcode::Mrs => {
            let [target, psr] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
            match (target, psr) {
                (Argument::Register(target), Argument::Psr { psr, flag: false }) => {
                    Ok(Processed::Instr(AsmInstr::Mrs { psr, target }))
                }
                _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<psr>"))),
            }
        }
        Opcode::Msr => {
            let [psr, op] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
            match (psr, op) {
                (Argument::Psr { psr, flag: false }, Argument::Register(reg)) => {
                    Ok(Processed::Instr(AsmInstr::Msr {
                        psr,
                        source: MsrSource::Register(reg),
                    }))
                }
                (Argument::Psr { psr, flag: true }, Argument::Register(reg)) => {
                    Ok(Processed::Instr(AsmInstr::Msr {
                        psr,
                        source: MsrSource::RegisterFlags(reg),
                    }))
                }
                (Argument::Psr { psr, flag: true }, Argument::Expression(expr)) => {
                    Ok(Processed::Instr(AsmInstr::Msr {
                        psr,
                        source: MsrSource::Flags(expr),
                    }))
                }
                _ => Err(Rich::custom(
                    span,
                    format!("syntax: {opcode} <psr>,Rm (for example)"),
                )),
            }
        }
        Opcode::Mul(set_condition_codes, false) => {
            let [dest, op1, op2] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 3 arguments"))?;
            match (dest, op1, op2) {
                (Argument::Register(dest), Argument::Register(op1), Argument::Register(op2)) => {
                    Ok(Processed::Instr(AsmInstr::Multiply {
                        set_condition_codes,
                        dest,
                        op1,
                        op2,
                        addend: None,
                    }))
                }
                _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rm,Rs"))),
            }
        }
        Opcode::Mul(set_condition_codes, true) => {
            let [dest, op1, op2, addend] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 4 arguments"))?;
            match (dest, op1, op2, addend) {
                (
                    Argument::Register(dest),
                    Argument::Register(op1),
                    Argument::Register(op2),
                    Argument::Register(addend),
                ) => Ok(Processed::Instr(AsmInstr::Multiply {
                    set_condition_codes,
                    dest,
                    op1,
                    op2,
                    addend: Some(addend),
                })),
                _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rm,Rs,Rn"))),
            }
        }
        Opcode::MulLong(set_condition_codes, signed, accumulate) => {
            let [dest_lo, dest_hi, op1, op2] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 4 arguments"))?;
            match (dest_lo, dest_hi, op1, op2) {
                (
                    Argument::Register(dest_lo),
                    Argument::Register(dest_hi),
                    Argument::Register(op1),
                    Argument::Register(op2),
                ) => Ok(Processed::Instr(AsmInstr::MultiplyLong {
                    set_condition_codes,
                    signed,
                    accumulate,
                    dest_lo,
                    dest_hi,
                    op1,
                    op2,
                })),
                _ => Err(Rich::custom(
                    span,
                    format!("syntax: {opcode} RdLo,RdHi,Rm,Rs"),
                )),
            }
        }
        Opcode::SingleTransfer(kind, size, t_flag) => match args.len() {
            2 => {
                let [data_register, addr] = args.try_into().unwrap();
                match (data_register, addr) {
                    (Argument::Register(data_register), Argument::Expression(addr)) => {
                        if t_flag {
                            Err(Rich::custom(
                                span,
                                "T flag not permitted with expression address",
                            ))
                        } else {
                            // Work out an offset to the given address,
                            // or rather, make the assembler do the calculation shortly.
                            // Because we might generate extra healing instructions between
                            // the start and the end of execution, we put the label *after*
                            // the PC location it's referencing.
                            let here = generate_label(generator);
                            Ok(Processed::Vec(vec![
                                Processed::Instr(AsmInstr::SingleTransfer {
                                    kind,
                                    size,
                                    write_back: false,
                                    offset_positive: true,
                                    pre_index: true,
                                    data_register,
                                    base_register: Register::R15,
                                    offset: DataOperand::Constant(Expression::Sub(
                                        Box::new(addr),
                                        Box::new(Expression::Add(
                                            Box::new(Expression::Label(here.clone())),
                                            Box::new(Expression::Constant(4)),
                                        )),
                                    )),
                                }),
                                Processed::Label(here),
                            ]))
                        }
                    }
                    (
                        Argument::Register(data_register),
                        Argument::Address {
                            base_register,
                            operands,
                            write_back,
                        },
                    ) => {
                        // This is a pre-indexed addressing specification.
                        if t_flag {
                            Err(Rich::custom(
                                span,
                                "T flag not permitted with pre-indexed address",
                            ))
                        } else {
                            let (offset_positive, offset) = match operands.len() {
                                0 => (true, DataOperand::Constant(Expression::Constant(0))),
                                1 => {
                                    let [operand] = operands.try_into().unwrap();
                                    match operand {
                                        Argument::Expression(expr) => {
                                            (true, DataOperand::Constant(expr))
                                        }
                                        Argument::Register(reg) => {
                                            (true, DataOperand::Register(reg, Shift::default()))
                                        }
                                        Argument::SignedRegister(sign, reg) => {
                                            (sign, DataOperand::Register(reg, Shift::default()))
                                        }
                                        _ => {
                                            return Err(Rich::custom(
                                                span,
                                                "expected expression or register as offset",
                                            ));
                                        }
                                    }
                                }
                                2 => {
                                    let [register, shift] = operands.try_into().unwrap();
                                    match (register, shift) {
                                        (Argument::Register(reg), Argument::Shift(shift)) => {
                                            (true, DataOperand::Register(reg, shift))
                                        }
                                        (
                                            Argument::SignedRegister(sign, reg),
                                            Argument::Shift(shift),
                                        ) => (sign, DataOperand::Register(reg, shift)),
                                        _ => {
                                            return Err(Rich::custom(
                                                span,
                                                "expected register followed by a shift",
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(Rich::custom(
                                        span,
                                        "too many operands inside addressing specification",
                                    ));
                                }
                            };
                            Ok(Processed::Instr(AsmInstr::SingleTransfer {
                                kind,
                                size,
                                write_back,
                                offset_positive,
                                pre_index: true,
                                data_register,
                                base_register,
                                offset,
                            }))
                        }
                    }
                    _ => Err(Rich::custom(
                        span,
                        "expected register then either an expression or address",
                    )),
                }
            }
            3 | 4 => {
                // This is a post-indexed addressing specification.
                let shift = if args.len() == 4 { args.pop() } else { None };
                let [data_register, base_register, offset] = args.try_into().unwrap();
                let (offset_positive, data_register) = match data_register {
                    Argument::Register(data_register) => (true, data_register),
                    Argument::SignedRegister(sign, data_register) => (sign, data_register),
                    _ => return Err(Rich::custom(span, "expected register")),
                };
                let (data_register, base_register) = match base_register {
                    Argument::Address {
                        base_register,
                        operands,
                        write_back,
                    } => {
                        if !operands.is_empty() {
                            return Err(Rich::custom(
                                span,
                                "additional address information can only come after ']'",
                            ));
                        }
                        if write_back {
                            let opcode_t = Opcode::SingleTransfer(kind, size, true);
                            return Err(Rich::custom(
                                span,
                                format!(
                                    "the write-back signifier '!' is not allowed, instead use the opcode {opcode_t}"
                                ),
                            ));
                        }
                        (data_register, base_register)
                    }
                    _ => {
                        return Err(Rich::custom(
                            span,
                            "expected a register followed by an address then an offset",
                        ));
                    }
                };
                let offset = match offset {
                    Argument::Expression(expression) => {
                        if shift.is_some() {
                            return Err(Rich::custom(
                                span,
                                "shift cannot be specified with expression offset",
                            ));
                        }
                        DataOperand::Constant(expression)
                    }
                    Argument::Register(register) => match shift {
                        Some(Argument::Shift(shift)) => DataOperand::Register(register, shift),
                        None => DataOperand::Register(register, Shift::default()),
                        _ => {
                            return Err(Rich::custom(span, "invalid offset, expected shift"));
                        }
                    },
                    _ => return Err(Rich::custom(span, "invalid offset")),
                };
                Ok(Processed::Instr(AsmInstr::SingleTransfer {
                    kind,
                    size,
                    write_back: t_flag,
                    offset_positive,
                    pre_index: false,
                    data_register,
                    base_register,
                    offset,
                }))
            }
            _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<address>"))),
        },
        Opcode::BlockTransfer(transfer_kind, _, _) => todo!(),
        Opcode::Swap(byte) => {
            let [dest, source, base] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match (dest, source, base) {
                (
                    Argument::Register(dest),
                    Argument::Register(source),
                    Argument::Address {
                        base_register: base,
                        operands,
                        write_back: false,
                    },
                ) if operands.is_empty() => Ok(Processed::Instr(AsmInstr::Swap {
                    byte,
                    dest,
                    source,
                    base,
                })),
                _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rm,[Rn]"))),
            }
        }
        Opcode::Swi => {
            let [comment] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match comment {
                Argument::Expression(comment) => {
                    Ok(Processed::Instr(AsmInstr::SoftwareInterrupt { comment }))
                }
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <expression>"))),
            }
        }
        Opcode::Equ => {
            let [expr] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match expr {
                Argument::Expression(expr) => Ok(Processed::Equ(expr)),
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <expression>"))),
            }
        }
        Opcode::DefW => {
            let exprs = args
                .into_iter()
                .map(|arg| match arg {
                    Argument::Expression(expression) => Ok(Processed::DefW(expression)),
                    _ => Err(Rich::custom(
                        span,
                        format!("syntax: {opcode} <expression>,...,<expression>"),
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Processed::Vec(exprs))
        }
    }
}

enum DataOpKind {
    NoDest,
    NoOp1,
    ThreeArg,
}

impl DataOp {
    fn kind(self) -> DataOpKind {
        match self {
            DataOp::Tst => DataOpKind::NoDest,
            DataOp::Teq => DataOpKind::NoDest,
            DataOp::Cmp => DataOpKind::NoDest,
            DataOp::Cmn => DataOpKind::NoDest,
            DataOp::Mov => DataOpKind::NoOp1,
            DataOp::Mvn => DataOpKind::NoOp1,
            _ => DataOpKind::ThreeArg,
        }
    }
}
