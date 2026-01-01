use armul::{
    assemble::{assemble, AssemblerError},
    processor::Processor,
};
use parking_lot::RwLock;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[derive(Default)]
struct MyState {
    processor: RwLock<Processor>,
}

#[tauri::command]
async fn load_program(
    state: tauri::State<'_, MyState>,
    contents: &str,
) -> Result<(), Vec<AssemblerError>> {
    let assembled = assemble(contents)?;
    let mut new_processor = Processor::default();
    new_processor
        .memory_mut()
        .set_words_aligned(0, &assembled.instrs);
    *state.processor.write() = new_processor;
    Ok(())
}

#[tauri::command]
fn line_at(state: tauri::State<'_, MyState>, addr: u32) -> u32 {
    state.processor.read().memory().get_word_aligned(addr)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MyState::default())
        .invoke_handler(tauri::generate_handler![load_program, line_at])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
