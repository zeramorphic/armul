//! A parser for ARM assembly.

use std::fmt::Display;

use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};
use logos::Logos;

use crate::{
    assemble::{
        AssemblerError, LineError,
        syntax::{
            AsmInstr, AsmLine, AsmLineContents, DataOperand, Expression, MsrSource, Shift,
            ShiftAmount,
        },
    },
    instr::{Cond, DataOp, Psr, Register, ShiftType, TransferKind, TransferSize},
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

    parser(&line_indices)
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
    ShiftType(ShiftType),

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
    #[token(",")]
    Comma,
    #[token("#")]
    Hash,

    #[regex(r"[ \t\f]+")]
    Whitespace,

    #[regex("\n")]
    Newline,

    #[regex(r";[^\n]+", allow_greedy = true)]
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

        fn disambiguate_shift(name: &str) -> Option<ShiftType> {
            match name {
                "lsl" => Some(ShiftType::LogicalLeft),
                "asl" => Some(ShiftType::LogicalLeft),
                "lsr" => Some(ShiftType::LogicalRight),
                "asr" => Some(ShiftType::ArithmeticRight),
                "ror" => Some(ShiftType::RotateRight),
                "rrx" => Some(ShiftType::RotateRightExtended),
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
                ("ldr", "", Opcode::SingleTransfer(TransferKind::Load, TransferSize::Word, false)),
                ("ldr", "b", Opcode::SingleTransfer(TransferKind::Load, TransferSize::Byte, true)),
                ("ldr", "t", Opcode::SingleTransfer(TransferKind::Load, TransferSize::Word, false)),
                ("ldr", "bt", Opcode::SingleTransfer(TransferKind::Load, TransferSize::Byte, true)),
                ("ldr", "h", Opcode::SingleTransfer(TransferKind::Load, TransferSize::HalfWord, false)),
                ("ldr", "sh", Opcode::SingleTransfer(TransferKind::Load, TransferSize::SignExtendedHalfWord, false)),
                ("ldr", "sb", Opcode::SingleTransfer(TransferKind::Load, TransferSize::SignExtendedByte, false)),
                ("str", "", Opcode::SingleTransfer(TransferKind::Store, TransferSize::Word, false)),
                ("str", "b", Opcode::SingleTransfer(TransferKind::Store, TransferSize::Byte, true)),
                ("str", "t", Opcode::SingleTransfer(TransferKind::Store, TransferSize::Word, false)),
                ("str", "bt", Opcode::SingleTransfer(TransferKind::Store, TransferSize::Byte, true)),
                ("str", "h", Opcode::SingleTransfer(TransferKind::Store, TransferSize::HalfWord, false)),
                ("str", "sh", Opcode::SingleTransfer(TransferKind::Store, TransferSize::SignExtendedHalfWord, false)),
                ("str", "sb", Opcode::SingleTransfer(TransferKind::Store, TransferSize::SignExtendedByte, false)),
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

        match self {
            Token::Name(name) => {
                let lower = name.to_lowercase();
                if let Some(reg) = disambiguate_register(&lower) {
                    Token::Register(reg)
                } else if let Some(shift) = disambiguate_shift(&lower) {
                    Token::ShiftType(shift)
                } else if let Some(mnemonic) = disambiguate_mnemonic(&lower) {
                    Token::Opcode(mnemonic)
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
    Mrs,
    Msr,
    /// Set condition codes; accumulate.
    Mul(bool, bool),
    /// Set condition codes; signed; accumulate.
    MulLong(bool, bool, bool),
    /// The bool is for forced writeback (the T flag).
    SingleTransfer(TransferKind, TransferSize, bool),
    /// The bool flags are positive offset and pre index.
    BlockTransfer(TransferKind, bool, bool),
    Swap(bool),
    Swi,
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
            Opcode::Adr => todo!(),
            Opcode::Nop => todo!(),
            Opcode::Mrs => todo!(),
            Opcode::Msr => todo!(),
            Opcode::Mul(_, _) => todo!(),
            Opcode::MulLong(_, _, _) => todo!(),
            Opcode::SingleTransfer(transfer_kind, transfer_size, _) => todo!(),
            Opcode::BlockTransfer(transfer_kind, _, _) => todo!(),
            Opcode::Swap(_) => todo!(),
            Opcode::Swi => todo!(),
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
            Token::ShiftType(shift_type) => write!(f, "{shift_type}"),
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Hash => write!(f, "#"),
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

fn parser<'tokens, 'src: 'tokens, I>(
    line_indices: &[usize],
) -> impl Parser<'tokens, I, Vec<AsmLine>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    line_contents()
        .or_not()
        .map(|x| x.unwrap_or_default())
        .spanned()
        .then(select! { Token::Comment(x) => x }.or_not())
        .then_ignore(select! { Token::Newline => () }.repeated().at_least(1))
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

fn line_contents<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<AsmLineContents>, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    let label = select! { Token::Name(label) => label };
    let mnemonic = select! { Token::Opcode(mnemonic) => mnemonic };
    let args = argument()
        .separated_by(
            whitespace()
                .then_ignore(select! { Token::Comma => () })
                .then_ignore(whitespace()),
        )
        .at_least(1)
        .collect::<Vec<_>>();

    label
        .or_not()
        .then_ignore(whitespace())
        .then((mnemonic.then_ignore(whitespace()).then(args)).or_not())
        .try_map(|(label, instr), span| {
            if let Some(((cond, opcode), args)) = instr {
                process_instruction(opcode, args, span).map(|instr| (label, Some((cond, instr))))
            } else {
                Ok((label, None))
            }
        })
        .map(|(label, instr)| {
            let mut result = Vec::new();
            if let Some(label) = label {
                result.push(AsmLineContents::Label(label.to_owned()))
            }
            if let Some((cond, instr)) = instr {
                result.push(AsmLineContents::Instr(cond, instr))
            }
            result
        })
}

#[derive(Debug)]
enum Argument {
    Register(Register),
    Shift(Shift),
    Expression(Expression),
}

fn argument<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Argument, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    choice((
        register().map(Argument::Register),
        shift().map(Argument::Shift),
        expression().map(Argument::Expression),
    ))
}

fn shift<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Shift, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! { Token::ShiftType(t) => t }
        .then_ignore(whitespace().or_not())
        .then(choice((
            register().map(ShiftAmount::Register),
            expression().map(ShiftAmount::Constant),
        )))
        .map(|(shift_type, shift_amount)| Shift {
            shift_type,
            shift_amount,
        })
}

fn register<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Register, extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token::Register(r) => r
    }
}

fn expression<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expression, extra::Err<Rich<'tokens, Token<'src>>>>
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
            select! { Token::Hash => () }.or_not().ignore_then(number),
            select! { Token::Name(name) => Expression::Label(name.to_owned()) },
        ));
        atom
    })
}

fn whitespace<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, (), extra::Err<Rich<'tokens, Token<'src>>>>
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = SimpleSpan>,
{
    select! {
        Token::Whitespace => ()
    }
    .or_not()
    .ignored()
}

fn process_instruction<'tokens, 'src: 'tokens>(
    opcode: Opcode,
    mut args: Vec<Argument>,
    span: SimpleSpan,
) -> Result<AsmInstr, Rich<'tokens, Token<'src>>> {
    match opcode {
        Opcode::BranchExchange => {
            let [operand] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match operand {
                Argument::Register(operand) => Ok(AsmInstr::BranchExchange { operand }),
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <offset>"))),
            }
        }
        Opcode::Branch { link } => {
            let [target] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match target {
                Argument::Expression(target) => Ok(AsmInstr::Branch { link, target }),
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <offset>"))),
            }
        }
        Opcode::Adr => {
            let [dest, expr] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
            match (dest, expr) {
                (Argument::Register(dest), Argument::Expression(expr)) => Ok(AsmInstr::Adr {
                    long: false,
                    dest,
                    expr,
                }),
                _ => Err(Rich::custom(
                    span,
                    format!("syntax: {opcode} Rd,<expression>"),
                )),
            }
        }
        Opcode::Nop => todo!(),
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
            };
            match op.kind() {
                DataOpKind::NoDest => {
                    let [op1] = args
                        .try_into()
                        .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
                    match op1 {
                        Argument::Register(op1) => Ok(AsmInstr::Data {
                            set_condition_codes,
                            op,
                            dest: Register::R0,
                            op1,
                            op2,
                        }),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rn,<Op2>"))),
                    }
                }
                DataOpKind::NoOp1 => {
                    let [dest] = args
                        .try_into()
                        .map_err(|_| Rich::custom(span, "expected 2 arguments"))?;
                    match dest {
                        Argument::Register(dest) => Ok(AsmInstr::Data {
                            set_condition_codes,
                            op,
                            dest,
                            op1: Register::R0,
                            op2,
                        }),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<Op2>"))),
                    }
                }
                DataOpKind::ThreeArg => match TryInto::<[Argument; 1]>::try_into(args) {
                    Ok([op1]) => match op1 {
                        Argument::Register(op1) => Ok(AsmInstr::Data {
                            set_condition_codes,
                            op,
                            dest: op1,
                            op1,
                            op2,
                        }),
                        _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,<Op2>"))),
                    },
                    Err(args) => {
                        let [dest, op1] = args
                            .try_into()
                            .map_err(|_| Rich::custom(span, "expected 3 arguments"))?;
                        match (dest, op1) {
                            (Argument::Register(dest), Argument::Register(op1)) => {
                                Ok(AsmInstr::Data {
                                    set_condition_codes,
                                    op,
                                    dest,
                                    op1,
                                    op2,
                                })
                            }
                            _ => Err(Rich::custom(span, format!("syntax: {opcode} Rd,Rn,<Op2>"))),
                        }
                    }
                },
            }
        }
        Opcode::Mrs => todo!(),
        Opcode::Msr => todo!(),
        Opcode::Mul(_, _) => todo!(),
        Opcode::MulLong(_, _, _) => todo!(),
        Opcode::SingleTransfer(transfer_kind, transfer_size, _) => todo!(),
        Opcode::BlockTransfer(transfer_kind, _, _) => todo!(),
        Opcode::Swap(_) => todo!(),
        Opcode::Swi => {
            let [comment] = args
                .try_into()
                .map_err(|_| Rich::custom(span, "expected 1 argument"))?;
            match comment {
                Argument::Expression(comment) => Ok(AsmInstr::SoftwareInterrupt { comment }),
                _ => Err(Rich::custom(span, format!("syntax: {opcode} <expression>"))),
            }
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
