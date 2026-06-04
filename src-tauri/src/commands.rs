//! # Musico — Tauri IPC Command Handlers
//!
//! Each `#[tauri::command]` function maps to a JS `invoke()` call in the
//! frontend. These handlers delegate to `musico_playback::PlaybackEngine`
//! and the shared `AppState`.

use std::path::Path;
use tauri::State;
use walkdir::WalkDir;

use crate::state::{AppSettings, AppState, SongMeta};

// ─── Library ─────────────────────────────────────────────────────────────────

/// Returns the full library as a Vec of SongMeta.
#[tauri::command]
pub fn get_library(state: State<AppState>) -> Vec<SongMeta> {
    state.library.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Scans the configured music folder and rebuilds the library index.
#[tauri::command]
pub fn scan_library(state: State<AppState>) -> Vec<SongMeta> {
    let folder = state.music_folder.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut songs = Vec::new();
    let mut id_counter = 0u64;

    let extensions = ["mp3", "flac", "ogg", "wav", "m4a", "aac", "opus", "wma"];

    for entry in WalkDir::new(&folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() { continue; }

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !extensions.contains(&ext.as_str()) { continue; }

        id_counter += 1;

        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Try to parse "Artist - Title" from filename
        let (artist, title) = if let Some(sep_pos) = file_stem.find(" - ") {
            (file_stem[..sep_pos].to_string(), file_stem[sep_pos + 3..].to_string())
        } else {
            ("Unknown Artist".to_string(), file_stem)
        };

        let album = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Album")
            .to_string();

        songs.push(SongMeta {
            id: id_counter.to_string(),
            title,
            artist,
            album,
            duration_secs: 0.0, // Will be filled on first play
            file_path: path.to_string_lossy().to_string(),
        });
    }

    *state.library.lock().unwrap_or_else(|e| e.into_inner()) = songs.clone();
    songs
}

/// Search the library by query string.
#[tauri::command]
pub fn search_library(state: State<AppState>, query: String) -> Vec<SongMeta> {
    let q = query.to_lowercase();
    let lib = state.library.lock().unwrap_or_else(|e| e.into_inner());
    lib.iter()
        .filter(|s| {
            s.title.to_lowercase().contains(&q)
                || s.artist.to_lowercase().contains(&q)
                || s.album.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

// ─── Playback ────────────────────────────────────────────────────────────────

/// Play a specific song.
#[tauri::command]
pub fn play_song(state: State<AppState>, song: SongMeta) -> Result<(), String> {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        eng.play(song.into()).map_err(|e| e.to_string())
    } else {
        Err("Playback engine not initialized".into())
    }
}

/// Pause playback.
#[tauri::command]
pub fn pause(state: State<AppState>) -> Result<(), String> {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        eng.pause().map_err(|e| e.to_string())
    } else {
        Err("Playback engine not initialized".into())
    }
}

/// Resume playback.
#[tauri::command]
pub fn resume(state: State<AppState>) -> Result<(), String> {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        eng.resume().map_err(|e| e.to_string())
    } else {
        Err("Playback engine not initialized".into())
    }
}

/// Seek to a position in seconds.
#[tauri::command]
pub fn seek(state: State<AppState>, position_secs: f32) -> Result<(), String> {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        eng.seek(position_secs).map_err(|e| e.to_string())
    } else {
        Err("Playback engine not initialized".into())
    }
}

/// Set volume (0.0 – 1.0).
#[tauri::command]
pub fn set_volume(state: State<AppState>, volume: f32) -> Result<(), String> {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        eng.set_volume(volume).map_err(|e| e.to_string())
    } else {
        Err("Playback engine not initialized".into())
    }
}

/// Toggle mute.
#[tauri::command]
pub fn toggle_mute(state: State<AppState>) -> Result<(), String> {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        eng.toggle_mute().map_err(|e| e.to_string())
    } else {
        Err("Playback engine not initialized".into())
    }
}

// ─── Playback State ──────────────────────────────────────────────────────────

/// Get current playback state for the frontend to poll.
#[derive(serde::Serialize)]
pub struct PlaybackStateDto {
    pub status: String,
    pub position_secs: f32,
    pub duration_secs: f32,
    pub volume: f32,
    pub muted: bool,
    pub current_song: Option<SongMeta>,
}

#[tauri::command]
pub fn get_playback_state(state: State<AppState>) -> PlaybackStateDto {
    let engine = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref eng) = *engine {
        let ps = eng.state();
        PlaybackStateDto {
            status: format!("{:?}", ps.status),
            position_secs: ps.position_secs,
            duration_secs: ps.duration_secs,
            volume: ps.volume,
            muted: ps.muted,
            current_song: ps.current_song.map(|s| SongMeta {
                id: s.id,
                title: s.title,
                artist: s.artist,
                album: s.album,
                duration_secs: s.duration_secs,
                file_path: s.file_path,
            }),
        }
    } else {
        PlaybackStateDto {
            status: "Stopped".into(),
            position_secs: 0.0,
            duration_secs: 0.0,
            volume: 0.68,
            muted: false,
            current_song: None,
        }
    }
}

// ─── Queue ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_queue(state: State<AppState>) -> Vec<SongMeta> {
    state.queue.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
pub fn add_to_queue(state: State<AppState>, song: SongMeta) {
    state.queue.lock().unwrap_or_else(|e| e.into_inner()).push(song);
}

#[tauri::command]
pub fn remove_from_queue(state: State<AppState>, song_id: String) {
    state.queue.lock().unwrap_or_else(|e| e.into_inner())
        .retain(|s| s.id != song_id);
}

// ─── Settings ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) {
    *state.settings.lock().unwrap_or_else(|e| e.into_inner()) = settings;
}

// ─── Recommendations ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_recommendations(state: State<AppState>) -> Vec<SongMeta> {
    // For now, return a random subset of the library.
    let lib = state.library.lock().unwrap_or_else(|e| e.into_inner());
    lib.iter().take(6).cloned().collect()
}
