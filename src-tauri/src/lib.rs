use armul::memory::Memory;
use parking_lot::RwLock;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct MyState {
    memory: RwLock<Memory>,
}

#[tauri::command]
fn line_at(state: tauri::State<'_, MyState>, addr: u32) -> u32 {
    state.memory.read().get_word_aligned(addr)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MyState {
            memory: RwLock::new(Memory::new(&[1, 2, 128, 931, 0, 4])),
        })
        .invoke_handler(tauri::generate_handler![greet, line_at])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
