//! Provides a test procedure for assembly routines.

use std::collections::BTreeMap;

use crate::{
    assemble::{AssemblerError, AssemblerOutput, assemble},
    instr::{Instr, Register},
    mode::Mode,
    processor::{Processor, ProcessorError, ProcessorState, test::TestProcessorListener},
    registers::PhysicalRegister,
};

#[derive(Debug)]
pub enum TestError {
    FileError(String),
    AssemblerError(Vec<AssemblerError>),
    ProcessorError(ProcessorError),
    InvalidComment(String),
    InvalidParams(&'static str, String),
    StepsNotGiven,
}

pub fn test(src: &str) -> Result<(), TestError> {
    let assembled = assemble(src).map_err(TestError::AssemblerError)?;
    println!("assembled in {} passes", assembled.passes);
    for instr in &assembled.instrs {
        println!(
            "{}",
            Instr::decode(*instr)
                .map_or_else(|| "???".to_owned(), |(cond, i)| Instr::display(&i, cond))
        );
    }

    // Extract the test comments at the start of the file.
    let mut steps = None;
    // Whether the procedure is expected to halt itself within the given number of steps.
    let mut halts = false;
    // The initial mode to initialise the processor with.
    let mut mode = Mode::Usr;
    let mut output = BTreeMap::<PhysicalRegister, u32>::new();
    for line in src.lines() {
        if let Some(comment) = line.trim_start().strip_prefix(";!") {
            let comment = comment.trim();
            let Some((kwd, params)) = comment.split_once(' ') else {
                return Err(TestError::InvalidComment(comment.to_owned()));
            };
            let kwd = kwd.to_uppercase();
            let mut kwd_found = false;
            // Iterate reversed so that longer strings are matched first.
            for (pattern, reg) in [
                ("R0", PhysicalRegister::R0),
                ("R1", PhysicalRegister::R1),
                ("R2", PhysicalRegister::R2),
                ("R3", PhysicalRegister::R3),
                ("R4", PhysicalRegister::R4),
                ("R5", PhysicalRegister::R5),
                ("R6", PhysicalRegister::R6),
                ("R7", PhysicalRegister::R7),
                ("R8", PhysicalRegister::R8),
                ("R9", PhysicalRegister::R9),
                ("R10", PhysicalRegister::R10),
                ("R11", PhysicalRegister::R11),
                ("R12", PhysicalRegister::R12),
                ("R13", PhysicalRegister::R13),
                ("SP", PhysicalRegister::R13),
                ("R14", PhysicalRegister::R14),
                ("LR", PhysicalRegister::R14),
                ("R15", PhysicalRegister::R15),
                ("PC", PhysicalRegister::R15),
                ("R8FIQ", PhysicalRegister::R8Fiq),
                ("R9FIQ", PhysicalRegister::R9Fiq),
                ("R10FIQ", PhysicalRegister::R10Fiq),
                ("R11FIQ", PhysicalRegister::R11Fiq),
                ("R12FIQ", PhysicalRegister::R12Fiq),
                ("R13FIQ", PhysicalRegister::R13Fiq),
                ("R14FIQ", PhysicalRegister::R14Fiq),
                ("R13SVC", PhysicalRegister::R13Svc),
                ("R14SVC", PhysicalRegister::R14Svc),
                ("R13ABT", PhysicalRegister::R13Abt),
                ("R14ABT", PhysicalRegister::R14Abt),
                ("R13IRQ", PhysicalRegister::R13Irq),
                ("R14IRQ", PhysicalRegister::R14Irq),
                ("R13UND", PhysicalRegister::R13Und),
                ("R14UND", PhysicalRegister::R14Und),
                ("CPSR", PhysicalRegister::Cpsr),
                ("SPSRFIQ", PhysicalRegister::SpsrFiq),
                ("SPSRSVC", PhysicalRegister::SpsrSvc),
                ("SPSRABT", PhysicalRegister::SpsrAbt),
                ("SPSRIRQ", PhysicalRegister::SpsrIrq),
                ("SPSRUND", PhysicalRegister::SpsrUnd),
            ]
            .into_iter()
            .rev()
            {
                if kwd == pattern {
                    output.insert(reg, parse_param(&assembled, params)?);
                    kwd_found = true;
                    break;
                }
            }
            if !kwd_found {
                match kwd.as_ref() {
                    "STEPS" => {
                        steps = Some(
                            params
                                .parse::<usize>()
                                .map_err(|x| TestError::InvalidParams("steps", x.to_string()))?,
                        );
                    }
                    "HALTS" => {
                        steps = Some(
                            params
                                .parse::<usize>()
                                .map_err(|x| TestError::InvalidParams("halts", x.to_string()))?,
                        );
                        halts = true;
                    }
                    "MODE" => {
                        let mut succeeded = false;
                        let param = params.trim().to_lowercase();
                        for test_mode in [
                            Mode::Usr,
                            Mode::Fiq,
                            Mode::Irq,
                            Mode::Supervisor,
                            Mode::Abort,
                            Mode::System,
                            Mode::Undefined,
                        ] {
                            if param == test_mode.to_string().to_lowercase() || param == format!("{test_mode:?}").to_lowercase() {
                                mode = test_mode;
                                succeeded = true;
                                break;
                            }
                        }
                        if !succeeded {
                            return Err(TestError::InvalidParams("mode", param));
                        }
                    }
                    _ => return Err(TestError::InvalidComment(comment.to_owned())),
                }
            }
        }
    }

    let Some(steps) = steps else {
        return Err(TestError::StepsNotGiven);
    };

    let mut proc = Processor::default();
    proc.registers_mut().set_mode(mode);
    let mut listener = TestProcessorListener::default();
    let mut halted = false;
    proc.memory_mut().set_words_aligned(0x0, &assembled.instrs);
    for i in 0..steps {
        let pc = proc.registers().get(Register::R15);
        println!();
        println!("{}", proc.registers());
        println!(
            "Step {}: about to execute {}",
            i + 1,
            Instr::decode(proc.memory().get_word_aligned(pc))
                .map_or_else(|| "???".to_owned(), |(cond, i)| Instr::display(&i, cond))
        );
        proc.try_execute(&mut listener)
            .map_err(TestError::ProcessorError)?;
        // Advance the program counter.
        *proc.registers_mut().get_mut(Register::R15) += 4;

        if proc.state() == ProcessorState::Stopped {
            println!("Halted.");
            halted = true;
            break;
        }
    }

    println!("Terminated.");
    println!("{listener:#?}");
    println!("Final state:");
    println!("{}", proc.registers());

    // Assert that all of the results were as expected.
    assert_eq!(halts, halted, "halting behaviour mismatch");
    for (reg, value) in output {
        assert_eq!(
            proc.registers().get_physical(reg),
            value,
            "mismatch on register {reg:?}"
        );
    }

    Ok(())
}

fn parse_param(assembled: &AssemblerOutput, params: &str) -> Result<u32, TestError> {
    match params.parse::<i64>() {
        Ok(x) => Ok(x as u32),
        Err(_) => {
            // Try to parse it as a label instead.
            match assembled.labels.get(params) {
                Some(offset) => Ok(*offset),
                None => Err(TestError::InvalidParams("parameter", params.to_string())),
            }
        }
    }
}
