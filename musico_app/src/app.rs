use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use iced::{executor, Application, Command, Element, Length, Subscription, Color};
use iced::widget::{container, row};
use iced::keyboard;

use musico_playback::{PlaybackEngine, PlaybackEvent, PlaybackStatus, SongInfo};
use musico_recommender::{MusicRecommender, SongRecord, RecommendedSong};

use crate::state::{AppState, View, LibraryViewMode, ShuffleMode, RepeatMode};
use crate::theme::{self, ColorPalette, FontMode, Palette};
use crate::scanner;
use crate::config::AppConfig;

use crate::views::{now_playing, library, queue, settings};
use crate::components::sidebar;

#[derive(Clone)]
#[allow(dead_code)]
pub enum Message {
    // Playback
    PlaySong(SongRecord),
    Pause,
    Resume,
    Next,
    Previous,
    Seek(f32),
    SetVolume(f32),
    ToggleMute,

    // Playback engine events (polled via subscription)
    PlaybackTick,
    PlaybackEvent(PlaybackEvent),

    // Navigation
    NavigateTo(View),

    // Library
    SearchChanged(String),
    ToggleLibraryView,
    ScanLibrary,
    IndexProgress(usize, usize),
    IndexComplete(Vec<SongRecord>),

    // Recommendations (fire-and-forget, arrives async)
    RecommendationsUpdated(Vec<RecommendedSong>),
    AddToQueue(SongRecord),
    RemoveFromQueue(usize),

    // Shuffle / Repeat
    ToggleShuffle,
    ToggleRepeat,

    // Settings
    PickFolder,
    MusicFolderChanged(PathBuf),
    ArtColorExtracted(Color),
    SetPalette(ColorPalette),
    SetFontMode(FontMode),

    // Window
    WindowResized(f32, f32),
    
    // Setup
    RecommenderReady(Arc<Mutex<MusicRecommender>>),
    PlaybackEngineReady(Arc<PlaybackEngine>),
    LoadAllSongs(Vec<SongRecord>),

    // Keyboard
    KeyPressed(keyboard::Key),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message")
    }
}

pub struct Musico(pub AppState);

impl Application for Musico {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let state = AppState::new();

        let init_recommender = Command::perform(
            async {
                let db_path = dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("musico")
                    .join("db");
                std::fs::create_dir_all(&db_path).ok();
                let db_str = db_path.to_string_lossy().to_string();
                let recommender = MusicRecommender::new(&db_str).expect("Failed to init recommender");
                Arc::new(Mutex::new(recommender))
            },
            Message::RecommenderReady,
        );

        let init_playback = Command::perform(
            async {
                let engine = PlaybackEngine::new().expect("Failed to init playback engine");
                Arc::new(engine)
            },
            Message::PlaybackEngineReady,
        );

        (Self(state), Command::batch(vec![init_recommender, init_playback]))
    }

    fn title(&self) -> String {
        if let Some(song) = &self.0.current_song {
            format!("{} — Musico", song.title)
        } else {
            "Musico".to_string()
        }
    }

    fn theme(&self) -> iced::Theme {
        theme::musico_theme()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RecommenderReady(rec) => {
                self.0.recommender = Some(rec.clone());
                
                let rec_clone = rec.clone();
                return Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let guard = rec_clone.lock().unwrap();
                            guard.get_all_songs().unwrap_or_default()
                        }).await.unwrap_or_default()
                    },
                    Message::LoadAllSongs,
                );
            }
            Message::PlaybackEngineReady(engine) => {
                self.0.playback = Some(engine);
            }
            Message::LoadAllSongs(songs) => {
                self.0.library = songs.clone();
                self.0.filtered_library = songs;
            }

            // ── Playback ──────────────────────────────────────────────
            Message::PlaySong(record) => {
                if let Some(engine) = &self.0.playback {
                    let song_info = record_to_song_info(&record);
                    let _ = engine.play(song_info);
                    
                    if let Some(rec) = &self.0.recommender {
                        let rec_clone = rec.clone();
                        let current_id = record.id.clone();
                        return Command::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    let mut guard = rec_clone.lock().unwrap();
                                    let _ = guard.on_song_changed(&current_id);
                                    guard.get_recommendations(&current_id, 10).unwrap_or_default()
                                }).await.unwrap_or_default()
                            },
                            Message::RecommendationsUpdated,
                        );
                    }
                }
            }
            Message::Pause => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.pause();
                }
            }
            Message::Resume => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.resume();
                }
            }
            Message::Previous => {
                if let Some(engine) = &self.0.playback {
                    if let Some(prev) = self.0.queue.previous() {
                        let _ = engine.play(prev.clone());
                    }
                }
            }
            Message::Next => {
                self.play_next();
                return Command::none();
            }
            Message::Seek(secs) => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.seek(secs);
                }
            }
            Message::SetVolume(vol) => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.set_volume(vol);
                }
                self.0.volume = vol;
                self.save_config();
            }
            Message::ToggleMute => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.toggle_mute();
                }
            }

            // ── Playback Polling ──────────────────────────────────────
            Message::PlaybackTick => {
                let mut cmds = Vec::new();
                if let Some(engine) = &self.0.playback {
                    let events = engine.poll_events();
                    for ev in events {
                        cmds.push(Command::perform(async { ev }, Message::PlaybackEvent));
                    }
                }
                if let Some(rx) = &mut self.0.index_rx {
                    while let Ok(prog) = rx.try_recv() {
                        cmds.push(Command::perform(async move { prog }, |p| Message::IndexProgress(p.0, p.1)));
                    }
                }
                return Command::batch(cmds);
            }
            Message::PlaybackEvent(ev) => {
                match ev {
                    PlaybackEvent::Playing(song) => {
                        self.0.playback_status = PlaybackStatus::Playing;
                        self.0.current_song = Some(song.clone());
                        
                        if let Some(art_bytes) = &song.cover_art {
                            let bytes_clone = art_bytes.clone();
                            return Command::perform(
                                async move {
                                    if let Ok(img) = image::load_from_memory(&bytes_clone) {
                                        crate::components::art_canvas::extract_dominant_color(&img)
                                    } else {
                                        Palette::default_palette().accent
                                    }
                                },
                                Message::ArtColorExtracted
                            );
                        } else {
                            self.0.art_tint = self.0.color_palette.primary;
                        }
                    }
                    PlaybackEvent::Paused { position_secs } => {
                        self.0.playback_status = PlaybackStatus::Paused;
                        self.0.position_secs = position_secs;
                    }
                    PlaybackEvent::Resumed => {
                        self.0.playback_status = PlaybackStatus::Playing;
                    }
                    PlaybackEvent::Stopped => {
                        self.0.playback_status = PlaybackStatus::Stopped;
                        self.0.current_song = None;
                        self.0.position_secs = 0.0;
                    }
                    PlaybackEvent::PositionUpdate { position_secs, listened_secs } => {
                        self.0.position_secs = position_secs;
                        self.0.listened_secs = listened_secs;
                    }
                    PlaybackEvent::SongEnded { song_id, listened_secs, duration_secs } => {
                        if let Some(rec) = &self.0.recommender {
                            let guard = rec.lock().unwrap();
                            let _ = guard.log_listen(&song_id, listened_secs, duration_secs as u32);
                        }
                        
                        if self.0.repeat_mode == RepeatMode::One {
                            if let Some(song) = &self.0.current_song {
                                if let Some(engine) = &self.0.playback {
                                    let _ = engine.play(song.clone());
                                }
                            }
                            return Command::none();
                        }

                        self.play_next();
                    }
                    PlaybackEvent::BufferingStarted => {
                        self.0.playback_status = PlaybackStatus::Buffering;
                    }
                    PlaybackEvent::BufferingComplete => {
                        self.0.playback_status = PlaybackStatus::Playing;
                    }
                    PlaybackEvent::VolumeChanged(v) => {
                        self.0.volume = v;
                    }
                    PlaybackEvent::Seeked { position_secs } => {
                        self.0.position_secs = position_secs;
                    }
                    _ => {}
                }
            }

            // ── Navigation ────────────────────────────────────────────
            Message::NavigateTo(view) => {
                self.0.active_view = view;
            }

            // ── Library ───────────────────────────────────────────────
            Message::SearchChanged(q) => {
                self.0.search_query = q.clone();
                let lower_q = q.to_lowercase();
                if lower_q.is_empty() {
                    self.0.filtered_library = self.0.library.clone();
                } else {
                    self.0.filtered_library = self.0.library.iter()
                        .filter(|s| s.title.to_lowercase().contains(&lower_q) || s.artist.to_lowercase().contains(&lower_q))
                        .cloned()
                        .collect();
                }
            }
            Message::ToggleLibraryView => {
                self.0.library_view_mode = match self.0.library_view_mode {
                    LibraryViewMode::Grid => LibraryViewMode::List,
                    LibraryViewMode::List => LibraryViewMode::Grid,
                };
                self.save_config();
            }
            Message::ScanLibrary => {
                if let Some(folder) = &self.0.music_folder {
                    if let Some(rec) = &self.0.recommender {
                        self.0.is_indexing = true;
                        self.0.index_progress = (0, 0);
                        let folder_clone = folder.clone();
                        let rec_clone: Arc<Mutex<MusicRecommender>> = rec.clone();

                        let (tx, rx) = mpsc::channel(100);
                        self.0.index_rx = Some(rx);

                        return Command::perform(
                            async move {
                                scanner::scan_and_index(folder_clone, rec_clone, tx).await
                            },
                            Message::IndexComplete,
                        );
                    }
                }
            }
            Message::IndexProgress(done, total) => {
                self.0.index_progress = (done, total);
            }
            Message::IndexComplete(records) => {
                self.0.is_indexing = false;
                self.0.library = records.clone();
                self.0.filtered_library = records;
            }

            // ── Recommendations ───────────────────────────────────────
            Message::RecommendationsUpdated(recs) => {
                self.0.recommendations = recs;
            }
            Message::AddToQueue(record) => {
                self.0.queue.push_back(record_to_song_info(&record));
            }
            Message::RemoveFromQueue(idx) => {
                self.0.queue.remove_at(idx);
            }

            // ── Shuffle / Repeat ──────────────────────────────────────
            Message::ToggleShuffle => {
                self.0.shuffle_mode = match self.0.shuffle_mode {
                    ShuffleMode::Off => ShuffleMode::Shuffle,
                    ShuffleMode::Shuffle => ShuffleMode::SmartRadio,
                    ShuffleMode::SmartRadio => ShuffleMode::Off,
                };
                if self.0.shuffle_mode == ShuffleMode::Shuffle {
                    self.0.queue.shuffle();
                }
            }
            Message::ToggleRepeat => {
                self.0.repeat_mode = match self.0.repeat_mode {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };
            }

            // ── Settings ──────────────────────────────────────────────
            Message::PickFolder => {
                return Command::perform(
                    async {
                        let folder = rfd::AsyncFileDialog::new()
                            .set_title("Select Music Folder")
                            .pick_folder()
                            .await;
                        folder.map(|f| f.path().to_path_buf())
                    },
                    |maybe_path| {
                        if let Some(path) = maybe_path {
                            Message::MusicFolderChanged(path)
                        } else {
                            Message::NavigateTo(View::Settings)
                        }
                    },
                );
            }
            Message::MusicFolderChanged(path) => {
                self.0.music_folder = Some(path);
                self.save_config();
            }
            Message::ArtColorExtracted(color) => {
                self.0.art_dominant_color = Some(color);
                self.0.art_tint = color;
            }
            Message::SetPalette(palette) => {
                self.0.color_palette = palette;
                // Reset art_tint to palette primary when no song playing
                if self.0.current_song.is_none() {
                    self.0.art_tint = palette.primary;
                }
                self.save_config();
            }
            Message::SetFontMode(mode) => {
                self.0.font_mode = mode;
                self.save_config();
            }

            // ── Window ────────────────────────────────────────────────
            Message::WindowResized(w, h) => {
                self.0.window_width = w;
                self.0.window_height = h;
            }

            // ── Keyboard ──────────────────────────────────────────────
            Message::KeyPressed(key) => {
                return self.handle_key(key);
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let events = iced::event::listen_with(|event, _status| match event {
            iced::Event::Window(_, iced::window::Event::Resized { width, height }) => {
                Some(Message::WindowResized(width as f32, height as f32))
            }
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                match &key {
                    keyboard::Key::Named(named) => match named {
                        keyboard::key::Named::Space => Some(Message::KeyPressed(key)),
                        keyboard::key::Named::ArrowLeft => Some(Message::KeyPressed(key)),
                        keyboard::key::Named::ArrowRight => Some(Message::KeyPressed(key)),
                        keyboard::key::Named::ArrowUp => Some(Message::KeyPressed(key)),
                        keyboard::key::Named::ArrowDown => Some(Message::KeyPressed(key)),
                        keyboard::key::Named::Escape => Some(Message::KeyPressed(key)),
                        _ => None,
                    },
                    keyboard::Key::Character(c) => {
                        let ch = c.as_str();
                        match ch {
                            "n" | "p" | "s" | "r" if !modifiers.shift() => {
                                Some(Message::KeyPressed(key))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        });

        let tick = iced::time::every(Duration::from_millis(250))
            .map(|_| Message::PlaybackTick);

        Subscription::batch(vec![events, tick])
    }

    fn view(&self) -> Element<'_, Message> {
        let width = self.0.window_width;
        let layout = if width < 700.0 {
            Layout::Compact
        } else if width < 1100.0 {
            Layout::Standard
        } else {
            Layout::Wide
        };

        let content = match layout {
            Layout::Compact => self.view_compact(),
            Layout::Standard => self.view_standard(),
            Layout::Wide => self.view_wide(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(iced::theme::Container::Custom(Box::new(BaseContainerStyle)))
            .into()
    }
}

struct BaseContainerStyle;
impl iced::widget::container::StyleSheet for BaseContainerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(crate::theme::BASE)),
            text_color: Some(crate::theme::TEXT_PRIMARY),
            ..Default::default()
        }
    }
}

enum Layout { Compact, Standard, Wide }

impl Musico {
    fn main_content(&self) -> Element<'_, Message> {
        match self.0.active_view {
            View::NowPlaying => now_playing(
                &self.0,
                if matches!(self.0.playback_status, PlaybackStatus::Playing) { Message::Pause } else { Message::Resume },
                Message::Previous,
                Message::Next,
                Message::Seek,
                Message::SetVolume,
                Message::PlaySong,
                Message::AddToQueue,
                Message::ToggleShuffle,
                Message::ToggleRepeat,
            ),
            View::Library => library(
                &self.0,
                Message::SearchChanged,
                Message::SearchChanged(String::new()),
                Message::ToggleLibraryView,
                Message::PlaySong,
                Message::AddToQueue,
            ),
            View::Queue => queue(
                &self.0,
                Message::PlaySong,
                Message::RemoveFromQueue,
                Message::PlaySong,
                Message::AddToQueue,
            ),
            View::Settings => settings(
                &self.0,
                Message::PickFolder,
                Message::ScanLibrary,
                |p| Message::SetPalette(p),
                |m| Message::SetFontMode(m),
            ),
        }
    }

    fn view_compact(&self) -> Element<'_, Message> {
        let sidebar = sidebar(&self.0);
        let main = self.main_content();
        
        row![sidebar, main].spacing(12).into()
    }

    fn view_standard(&self) -> Element<'_, Message> {
        let sidebar = sidebar(&self.0);
        let main = self.main_content();
        
        row![sidebar, main].spacing(12).into()
    }

    fn view_wide(&self) -> Element<'_, Message> {
        let sidebar = sidebar(&self.0);
        
        let main = if self.0.active_view == View::Queue {
            self.main_content()
        } else {
            let q_panel = queue(
                &self.0,
                Message::PlaySong,
                Message::RemoveFromQueue,
                Message::PlaySong,
                Message::AddToQueue,
            );
            
            row![
                container(self.main_content()).width(Length::Fill),
                container(q_panel).width(Length::Fixed(280.0)).style(iced::theme::Container::Custom(Box::new(QueuePanelStyle)))
            ].spacing(12).into()
        };

        row![sidebar, main].spacing(12).into()
    }

    fn handle_key(&mut self, key: keyboard::Key) -> Command<Message> {
        match key {
            keyboard::Key::Named(keyboard::key::Named::Space) => {
                if matches!(self.0.playback_status, PlaybackStatus::Playing) {
                    if let Some(engine) = &self.0.playback {
                        let _ = engine.pause();
                    }
                } else if matches!(self.0.playback_status, PlaybackStatus::Paused) {
                    if let Some(engine) = &self.0.playback {
                        let _ = engine.resume();
                    }
                }
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                let new_pos = (self.0.position_secs - 5.0).max(0.0);
                if let Some(engine) = &self.0.playback {
                    let _ = engine.seek(new_pos);
                }
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                let new_pos = (self.0.position_secs + 5.0).min(self.0.duration_secs);
                if let Some(engine) = &self.0.playback {
                    let _ = engine.seek(new_pos);
                }
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                let new_vol = (self.0.volume + 0.05).clamp(0.0, 1.0);
                if let Some(engine) = &self.0.playback {
                    let _ = engine.set_volume(new_vol);
                }
                self.0.volume = new_vol;
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                let new_vol = (self.0.volume - 0.05).clamp(0.0, 1.0);
                if let Some(engine) = &self.0.playback {
                    let _ = engine.set_volume(new_vol);
                }
                self.0.volume = new_vol;
            }
            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                if !self.0.search_query.is_empty() {
                    self.0.search_query.clear();
                    self.0.filtered_library = self.0.library.clone();
                } else {
                    self.0.active_view = View::NowPlaying;
                }
            }
            keyboard::Key::Character(ref c) => match c.as_str() {
                "n" => { self.play_next(); }
                "p" => {
                    if let Some(engine) = &self.0.playback {
                        if let Some(prev) = self.0.queue.previous() {
                            let _ = engine.play(prev.clone());
                        }
                    }
                }
                "s" => {
                    self.0.shuffle_mode = match self.0.shuffle_mode {
                        ShuffleMode::Off => ShuffleMode::Shuffle,
                        ShuffleMode::Shuffle => ShuffleMode::SmartRadio,
                        ShuffleMode::SmartRadio => ShuffleMode::Off,
                    };
                }
                "r" => {
                    self.0.repeat_mode = match self.0.repeat_mode {
                        RepeatMode::Off => RepeatMode::All,
                        RepeatMode::All => RepeatMode::One,
                        RepeatMode::One => RepeatMode::Off,
                    };
                }
                _ => {}
            },
            _ => {}
        }
        Command::none()
    }

    fn play_next(&mut self) {
        if let Some(next_song) = self.0.queue.next() {
            if let Some(engine) = &self.0.playback {
                let _ = engine.play(next_song);
            }
            return;
        }

        if self.0.shuffle_mode == ShuffleMode::SmartRadio && !self.0.recommendations.is_empty() {
            let rec = self.0.recommendations.remove(0);
            let song_info = record_to_song_info(&rec.record);
            if let Some(engine) = &self.0.playback {
                let _ = engine.play(song_info);
            }
            return;
        }

        if self.0.shuffle_mode == ShuffleMode::Shuffle && !self.0.library.is_empty() {
            use rand::Rng;
            let idx = rand::thread_rng().gen_range(0..self.0.library.len());
            let record = self.0.library[idx].clone();
            let song_info = record_to_song_info(&record);
            if let Some(engine) = &self.0.playback {
                let _ = engine.play(song_info);
            }
            return;
        }

        if self.0.repeat_mode == RepeatMode::All && !self.0.library.is_empty() {
            let record = self.0.library[0].clone();
            let song_info = record_to_song_info(&record);
            if let Some(engine) = &self.0.playback {
                let _ = engine.play(song_info);
            }
        }
    }

    fn save_config(&self) {
        let mut config = AppConfig::load();
        config.music_folder = self.0.music_folder.clone();
        config.volume = self.0.volume;
        config.palette_id = self.0.color_palette.id.to_string();
        config.font_mode = self.0.font_mode.id().to_string();
        config.library_view_mode = match self.0.library_view_mode {
            LibraryViewMode::Grid => "grid".to_string(),
            LibraryViewMode::List => "list".to_string(),
        };
        config.save();
    }
}

fn record_to_song_info(record: &SongRecord) -> SongInfo {
    SongInfo {
        id: record.id.clone(),
        file_path: record.file_path.clone(),
        title: record.title.clone(),
        artist: record.artist.clone(),
        album: record.album.clone(),
        duration_secs: record.duration_secs as f32,
        cover_art: None,
    }
}

struct QueuePanelStyle;
impl iced::widget::container::StyleSheet for QueuePanelStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(crate::theme::SURFACE)),
            border: iced::Border {
                color: crate::theme::BORDER_SUBTLE,
                width: 1.0,
                radius: 24.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.3, ..crate::theme::BASE },
                offset: iced::Vector { x: 0.0, y: 10.0 },
                blur_radius: 30.0,
            },
            ..Default::default()
        }
    }
}
