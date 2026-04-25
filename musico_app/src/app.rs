use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use iced::{executor, Application, Command, Element, Length, Subscription, Color};
use iced::widget::{container, row};

use musico_playback::{PlaybackEngine, PlaybackEvent, PlaybackStatus, SongInfo};
use musico_recommender::{MusicRecommender, SongRecord, RecommendedSong};

use crate::state::{AppState, View, LibraryViewMode};
use crate::theme::{self, Palette};
use crate::scanner;

use crate::views::{now_playing, library, queue, settings};
use crate::components::sidebar;

#[derive(Clone)]
pub enum Message {
    // Playback
    PlaySong(SongRecord),
    Pause,
    Resume,
    Stop,
    Previous,
    Next,
    Seek(f32),
    SetVolume(f32),
    ToggleMute,

    // Playback engine events (polled via subscription)
    PlaybackTick,
    PlaybackEvent(PlaybackEvent),

    // Navigation
    NavigateTo(View),
    ToggleSidebar,

    // Library
    SearchChanged(String),
    ToggleLibraryView,
    ScanLibrary,
    IndexProgress(usize, usize),
    IndexComplete(Vec<SongRecord>),

    // Recommendations
    RecommendationsUpdated(Vec<RecommendedSong>),
    AddToQueue(SongRecord),
    RemoveFromQueue(usize),

    // Settings
    MusicFolderChanged(PathBuf),

    // Dynamic theming
    ArtColorExtracted(Color),

    // Window
    WindowResized(f32, f32),
    
    // Setup
    RecommenderReady(Arc<Mutex<MusicRecommender>>),
    PlaybackEngineReady(Arc<PlaybackEngine>),
    LoadAllSongs(Vec<SongRecord>),
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

        // Initialization commands
        let init_recommender = Command::perform(
            async {
                let db_path = "/tmp/musico_db";
                let recommender = MusicRecommender::new(db_path).expect("Failed to init recommender");
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
        "Musico".to_string()
    }

    fn theme(&self) -> iced::Theme {
        theme::musico_theme()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RecommenderReady(rec) => {
                self.0.recommender = Some(rec.clone());
                
                // Load library initially
                let rec_clone = rec.clone();
                return Command::perform(
                    async move {
                        let guard = rec_clone.lock().unwrap();
                        guard.get_all_songs().unwrap_or_default()
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
            Message::PlaySong(record) => {
                if let Some(engine) = &self.0.playback {
                    let song_info = record_to_song_info(&record);
                    let _ = engine.play(song_info);
                    
                    // We might want to notify recommender here, or wait for Playing event.
                    // The prompt specifies we wait for Playing event to extract art, 
                    // but on_song_changed for recommender can be called here or on Playing.
                    // We will call on_song_changed.
                    if let Some(rec) = &self.0.recommender {
                        let mut guard: std::sync::MutexGuard<'_, MusicRecommender> = rec.lock().unwrap();
                        let _ = guard.on_song_changed(&record.id);
                        
                        // Update recommendations
                        let current_id = record.id.clone();
                        if let Ok(recs) = guard.get_recommendations(&current_id, 10) {
                            return Command::perform(async { recs }, Message::RecommendationsUpdated);
                        }
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
            Message::Stop => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.stop();
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
                if let Some(engine) = &self.0.playback {
                    if let Some(next) = self.0.queue.next() {
                        let _ = engine.play(next.clone());
                    }
                }
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
            }
            Message::ToggleMute => {
                if let Some(engine) = &self.0.playback {
                    let _ = engine.toggle_mute();
                }
            }
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
                        
                        // Async task to extract dominant color if there's cover art
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
                            self.0.art_tint = Palette::default_palette().accent;
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
                            let guard: std::sync::MutexGuard<'_, MusicRecommender> = rec.lock().unwrap();
                            let _ = guard.log_listen(&song_id, listened_secs, duration_secs as u32);
                        }
                        
                        // Play next from queue
                        if let Some(next_song) = self.0.queue.next() {
                            return Command::perform(async { next_song }, |_song| {
                                // Find song record
                                // Hack: We convert song_info to song_record for play message
                                // But play message expects song_record.
                                // We can just call play directly on engine.
                                Message::Next
                            });
                        }
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
            Message::NavigateTo(view) => {
                self.0.active_view = view;
            }
            Message::ToggleSidebar => {
                self.0.sidebar_collapsed = !self.0.sidebar_collapsed;
            }
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
            Message::RecommendationsUpdated(recs) => {
                self.0.recommendations = recs;
            }
            Message::AddToQueue(record) => {
                self.0.queue.push_back(record_to_song_info(&record));
            }
            Message::RemoveFromQueue(_) => {
                // Not fully implemented in queue yet
            }
            Message::MusicFolderChanged(path) => {
                self.0.music_folder = Some(path);
            }
            Message::ArtColorExtracted(color) => {
                self.0.art_dominant_color = Some(color);
                self.0.art_tint = color;
            }
            Message::WindowResized(w, h) => {
                self.0.window_width = w;
                self.0.window_height = h;
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let events = iced::event::listen_with(|event, status| match (event, status) {
            (iced::Event::Window(_, iced::window::Event::Resized { width, height }), _) => {
                Some(Message::WindowResized(width as f32, height as f32))
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
                Message::PlaySong,
                Message::AddToQueue,
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
                Message::MusicFolderChanged(PathBuf::from("/tmp/new_music")), // Mock for now
                Message::ScanLibrary,
                |c| { Message::ArtColorExtracted(c) }, // Just set it directly
            ),
        }
    }

    fn view_compact(&self) -> Element<'_, Message> {
        let sidebar = sidebar(&self.0);
        let main = self.main_content();
        
        row![sidebar, main].into()
    }

    fn view_standard(&self) -> Element<'_, Message> {
        let sidebar = sidebar(&self.0);
        let main = self.main_content();
        
        row![sidebar, main].into()
    }

    fn view_wide(&self) -> Element<'_, Message> {
        let sidebar = sidebar(&self.0);
        
        // In wide mode, queue might be visible on the side
        let main = if self.0.active_view == View::Queue {
            self.main_content() // Don't show queue twice
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
            ].into()
        };

        row![sidebar, main].into()
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
        let p = Palette::default_palette();
        iced::widget::container::Appearance {
            background: Some(p.surface.into()),
            border: iced::Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}
