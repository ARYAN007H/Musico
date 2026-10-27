//! # Musico — Tauri Entry Point
//!
//! Bootstraps the Tauri application, registers state and command handlers,
//! spawns the playback event bridge, and opens the main window.

// Prevents console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod event_bridge;
mod lyrics;
#[cfg(target_os = "linux")]
mod mpris;
mod scanner;
mod state;

use tauri::Manager;

use state::AppState;

fn main() {
    // Logs go to ~/.local/share/musico/musico.log (and stderr when RUST_LOG
    // is set explicitly, for development).
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    if let Some(dir) = dirs::data_dir().map(|d| d.join("musico")) {
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("musico.log"))
        {
            builder.target(env_logger::Target::Pipe(Box::new(file)));
        }
    }
    if std::env::var("RUST_LOG").is_ok() {
        builder.parse_default_env();
    }
    builder.init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let engine = app.state::<AppState>().engine.clone();
            if let Some(engine) = engine {
                event_bridge::spawn(engine, handle.clone());
            } else {
                log::warn!("Playback engine failed to initialize — no audio device?");
            }

            // Optional: force a scan at launch (testing)
            if std::env::args().any(|a| a == "--scan-on-start") {
                scanner::start_scan(handle.clone());
            } else {
                // Normal startup: a configured folder triggers an incremental
                // scan — cache hits make it near-instant.
                let has_folder = {
                    let state = app.state::<AppState>();
                    let guard = state.settings.lock().unwrap_or_else(|e| e.into_inner());
                    guard.music_folder.is_some()
                };
                if has_folder {
                    scanner::start_scan(handle.clone());
                }
            }

            // MPRIS: system media keys, playerctl, desktop widgets (D-Bus — Linux only)
            #[cfg(target_os = "linux")]
            {
                let mpris_handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    match mpris::start_mpris_server().await {
                        Ok((cmd_rx, mpris_state)) => {
                            // Idle start: report Stopped immediately.
                            mpris::update_mpris_state(
                                &mpris_state,
                                mpris::MprisMetadata {
                                    playback_status: "Stopped".into(),
                                    ..Default::default()
                                },
                            );
                            *mpris_handle
                                .state::<AppState>()
                                .mpris_state
                                .lock()
                                .unwrap_or_else(|e| e.into_inner()) = Some(mpris_state);
                            std::thread::spawn(move || {
                                mpris::command_loop(cmd_rx, mpris_handle);
                            });
                        }
                        Err(e) => log::warn!("MPRIS unavailable: {e}"),
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Library
            commands::get_library,
            commands::start_scan,
            commands::get_cover,
            commands::search_library,
            // Playback
            commands::play_song,
            commands::pause,
            commands::resume,
            commands::seek,
            commands::set_volume,
            commands::toggle_mute,
            commands::get_playback_state,
            commands::next,
            commands::previous,
            commands::set_playback_modes,
            // Queue
            commands::get_queue,
            commands::add_to_queue,
            commands::play_next,
            commands::remove_from_queue,
            commands::clear_queue,
            // Settings
            commands::get_settings,
            commands::save_settings,
            commands::pick_folder,
            commands::set_music_folder,
            // Recommendations
            commands::get_recommendations,
            // Stats & playlists
            commands::get_stats,
            commands::get_playlists,
            // Lyrics
            commands::get_lyrics,
            // EQ & normalization
            commands::set_eq,
            commands::set_normalization,
        ])
        .run(tauri::generate_context!())
        .expect("error running Musico");
}
