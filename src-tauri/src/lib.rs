use armul::{
    assemble::{assemble, AssemblerError, AssemblerOutput},
    instr::{LineInfo, Register},
    processor::{Processor, ProcessorError, ProcessorListener, ProcessorState},
    registers::Registers,
};
use parking_lot::RwLock;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[derive(Default)]
struct MyState {
    assembled: Option<AssemblerOutput>,
    processor: Processor,
}

#[derive(Default)]
struct MyStateLock(RwLock<MyState>);

#[tauri::command]
async fn load_program(
    state: tauri::State<'_, MyStateLock>,
    contents: &str,
) -> Result<(), Vec<AssemblerError>> {
    let assembled = assemble(contents)?;
    let mut new_processor = Processor::default();
    new_processor
        .memory_mut()
        .set_words_aligned(0, &assembled.instrs);
    state.0.write().processor = new_processor;
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

#[derive(Default)]
pub struct TauriProcessorListener {}

impl ProcessorListener for TauriProcessorListener {
    fn cycle(&mut self, cycle: armul::processor::Cycle, count: usize, pc: u32) {}

    fn pipeline_flush(&mut self, pc: u32) {}
}

#[tauri::command]
fn step_once(state: tauri::State<'_, MyStateLock>) -> Result<ProcessorState, ProcessorError> {
    let proc = &mut state.0.write().processor;
    let pc = proc.registers().get(Register::R15);
    // println!();
    // println!("{}", proc.registers());
    // println!(
    //     "Step {}: about to execute {}",
    //     i + 1,
    //     Instr::decode(proc.memory().get_word_aligned(pc))
    //         .map_or_else(|| "???".to_owned(), |(cond, i)| Instr::display(&i, cond))
    // );
    proc.try_execute(&mut TauriProcessorListener {})?;
    // Advance the program counter.
    *proc.registers_mut().get_mut(Register::R15) += 4;

    Ok(proc.state())
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
            step_once
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
