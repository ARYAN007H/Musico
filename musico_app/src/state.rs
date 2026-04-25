use std::path::PathBuf;
use iced::Color;
use musico_playback::{PlaybackEngine, PlaybackStatus, SongInfo, PlaybackQueue};
use musico_recommender::{MusicRecommender, SongRecord, RecommendedSong};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NowPlaying,
    Library,
    Queue,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryViewMode {
    Grid,
    List,
}

pub struct AppState {
    // Navigation
    pub active_view: View,
    pub sidebar_collapsed: bool,
    pub window_width: f32,
    pub window_height: f32,

    // Playback (mirrored from PlaybackEngine events)
    pub current_song: Option<SongInfo>,
    pub playback_status: PlaybackStatus,
    pub position_secs: f32,
    pub duration_secs: f32,
    pub volume: f32,
    pub listened_secs: u32,
    pub is_muted: bool,

    // Library
    pub library: Vec<SongRecord>,
    pub filtered_library: Vec<SongRecord>,
    pub search_query: String,
    pub library_view_mode: LibraryViewMode,

    // Queue & Recommendations
    pub queue: PlaybackQueue,
    pub recommendations: Vec<RecommendedSong>,

    // Dynamic theming
    pub art_dominant_color: Option<Color>,
    pub art_tint: Color,

    // Settings
    pub music_folder: Option<PathBuf>,
    pub is_indexing: bool,
    pub index_progress: (usize, usize), // (done, total)

    // Recommender (owned via Arc/Mutex for shared access if needed)
    pub recommender: Option<Arc<Mutex<MusicRecommender>>>,
    pub playback: Option<Arc<PlaybackEngine>>,
    pub index_rx: Option<tokio::sync::mpsc::Receiver<(usize, usize)>>,
}

impl AppState {
    pub fn new() -> Self {
        let music_folder = dirs::audio_dir();

        Self {
            active_view: View::NowPlaying,
            sidebar_collapsed: false,
            window_width: 900.0,
            window_height: 600.0,

            current_song: None,
            playback_status: PlaybackStatus::Stopped,
            position_secs: 0.0,
            duration_secs: 0.0,
            volume: 1.0,
            listened_secs: 0,
            is_muted: false,

            library: Vec::new(),
            filtered_library: Vec::new(),
            search_query: String::new(),
            library_view_mode: LibraryViewMode::Grid,

            queue: PlaybackQueue::new(),
            recommendations: Vec::new(),

            art_dominant_color: None,
            art_tint: crate::theme::Palette::default_palette().accent,

            music_folder,
            is_indexing: false,
            index_progress: (0, 0),

            recommender: None,
            playback: None,
            index_rx: None,
        }
    }
}
