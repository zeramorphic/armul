use armul::{
    assemble::{assemble, AssemblerError, AssemblerOutput},
    instr::{LineInfo, Register},
    processor::{Processor, ProcessorError, ProcessorListener, ProcessorState},
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
}

#[derive(Default)]
struct MyStateLock(RwLock<MyState>);

#[derive(Serialize)]
struct PrettyAssemblerError {
    line_number: usize,
    error: String,
}

#[tauri::command]
async fn load_program(
    state: tauri::State<'_, MyStateLock>,
    file: &str,
    contents: &str,
) -> Result<(), Vec<PrettyAssemblerError>> {
    let assembled = assemble(contents).map_err(|errs| {
        errs.into_iter()
            .map(|err| PrettyAssemblerError {
                line_number: err.line_number,
                error: err.error.to_string(),
            })
            .collect::<Vec<_>>()
    })?;
    let mut new_processor = Processor::default();
    new_processor
        .memory_mut()
        .set_words_aligned(0, &assembled.instrs);
    let mut state = state.0.write();
    state.processor = new_processor;
    state.info = ProcessorInformation::new(file.to_owned());
    Ok(())
}

#[tauri::command]
fn line_at(state: tauri::State<'_, MyStateLock>, addr: u32, disassemble: bool) -> LineInfo {
    let state = state.0.read();
    LineInfo::new(
        addr,
        state.processor.memory().get_word_aligned(addr),
        state.assembled.as_ref(),
        disassemble,
    )
}

#[tauri::command]
fn registers(state: tauri::State<'_, MyStateLock>) -> Registers {
    state.0.read().processor.registers().clone()
}

#[derive(Clone, Serialize)]
pub struct ProcessorInformation {
    file: String,
    state: ProcessorState,
    previous_pc: u32,

    steps: usize,
    nonseq_cycles: usize,
    seq_cycles: usize,
    internal_cycles: usize,
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
            state: Default::default(),
            previous_pc: 0,
            steps: Default::default(),
            nonseq_cycles: Default::default(),
            seq_cycles: Default::default(),
            internal_cycles: Default::default(),
        }
    }
}

#[tauri::command]
fn processor_info(state: tauri::State<'_, MyStateLock>) -> ProcessorInformation {
    state.0.read().info.clone()
}

pub struct TauriProcessorListener<'a> {
    info: &'a mut ProcessorInformation,
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
}

#[tauri::command]
fn step_once(state: tauri::State<'_, MyStateLock>) -> Result<ProcessorState, ProcessorError> {
    let mut guard = state.0.write();
    let state = &mut *guard;
    state.info.steps += 1;
    state.info.previous_pc = state.processor.registers().get(Register::R15);
    state.processor.try_execute(&mut TauriProcessorListener {
        info: &mut state.info,
    })?;
    state.info.state = state.processor.state();
    // Advance the program counter.
    *state.processor.registers_mut().get_mut(Register::R15) += 4;

    Ok(state.processor.state())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MyStateLock::default())
        .invoke_handler(tauri::generate_handler![
            load_program,
            line_at,
            registers,
            step_once,
            processor_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
