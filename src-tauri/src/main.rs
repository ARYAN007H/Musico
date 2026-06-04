//! # Musico — Tauri Entry Point
//!
//! Bootstraps the Tauri application, registers state and command handlers,
//! and opens the main window.

// Prevents console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Library
            commands::get_library,
            commands::scan_library,
            commands::search_library,
            // Playback
            commands::play_song,
            commands::pause,
            commands::resume,
            commands::seek,
            commands::set_volume,
            commands::toggle_mute,
            commands::get_playback_state,
            // Queue
            commands::get_queue,
            commands::add_to_queue,
            commands::remove_from_queue,
            // Settings
            commands::get_settings,
            commands::save_settings,
            // Recommendations
            commands::get_recommendations,
        ])
        .run(tauri::generate_context!())
        .expect("error running Musico");
}
