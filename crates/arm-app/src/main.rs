// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arm_app::commands::{self, AppState};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_ruleset,
            commands::validate_entity,
            commands::effective_scores,
            commands::save_entity,
            commands::load_entity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
