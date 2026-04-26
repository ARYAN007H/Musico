use std::path::PathBuf;
use iced::Color;
use musico_playback::{PlaybackEngine, PlaybackStatus, SongInfo, PlaybackQueue};
use musico_recommender::{MusicRecommender, SongRecord, RecommendedSong};
use std::sync::{Arc, Mutex};
use crate::config::AppConfig;
use crate::theme::{self, ColorPalette, FontMode, ThemeCtx};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuffleMode {
    Off,
    Shuffle,
    SmartRadio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

pub struct AppState {
    // Navigation
    pub active_view: View,
    #[allow(dead_code)]
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

    // Library
    pub library: Vec<SongRecord>,
    pub filtered_library: Vec<SongRecord>,
    pub search_query: String,
    pub library_view_mode: LibraryViewMode,

    // Queue & Recommendations
    pub queue: PlaybackQueue,
    pub recommendations: Vec<RecommendedSong>,

    // Playback modes
    pub shuffle_mode: ShuffleMode,
    pub repeat_mode: RepeatMode,

    // Dynamic theming
    pub art_dominant_color: Option<Color>,
    pub art_tint: Color,
    pub color_palette: ColorPalette,
    pub font_mode: FontMode,

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
        let config = AppConfig::load();

        let library_view_mode = if config.library_view_mode == "list" {
            LibraryViewMode::List
        } else {
            LibraryViewMode::Grid
        };

        let color_palette = theme::palette_by_id(&config.palette_id);
        let font_mode = FontMode::from_id(&config.font_mode);

        Self {
            active_view: View::NowPlaying,
            sidebar_collapsed: false,
            window_width: 900.0,
            window_height: 600.0,

            current_song: None,
            playback_status: PlaybackStatus::Stopped,
            position_secs: 0.0,
            duration_secs: 0.0,
            volume: config.volume,
            listened_secs: 0,

            library: Vec::new(),
            filtered_library: Vec::new(),
            search_query: String::new(),
            library_view_mode,

            queue: PlaybackQueue::new(),
            recommendations: Vec::new(),

            shuffle_mode: ShuffleMode::Off,
            repeat_mode: RepeatMode::Off,

            art_dominant_color: None,
            art_tint: color_palette.primary,
            color_palette,
            font_mode,

            music_folder: config.music_folder,
            is_indexing: false,
            index_progress: (0, 0),

            recommender: None,
            playback: None,
            index_rx: None,
        }
    }

    /// Build a ThemeCtx snapshot for use in views.
    pub fn theme_ctx(&self) -> ThemeCtx {
        ThemeCtx::new(self.color_palette, self.font_mode)
    }

    /// Is the sidebar in compact (icon-only) mode?
    pub fn is_compact(&self) -> bool {
        theme::is_compact(self.window_width)
    }
}
