use std::path::PathBuf;
use iced::Color;
use musico_playback::{PlaybackEngine, PlaybackStatus, SongInfo, PlaybackQueue};
use musico_recommender::{MusicRecommender, SongRecord, RecommendedSong, ListeningStats};
use std::sync::{Arc, Mutex};
use crate::config::AppConfig;
use crate::theme::{self, ColorPalette, FontMode, ThemeCtx};
use crate::timer::SleepTimer;
use crate::lyrics::Lyrics;
use crate::mpris::MprisState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NowPlaying,
    Library,
    Queue,
    Playlists,
    Settings,
    Stats,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationMode {
    Off,
    Track,
    Album,
}

impl NormalizationMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Off => "Off",
            Self::Track => "Track",
            Self::Album => "Album",
        }
    }
    pub fn id(&self) -> &str {
        match self {
            Self::Off => "off",
            Self::Track => "track",
            Self::Album => "album",
        }
    }
    pub fn from_id(id: &str) -> Self {
        match id {
            "track" => Self::Track,
            "album" => Self::Album,
            _ => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Title,
    Artist,
    Album,
    Duration,
    DateAdded,
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available { version: String, url: String },
    Downloading,
    Ready,
    Error(String),
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
    pub is_liked: bool,

    // Library
    pub library: Vec<SongRecord>,
    pub filtered_library: Vec<SongRecord>,
    pub search_query: String,
    pub library_view_mode: LibraryViewMode,
    pub sort_field: SortField,
    pub sort_ascending: bool,

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
    pub cached_art_handle: Option<iced::widget::image::Handle>,

    // Settings
    pub music_folder: Option<PathBuf>,
    pub is_indexing: bool,
    pub index_progress: (usize, usize), // (done, total)

    // Auto-update
    pub update_status: UpdateStatus,

    // ── New features ──────────────────────────────────────────────────

    // EQ (Plan 2)
    pub eq_enabled: bool,
    pub eq_preset_id: String,
    pub eq_gains: [f32; 10],

    // Normalization (Plan 7)
    pub normalization_mode: NormalizationMode,

    // Sleep Timer (Plan 10)
    pub sleep_timer: Option<SleepTimer>,
    pub sleep_timer_volume_factor: f32,

    // Lyrics (Plan 6)
    pub lyrics: Lyrics,
    pub show_lyrics: bool,

    // Stats (Plan 9)
    pub stats: Option<ListeningStats>,
    pub stats_loading: bool,

    // MPRIS (Plan 3)
    pub mpris_state: Option<MprisState>,
    pub mpris_rx: Option<tokio::sync::mpsc::UnboundedReceiver<crate::mpris::MprisCommand>>,

    // Crossfade (Plan 1)
    pub crossfade_config: musico_playback::CrossfadeConfig,

    // Playlists (Plan 4)
    pub playlists: Vec<musico_recommender::SmartPlaylist>,
    pub active_playlist_idx: Option<usize>,

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
        let normalization_mode = NormalizationMode::from_id(&config.normalization_mode);

        let eq_preset = musico_playback::eq::preset_by_id(&config.eq_preset_id);

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
            is_liked: false,

            library: Vec::new(),
            filtered_library: Vec::new(),
            search_query: String::new(),
            library_view_mode,
            sort_field: SortField::Title,
            sort_ascending: true,

            queue: PlaybackQueue::new(),
            recommendations: Vec::new(),

            shuffle_mode: ShuffleMode::Off,
            repeat_mode: RepeatMode::Off,

            art_dominant_color: None,
            art_tint: color_palette.primary,
            color_palette,
            font_mode,
            cached_art_handle: None,

            music_folder: config.music_folder,
            is_indexing: false,
            index_progress: (0, 0),

            update_status: UpdateStatus::Idle,

            // EQ
            eq_enabled: config.eq_enabled,
            eq_preset_id: config.eq_preset_id.clone(),
            eq_gains: eq_preset.gains,

            // Normalization
            normalization_mode,

            // Sleep timer
            sleep_timer: None,
            sleep_timer_volume_factor: 1.0,

            // Lyrics
            lyrics: Lyrics::None,
            show_lyrics: false,

            // Stats
            stats: None,
            stats_loading: false,

            // MPRIS
            mpris_state: None,
            mpris_rx: None,

            // Crossfade
            crossfade_config: musico_playback::CrossfadeConfig {
                enabled: config.crossfade_enabled,
                duration_secs: config.crossfade_duration,
                curve: musico_playback::CrossfadeCurve::from_id(&config.crossfade_curve),
            },

            // Playlists
            playlists: Vec::new(),
            active_playlist_idx: None,

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
