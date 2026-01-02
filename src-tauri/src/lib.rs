use armul::{
    assemble::{assemble, AssemblerError, AssemblerOutput},
    instr::LineInfo,
    processor::Processor,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MyStateLock::default())
        .invoke_handler(tauri::generate_handler![load_program, line_at])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
