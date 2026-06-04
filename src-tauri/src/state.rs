//! # Musico — Shared Tauri State
//!
//! Thread-safe state shared between Tauri command handlers and the playback
//! engine. Wraps `PlaybackEngine` and app-level state (library, settings, queue).

use std::path::PathBuf;
use std::sync::Mutex;

use musico_playback::{PlaybackEngine, SongInfo};
use serde::{Deserialize, Serialize};

/// Application-level state managed by Tauri.
pub struct AppState {
    pub engine: Mutex<Option<PlaybackEngine>>,
    pub library: Mutex<Vec<SongMeta>>,
    pub music_folder: Mutex<PathBuf>,
    pub queue: Mutex<Vec<SongMeta>>,
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn new() -> Self {
        let music_dir = dirs_or_default();
        Self {
            engine: Mutex::new(PlaybackEngine::new().ok()),
            library: Mutex::new(Vec::new()),
            music_folder: Mutex::new(music_dir),
            queue: Mutex::new(Vec::new()),
            settings: Mutex::new(AppSettings::default()),
        }
    }
}

fn dirs_or_default() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join("Music"))
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// Song metadata sent to the frontend — serializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongMeta {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f32,
    pub file_path: String,
}

impl From<SongMeta> for SongInfo {
    fn from(m: SongMeta) -> Self {
        SongInfo {
            id: m.id,
            file_path: m.file_path,
            title: m.title,
            artist: m.artist.clone(),
            album: m.album.clone(),
            duration_secs: m.duration_secs,
            cover_art: None,
        }
    }
}

/// Persisted app settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub palette: String,
    pub art_tint: bool,
    pub smart_recs: bool,
    pub eq_enabled: bool,
    pub eq_preset: String,
    pub normalization: String, // "Off", "Track", "Album"
    pub crossfade_enabled: bool,
    pub crossfade_duration: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            palette: "nebula".into(),
            art_tint: true,
            smart_recs: true,
            eq_enabled: true,
            eq_preset: "Flat".into(),
            normalization: "Off".into(),
            crossfade_enabled: false,
            crossfade_duration: 3.0,
        }
    }
}
