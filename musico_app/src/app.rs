use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use iced::{executor, Application, Command, Element, Length, Subscription, Color};
use iced::widget::{container, row};
use iced::keyboard;

use musico_playback::{PlaybackEngine, PlaybackEvent, PlaybackStatus, SongInfo};
use musico_playback::eq;
use musico_recommender::{MusicRecommender, SongRecord, RecommendedSong, ListeningStats};

use crate::state::{AppState, View, LibraryViewMode, ShuffleMode, RepeatMode, NormalizationMode};
use crate::theme::{self, ColorPalette, FontMode, Palette};
use crate::scanner;
use crate::config::AppConfig;
use crate::timer::{SleepTimer, TimerStatus};
use crate::lyrics::{self, Lyrics};
use crate::mpris::{self, MprisCommand, MprisMetadata};

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

    // Actions
    ToggleLike,

    // Settings
    PickFolder,
    MusicFolderChanged(PathBuf),
    ArtColorExtracted(Color),
    SetPalette(ColorPalette),
    SetFontMode(FontMode),

    // Auto-update
    CheckForUpdate,
    UpdateAvailable(String, String), // (version, download_url)
    DownloadUpdate(String),          // download_url
    UpdateDownloaded,
    UpdateError(String),

    // Window
    WindowResized(f32, f32),
    
    // Setup
    RecommenderReady(Arc<Mutex<MusicRecommender>>),
    PlaybackEngineReady(Arc<PlaybackEngine>),
    LoadAllSongs(Vec<SongRecord>),

    // Keyboard
    KeyPressed(keyboard::Key),

    // ── Plan 2: EQ ──────────────────────────────────────────────────
    SetEQPreset(String),
    SetEQBand(usize, f32),
    ToggleEQ,

    // ── Plan 3: MPRIS ───────────────────────────────────────────────
    MprisReady(std::sync::Arc<std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<MprisCommand>>>>, mpris::MprisState),
    MprisIncoming(MprisCommand),

    // ── Plan 6: Lyrics ──────────────────────────────────────────────
    LyricsLoaded(Lyrics),
    ToggleLyrics,

    // ── Plan 7: Normalization ───────────────────────────────────────
    SetNormalizationMode(NormalizationMode),

    // ── Plan 9: Stats ───────────────────────────────────────────────
    StatsLoaded(ListeningStats),
    RefreshStats,

    // ── Plan 10: Sleep Timer ────────────────────────────────────────
    SetSleepTimer(u64),   // minutes (0 = cancel)
    SleepTimerExpired,

    // ── Plan 1: Crossfade ───────────────────────────────────────────
    ToggleCrossfade,
    SetCrossfadeDuration(f32),
    SetCrossfadeCurve(String),
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

        let init_mpris = Command::perform(
            async {
                match mpris::start_mpris_server().await {
                    Ok((rx, state)) => Some((rx, state)),
                    Err(e) => {
                        log::warn!("MPRIS init failed (non-fatal): {e}");
                        None
                    }
                }
            },
            |result| match result {
                Some((rx, state)) => Message::MprisReady(
                    std::sync::Arc::new(std::sync::Mutex::new(Some(rx))),
                    state,
                ),
                None => Message::NavigateTo(View::NowPlaying),
            },
        );

        (Self(state), Command::batch(vec![init_recommender, init_playback, init_mpris]))
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

                    // ── Load lyrics from sidecar .lrc file ──
                    let file_path = record.file_path.clone();
                    let lyrics_cmd = Command::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                lyrics::load_sidecar_lrc(&file_path)
                            }).await.unwrap_or(Lyrics::None)
                        },
                        Message::LyricsLoaded,
                    );

                    // ── Apply normalization gain ──
                    if !matches!(self.0.normalization_mode, NormalizationMode::Off) {
                        self.apply_normalization();
                    }

                    // ── Update MPRIS ──
                    self.update_mpris();

                    if let Some(rec) = &self.0.recommender {
                        let rec_clone = rec.clone();
                        let current_id = record.id.clone();
                        let recs_cmd = Command::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    let mut guard = rec_clone.lock().unwrap();
                                    let _ = guard.on_song_changed(&current_id);
                                    guard.get_recommendations(&current_id, 10).unwrap_or_default()
                                }).await.unwrap_or_default()
                            },
                            Message::RecommendationsUpdated,
                        );
                        return Command::batch(vec![lyrics_cmd, recs_cmd]);
                    }
                    return lyrics_cmd;
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

                // ── Sleep Timer tick ──
                if let Some(timer) = &self.0.sleep_timer {
                    match timer.tick() {
                        TimerStatus::Active { .. } => {}
                        TimerStatus::Fading { volume_factor, .. } => {
                            self.0.sleep_timer_volume_factor = volume_factor;
                            // Modulate the actual volume.
                            let effective = self.0.volume * volume_factor;
                            if let Some(engine) = &self.0.playback {
                                let _ = engine.set_volume(effective);
                            }
                        }
                        TimerStatus::Expired => {
                            cmds.push(Command::perform(async {}, |_| Message::SleepTimerExpired));
                        }
                    }
                }

                // ── MPRIS D-Bus command polling ──
                if let Some(rx) = &mut self.0.mpris_rx {
                    while let Ok(cmd) = rx.try_recv() {
                        cmds.push(Command::perform(async { cmd }, Message::MprisIncoming));
                    }
                }

                // ── Update MPRIS metadata (every tick) ──
                self.update_mpris();

                return Command::batch(cmds);
            }
            Message::PlaybackEvent(ev) => {
                match ev {
                    PlaybackEvent::Playing(song) => {
                        self.0.playback_status = PlaybackStatus::Playing;
                        self.0.duration_secs = song.duration_secs;
                        self.0.position_secs = 0.0;
                        self.0.current_song = Some(song.clone());
                        
                        if let Some(art_bytes) = &song.cover_art {
                            self.0.cached_art_handle = Some(iced::widget::image::Handle::from_memory(art_bytes.clone()));
                            let bytes_clone = art_bytes.clone();
                            return Command::perform(
                                async move {
                                    if let Ok(img) = image::load_from_memory(&bytes_clone) {
                                        crate::components::art_canvas::extract_dominant_color(&img)
                                    } else {
                                        crate::theme::Palette::default_palette().accent
                                    }
                                },
                                Message::ArtColorExtracted
                            );
                        } else {
                            self.0.cached_art_handle = None;
                            self.0.art_tint = self.0.color_palette.primary;
                        }

                        // ── Preload next track for gapless transition ──
                        if let Some(engine) = &self.0.playback {
                            if let Some(next_song) = self.0.queue.peek_next() {
                                let _ = engine.preload_next(next_song);
                            }
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
                        self.0.cached_art_handle = None;
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
                // Auto-load stats on first visit to Stats view.
                if view == View::Stats && self.0.stats.is_none() && !self.0.stats_loading {
                    if let Some(rec) = &self.0.recommender {
                        self.0.stats_loading = true;
                        let rec_clone = rec.clone();
                        return Command::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    let guard = rec_clone.lock().unwrap();
                                    guard.get_stats().unwrap_or_default()
                                }).await.unwrap_or_default()
                            },
                            Message::StatsLoaded,
                        );
                    }
                }
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

            // ── Actions ───────────────────────────────────────────────
            Message::ToggleLike => {
                // Toggle liked status — for now just log, future: persist in recommender
                if let Some(song) = &self.0.current_song {
                    self.0.is_liked = !self.0.is_liked;
                    log::info!("Toggled like for: {} → {}", song.title, self.0.is_liked);
                }
            }

            // ── Auto-Update ───────────────────────────────────────────
            Message::CheckForUpdate => {
                self.0.update_status = crate::state::UpdateStatus::Checking;
                return Command::perform(
                    crate::updater::check_for_update(),
                    |result| match result {
                        Ok(Some((version, url))) => Message::UpdateAvailable(version, url),
                        Ok(None) => Message::UpdateError("You're on the latest version ✓".into()),
                        Err(e) => Message::UpdateError(format!("{e}")),
                    },
                );
            }
            Message::UpdateAvailable(version, url) => {
                self.0.update_status = crate::state::UpdateStatus::Available {
                    version: version.clone(),
                    url: url.clone(),
                };
            }
            Message::DownloadUpdate(url) => {
                self.0.update_status = crate::state::UpdateStatus::Downloading;
                return Command::perform(
                    crate::updater::download_and_install(url),
                    |result| match result {
                        Ok(()) => Message::UpdateDownloaded,
                        Err(e) => Message::UpdateError(format!("{e}")),
                    },
                );
            }
            Message::UpdateDownloaded => {
                self.0.update_status = crate::state::UpdateStatus::Ready;
            }
            Message::UpdateError(msg) => {
                self.0.update_status = crate::state::UpdateStatus::Error(msg);
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

            // ── Plan 2: EQ ───────────────────────────────────────────
            Message::ToggleEQ => {
                self.0.eq_enabled = !self.0.eq_enabled;
                if let Some(engine) = &self.0.playback {
                    let _ = engine.set_eq(self.0.eq_enabled, self.0.eq_gains);
                }
                self.save_config();
            }
            Message::SetEQPreset(preset_id) => {
                let preset = eq::preset_by_id(&preset_id);
                self.0.eq_preset_id = preset_id;
                self.0.eq_gains = preset.gains;
                self.0.eq_enabled = preset.id != "flat";
                if let Some(engine) = &self.0.playback {
                    let _ = engine.set_eq(self.0.eq_enabled, self.0.eq_gains);
                }
                self.save_config();
            }
            Message::SetEQBand(band_idx, gain_db) => {
                if band_idx < 10 {
                    self.0.eq_gains[band_idx] = gain_db.clamp(-12.0, 12.0);
                    self.0.eq_preset_id = "custom".to_string();
                    if let Some(engine) = &self.0.playback {
                        let _ = engine.set_eq(self.0.eq_enabled, self.0.eq_gains);
                    }
                }
            }

            // ── Plan 3: MPRIS ────────────────────────────────────────
            Message::MprisReady(rx_arc, state) => {
                self.0.mpris_state = Some(state);
                if let Ok(mut guard) = rx_arc.lock() {
                    self.0.mpris_rx = guard.take();
                }
            }
            Message::MprisIncoming(cmd) => {
                match cmd {
                    MprisCommand::PlayPause => {
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
                    MprisCommand::Next => { self.play_next(); }
                    MprisCommand::Previous => {
                        if let Some(engine) = &self.0.playback {
                            if let Some(prev) = self.0.queue.previous() {
                                let _ = engine.play(prev.clone());
                            }
                        }
                    }
                    MprisCommand::Stop => {
                        if let Some(engine) = &self.0.playback {
                            let _ = engine.stop();
                        }
                    }
                    MprisCommand::Seek(offset_us) => {
                        let offset_secs = (offset_us / 1_000_000.0) as f32;
                        let new_pos = (self.0.position_secs + offset_secs).max(0.0).min(self.0.duration_secs);
                        if let Some(engine) = &self.0.playback {
                            let _ = engine.seek(new_pos);
                        }
                    }
                    MprisCommand::SetVolume(vol) => {
                        let v = (vol as f32).clamp(0.0, 1.0);
                        if let Some(engine) = &self.0.playback {
                            let _ = engine.set_volume(v);
                        }
                        self.0.volume = v;
                    }
                }
            }

            // ── Plan 6: Lyrics ───────────────────────────────────────
            Message::LyricsLoaded(lyr) => {
                self.0.lyrics = lyr;
            }
            Message::ToggleLyrics => {
                self.0.show_lyrics = !self.0.show_lyrics;
            }

            // ── Plan 7: Normalization ────────────────────────────────
            Message::SetNormalizationMode(mode) => {
                self.0.normalization_mode = mode;
                self.apply_normalization();
                self.save_config();
            }

            // ── Plan 9: Stats ────────────────────────────────────────
            Message::StatsLoaded(stats) => {
                self.0.stats = Some(stats);
                self.0.stats_loading = false;
            }
            Message::RefreshStats => {
                if let Some(rec) = &self.0.recommender {
                    self.0.stats_loading = true;
                    let rec_clone = rec.clone();
                    return Command::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                let guard = rec_clone.lock().unwrap();
                                guard.get_stats().unwrap_or_default()
                            }).await.unwrap_or_default()
                        },
                        Message::StatsLoaded,
                    );
                }
            }

            // ── Plan 10: Sleep Timer ─────────────────────────────────
            Message::SetSleepTimer(mins) => {
                if mins == 0 {
                    self.0.sleep_timer = None;
                    self.0.sleep_timer_volume_factor = 1.0;
                } else {
                    self.0.sleep_timer = Some(SleepTimer::new(mins));
                    self.0.sleep_timer_volume_factor = 1.0;
                    // Save last used duration.
                    let mut config = AppConfig::load();
                    config.last_sleep_timer_mins = mins;
                    config.save();
                }
            }
            Message::SleepTimerExpired => {
                self.0.sleep_timer = None;
                self.0.sleep_timer_volume_factor = 1.0;
                // Pause playback.
                if let Some(engine) = &self.0.playback {
                    let _ = engine.pause();
                }
            }

            // ── Plan 1: Crossfade ────────────────────────────────────
            Message::ToggleCrossfade => {
                self.0.crossfade_config.enabled = !self.0.crossfade_config.enabled;
                self.sync_crossfade();
                self.save_config();
            }
            Message::SetCrossfadeDuration(secs) => {
                self.0.crossfade_config.duration_secs = secs;
                self.sync_crossfade();
                self.save_config();
            }
            Message::SetCrossfadeCurve(id) => {
                self.0.crossfade_config.curve = musico_playback::CrossfadeCurve::from_id(&id);
                self.sync_crossfade();
                self.save_config();
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
                            "n" | "p" | "s" | "r" | "l" if !modifiers.shift() => {
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
            .style(iced::theme::Container::Custom(Box::new(BaseContainerStyle(self.0.art_tint))))
            .into()
    }
}

struct BaseContainerStyle(Color);
impl iced::widget::container::StyleSheet for BaseContainerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let mix = 0.08;
        let base = crate::theme::BASE;
        let r = base.r * (1.0 - mix) + self.0.r * mix;
        let g = base.g * (1.0 - mix) + self.0.g * mix;
        let b = base.b * (1.0 - mix) + self.0.b * mix;
        let ambient_bg = Color::from_rgb(r, g, b);

        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(ambient_bg)),
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
                Message::ToggleLike,
                Message::AddToQueue,
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
            View::Settings => {
                let general = settings(
                    &self.0,
                    Message::PickFolder,
                    Message::ScanLibrary,
                    |p| Message::SetPalette(p),
                    |m| Message::SetFontMode(m),
                    Message::CheckForUpdate,
                    |url| Message::DownloadUpdate(url),
                );
                let audio = crate::views::settings::audio_settings(&self.0);
                iced::widget::column![general, audio].spacing(20).into()
            },
            View::Stats => {
                // Auto-load stats on first visit.
                if self.0.stats.is_none() && !self.0.stats_loading {
                    // Can't return Command from view, just show loading.
                }
                crate::views::stats::stats_view(&self.0)
            }
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
                "l" => {
                    self.0.show_lyrics = !self.0.show_lyrics;
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
        config.eq_enabled = self.0.eq_enabled;
        config.eq_preset_id = self.0.eq_preset_id.clone();
        config.normalization_mode = self.0.normalization_mode.id().to_string();
        config.crossfade_enabled = self.0.crossfade_config.enabled;
        config.crossfade_duration = self.0.crossfade_config.duration_secs;
        config.crossfade_curve = self.0.crossfade_config.curve.id().to_string();
        config.save();
    }

    /// Compute and apply normalization gain for the current song.
    fn apply_normalization(&self) {
        const TARGET_DB: f32 = -18.0;
        let gain = match self.0.normalization_mode {
            NormalizationMode::Off => 1.0,
            NormalizationMode::Track | NormalizationMode::Album => {
                // Find the current song's replay_gain_db from the library.
                if let Some(current) = &self.0.current_song {
                    let rms_db = self.0.library.iter()
                        .find(|s| s.id == current.id)
                        .map(|s| s.replay_gain_db)
                        .unwrap_or(-18.0);
                    // gain = 10^((target - track_db) / 20)
                    10.0_f32.powf((TARGET_DB - rms_db) / 20.0).clamp(0.1, 5.0)
                } else {
                    1.0
                }
            }
        };
        if let Some(engine) = &self.0.playback {
            let _ = engine.set_norm_gain(gain);
        }
    }

    /// Sync the crossfade config to the playback engine.
    fn sync_crossfade(&self) {
        if let Some(engine) = &self.0.playback {
            let _ = engine.set_crossfade(self.0.crossfade_config);
        }
    }

    /// Update MPRIS metadata from current state.
    fn update_mpris(&self) {
        if let Some(mpris_state) = &self.0.mpris_state {
            let status = match self.0.playback_status {
                PlaybackStatus::Playing => "Playing",
                PlaybackStatus::Paused => "Paused",
                _ => "Stopped",
            };
            let meta = MprisMetadata {
                track_id: self.0.current_song.as_ref().map(|s| s.id.clone()).unwrap_or_default(),
                title: self.0.current_song.as_ref().map(|s| s.title.clone()).unwrap_or_default(),
                artist: self.0.current_song.as_ref().map(|s| s.artist.clone()).unwrap_or_default(),
                album: self.0.current_song.as_ref().map(|s| s.album.clone()).unwrap_or_default(),
                duration_us: (self.0.duration_secs * 1_000_000.0) as i64,
                art_url: String::new(),
                playback_status: status.to_string(),
                volume: self.0.volume as f64,
                position_us: (self.0.position_secs * 1_000_000.0) as i64,
            };
            mpris::update_mpris_state(mpris_state, meta);
        }
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
