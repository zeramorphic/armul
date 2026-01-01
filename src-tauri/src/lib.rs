use armul::processor::Processor;
use parking_lot::RwLock;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[derive(Default)]
struct MyState {
    processor: RwLock<Processor>,
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
        .invoke_handler(tauri::generate_handler![greet, line_at])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
