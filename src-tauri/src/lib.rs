use std::path::Path;

use armul::{
    assemble::{assemble, AssemblerOutput},
    instr::{LineInfo, Register},
    processor::{Processor, ProcessorListener, ProcessorState},
    registers::Registers,
};
use parking_lot::RwLock;
use serde::Serialize;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[derive(Default)]
struct MyState {
    assembled: Option<AssemblerOutput>,
    processor: Processor,
    info: ProcessorInformation,
    user_input: String,
}

#[derive(Default)]
struct MyStateLock(RwLock<MyState>);

#[derive(Serialize)]
struct PrettyAssemblerError {
    line_number: Option<usize>,
    error: String,
}

#[tauri::command]
async fn load_program(
    state: tauri::State<'_, MyStateLock>,
    path: &Path,
) -> Result<(), Vec<PrettyAssemblerError>> {
    let contents = std::fs::read_to_string(path).map_err(|e| {
        vec![PrettyAssemblerError {
            line_number: None,
            error: e.to_string(),
        }]
    })?;
    let assembled = assemble(&contents).map_err(|errs| {
        errs.into_iter()
            .map(|err| match err.error {
                armul::assemble::LineError::ParseError(parse_error) => PrettyAssemblerError {
                    line_number: None,
                    error: parse_error,
                },
                error => PrettyAssemblerError {
                    line_number: Some(err.line_number),
                    error: error.to_string(),
                },
            })
            .collect::<Vec<_>>()
    })?;
    let mut new_processor = Processor::default();
    new_processor
        .memory_mut()
        .set_words_aligned(0, &assembled.instrs);
    let mut state = state.0.write();
    state.processor = new_processor;
    state.assembled = Some(assembled);
    state.info = ProcessorInformation::new(path.file_name().map_or_else(
        || path.to_string_lossy().to_string(),
        |base| base.to_string_lossy().to_string(),
    ));
    Ok(())
}

#[tauri::command]
fn line_at(state: tauri::State<'_, MyStateLock>, addr: u32) -> LineInfo {
    let state = state.0.read();
    LineInfo::new(
        addr,
        state.processor.memory().get_word_aligned(addr),
        state.assembled.as_ref(),
    )
}

#[tauri::command]
fn registers(state: tauri::State<'_, MyStateLock>) -> Registers {
    state.0.read().processor.registers().clone()
}

#[tauri::command]
fn set_user_input(state: tauri::State<'_, MyStateLock>, user_input: String) {
    state.0.write().user_input = user_input;
}

#[derive(Clone, Serialize)]
pub struct ProcessorInformation {
    file: String,
    state: Result<ProcessorState, String>,
    previous_pc: u32,

    steps: usize,
    nonseq_cycles: usize,
    seq_cycles: usize,
    internal_cycles: usize,

    /// This is stupidly inefficient because we're sending the entire output to the TS side
    /// even when only a single character has changed. But it's good enough for now.
    output: String,
}

impl Default for ProcessorInformation {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl ProcessorInformation {
    pub fn new(file: String) -> ProcessorInformation {
        Self {
            file,
            state: Ok(Default::default()),
            previous_pc: 0,
            steps: Default::default(),
            nonseq_cycles: Default::default(),
            seq_cycles: Default::default(),
            internal_cycles: Default::default(),
            output: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self {
            file: std::mem::take(&mut self.file),
            ..Default::default()
        }
    }
}

#[tauri::command]
fn processor_info(state: tauri::State<'_, MyStateLock>) -> ProcessorInformation {
    state.0.read().info.clone()
}

pub struct TauriProcessorListener<'a> {
    info: &'a mut ProcessorInformation,
    user_input: &'a mut String,
    input_used: bool,
}

impl<'a> ProcessorListener for TauriProcessorListener<'a> {
    fn cycle(&mut self, cycle: armul::processor::Cycle, count: usize, _pc: u32) {
        match cycle {
            armul::processor::Cycle::NonSeq => self.info.nonseq_cycles += count,
            armul::processor::Cycle::Seq => self.info.seq_cycles += count,
            armul::processor::Cycle::Internal => self.info.internal_cycles += count,
            armul::processor::Cycle::Coprocessor => {}
        }
    }

    fn pipeline_flush(&mut self, _pc: u32) {
        self.info.nonseq_cycles += 1;
        self.info.seq_cycles += 1;
    }

    fn getc(&mut self) -> Option<char> {
        if self.user_input.is_empty() {
            None
        } else {
            self.input_used = true;
            Some(self.user_input.remove(0))
        }
    }

    fn putc(&mut self, c: char) {
        self.info.output.push(c);
    }
}

/// Returns the new user input field, if it was changed.
#[tauri::command]
fn step_once(state: tauri::State<'_, MyStateLock>) -> Option<String> {
    let mut guard = state.0.write();
    let state = &mut *guard;
    state.info.previous_pc = state.processor.registers().get(Register::R15);

    // Save some of the old info.
    let old_n = state.info.nonseq_cycles;
    let old_s = state.info.seq_cycles;
    let old_i = state.info.internal_cycles;

    let mut listener = TauriProcessorListener {
        info: &mut state.info,
        user_input: &mut state.user_input,
        input_used: false,
    };
    let input_used;
    match state.processor.try_execute(&mut listener) {
        Ok(()) => {
            input_used = listener.input_used;
            state.info.state = Ok(state.processor.state());

            // Advance the program counter and log that we've done a step.
            state.info.steps += 1;
            *state.processor.registers_mut().get_mut(Register::R15) += 4;
        }
        Err(err) => {
            input_used = listener.input_used;
            // Reset the old info because we didn't complete a step.
            state.info.nonseq_cycles = old_n;
            state.info.seq_cycles = old_s;
            state.info.internal_cycles = old_i;

            state.info.state = Err(err.to_string());
        }
    }
    if input_used {
        Some(state.user_input.clone())
    } else {
        None
    }
}

#[tauri::command]
fn reset(state: tauri::State<'_, MyStateLock>, hard: bool) {
    let mut guard = state.0.write();
    let state = &mut *guard;

    state.info.reset();
    if hard {
        // Hard resets put everything (even memory) back to where it was at the start.
        state.processor = Processor::default();
        if let Some(assembled) = state.assembled.as_ref() {
            state
                .processor
                .memory_mut()
                .set_words_aligned(0, &assembled.instrs);
        }
    } else {
        // Soft resets just put the PC back to 0.
        state.processor.registers_mut().set(Register::R15, 0);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(MyStateLock::default())
        .invoke_handler(tauri::generate_handler![
            load_program,
            line_at,
            registers,
            set_user_input,
            step_once,
            processor_info,
            reset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
