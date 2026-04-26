# MUSICO — AI Context File

## IDENTITY
- Native Linux offline music player
- Rust workspace, 3 crates, Iced 0.12 GUI
- Target: i3-1115G4, <35MB RAM, <100ms click-to-sound

## WORKSPACE LAYOUT
```
Cargo.toml (workspace members: musico_app, musico_playback, musico_recommender)
musico_app/src/           # GUI binary (iced 0.12)
  main.rs                 # Entry: Musico::run(Settings{...})
  app.rs                  # impl Application for Musico — Message enum, update(), view(), subscription()
  state.rs                # AppState struct, View/LibraryViewMode/ShuffleMode/RepeatMode enums
  config.rs               # AppConfig: load/save ~/.config/musico/settings.json
  scanner.rs              # async scan_and_index(folder, recommender, progress_tx) -> Vec<SongRecord>
  theme.rs                # Celestia Shell design tokens: colors, fonts, container/button StyleSheets
  icons.rs                # 12 inline SVG &[u8] constants (PLAY, PAUSE, PREV, NEXT, SHUFFLE, REPEAT, etc.)
  components/
    mod.rs                # pub mod sidebar, seek_bar, art_canvas, song_row
    sidebar.rs            # sidebar(state) -> Element<Message> — logo, nav items, now-playing mini card
    seek_bar.rs           # Canvas-based Program<Message> with drag state, seek_bar() helper fn
    art_canvas.rs         # art_canvas(handle, size, radius, fallback_color), extract_dominant_color(img)
    song_row.rs           # song_row(song, index, is_playing, on_play, on_queue)
  views/
    mod.rs                # pub mod now_playing, library, queue, settings
    now_playing.rs        # Full player: art + glow + meta + seek + controls + shuffle/repeat indicators + recs
    library.rs            # Search + grid/list toggle + song list/grid
    queue.rs              # Upcoming queue items + recommended songs with similarity dots
    settings.rs           # Folder picker, re-index, accent color swatches, keyboard shortcuts ref
musico_playback/src/      # Audio engine library crate
  lib.rs                  # PlaybackEngine struct (cmd_tx, event_rx, state Arc<Mutex>), decoder_thread fn
  decoder.rs              # AudioDecoder: Symphonia packet-by-packet, seek_to(), metadata extraction
  resampler.rs            # AudioResampler: Rubato SincFixedIn, process_into(&[f32], &mut Vec<f32>)
  output.rs               # AudioOutput: CPAL stream, HeapRb ring buffer, AtomicBool flush_requested
  queue.rs                # PlaybackQueue: VecDeque + history Vec, next/previous/shuffle/iter/remove_at
  events.rs               # PlaybackCommand enum (Play/Pause/Resume/Stop/Seek/SetVolume/Mute/Unmute/PreloadNext)
                          # PlaybackEvent enum (Playing/Paused/Resumed/Stopped/Seeked/PositionUpdate/SongEnded/Error/...)
  state.rs                # PlaybackState, PlaybackStatus enum, SongInfo struct (id,file_path,title,artist,album,duration_secs,cover_art)
  error.rs                # PlaybackError enum (NoOutputDevice/StreamBuild/DecodeFailed/SeekOutOfRange/ResamplerError/...)
musico_recommender/src/   # Recommendation engine library crate
  lib.rs                  # MusicRecommender: index_song, on_song_changed, get_recommendations, log_listen, get_all_songs
  models.rs               # SongRecord, FeatureVector(mfcc[13],chroma[12],5 scalars), PlayEvent, SessionState, RecommendedSong
  analysis.rs             # RustFFT MFCC/chroma/spectral feature extraction
  recommender.rs          # Cosine similarity + session EMA centroid + affinity + cooldown scoring
```

## DEPENDENCY MAP
```
musico_app -> musico_playback, musico_recommender, iced 0.12 (wgpu,image,canvas,tokio,svg), walkdir, image 0.25, tokio, palette, serde/serde_json, dirs, rfd 0.14, rand 0.8, log, env_logger
musico_playback -> symphonia (all features), rubato 0.15, ringbuf 0.3, cpal 0.15, crossbeam-channel, rand 0.8, serde, thiserror, log
musico_recommender -> sled, rustfft, symphonia, bincode, serde, chrono, uuid, log
```

## THREADING MODEL
```
[GUI Thread (iced)] --PlaybackCommand--> [Decoder Thread] --f32 samples--> [Ring Buffer] --pop--> [CPAL Audio Callback]
                    <--PlaybackEvent---                                                           (AtomicU32 volume, AtomicBool muted, AtomicBool flush_requested)
[GUI Thread] --tokio::spawn_blocking--> [Recommender queries] --Message::RecommendationsUpdated--> [GUI Thread]
```

## MESSAGE ENUM (app.rs)
PlaySong(SongRecord) | Pause | Resume | Next | Previous | Seek(f32) | SetVolume(f32) | ToggleMute |
PlaybackTick | PlaybackEvent(PlaybackEvent) |
NavigateTo(View) | SearchChanged(String) | ToggleLibraryView | ScanLibrary | IndexProgress(usize,usize) | IndexComplete(Vec<SongRecord>) |
RecommendationsUpdated(Vec<RecommendedSong>) | AddToQueue(SongRecord) | RemoveFromQueue(usize) |
ToggleShuffle | ToggleRepeat |
PickFolder | MusicFolderChanged(PathBuf) | ArtColorExtracted(Color) | SetAccentColor(Color) |
WindowResized(f32,f32) | RecommenderReady(Arc<Mutex<MusicRecommender>>) | PlaybackEngineReady(Arc<PlaybackEngine>) | LoadAllSongs(Vec<SongRecord>) | KeyPressed(keyboard::Key)

## KEY ARCHITECTURAL DECISIONS
1. Recommender is DECOUPLED from playback: PlaySong sends Play to engine immediately, then fires async spawn_blocking for recommendations. No UI blocking.
2. Seek uses AtomicBool flush_requested on AudioOutput. CPAL callback checks compare_exchange(true→false) and drains ring buffer.
3. Zero-alloc hot path: process_buf/channel_buf Vec<f32> pre-allocated once, reused every packet via clear()+extend_from_slice().
4. Resampler uses process_into(&[f32], &mut Vec<f32>) instead of returning new Vec.
5. PlaybackQueue exposes iter() for UI rendering, shuffle() uses rand::SliceRandom.
6. Settings persist to ~/.config/musico/settings.json via AppConfig (music_folder, accent_color_hex, volume, library_view_mode).
7. DB path: ~/.local/share/musico/db/ (sled).
8. Folder picker: rfd::AsyncFileDialog (native OS dialog).
9. Keyboard shortcuts captured via iced::event::listen_with on KeyPressed, handled in handle_key().
10. Smart Radio: when queue empty + ShuffleMode::SmartRadio, auto-plays top recommendation. ShuffleMode::Shuffle picks random from library.

## THEME TOKENS (theme.rs)
BASE=#040409 SURFACE=#0e0f16 ELEVATED=#161721 HIGHLIGHT=#202233
BORDER_SUBTLE=#1e2033 TEXT_PRIMARY=#e2e4f0 TEXT_SECONDARY=#8b8fa8 TEXT_MUTED=#4a4d63
ACCENT_PURPLE=#9d8cff (default accent)
Fonts: FONT_DISPLAY="SF Pro Display" FONT_TEXT="SF Pro Text" FONT_ROUNDED="SF Pro Rounded"
Radii: LG=16 MD=10 SM=6 | Sidebar=208px

## STYLE PATTERN
Views define inline StyleSheet structs (e.g. PlayButtonStyle(Color), GlowStyle(Color), RowStyle{bg,hover_bg}).
theme.rs exports shared styles: NavButton{is_active}, TransportButton, SvgStyle(Color), floating_panel(), glass_card(), elevated_card().

## STATE SHAPE (state.rs → AppState)
```rust
active_view: View, window_width/height: f32,
current_song: Option<SongInfo>, playback_status: PlaybackStatus, position_secs/duration_secs: f32, volume: f32,
library: Vec<SongRecord>, filtered_library: Vec<SongRecord>, search_query: String, library_view_mode: LibraryViewMode,
queue: PlaybackQueue, recommendations: Vec<RecommendedSong>,
shuffle_mode: ShuffleMode, repeat_mode: RepeatMode,
art_dominant_color: Option<Color>, art_tint: Color,
music_folder: Option<PathBuf>, is_indexing: bool, index_progress: (usize,usize),
recommender: Option<Arc<Mutex<MusicRecommender>>>, playback: Option<Arc<PlaybackEngine>>,
```

## PLAYBACK PIPELINE (lib.rs decoder_thread)
1. wait_for_play() blocks on cmd_rx until Play(SongInfo) arrives
2. AudioDecoder::new(path) → (decoder, song_info) via Symphonia probe
3. Create AudioResampler if source_sr ≠ device_sr
4. Pre-fill ring buffer ~0.5s, emit BufferingStarted/BufferingComplete
5. Main loop: try_recv commands (Pause blocks until Resume/Stop/Play, Seek calls dec.seek_to + output.request_flush), decode_next_packet → process_samples_reuse → push_samples_blocking
6. On Ok(None): song ended → emit SongEnded{song_id, listened_secs, duration_secs}, set_stopped, return
7. Position updates emitted every 250ms

## RECOMMENDATION PIPELINE (musico_recommender)
FeatureVector: mfcc[13] + chroma[12] + spectral_centroid + spectral_rolloff + zero_crossing_rate + rms_energy + tempo_bpm
Scoring: final = w_sim*cosine_similarity + w_session*session_centroid_match + w_affinity*affinity - w_cooldown*cooldown
Session centroid: EMA of recently played feature vectors
Affinity: +delta for plays (scaled by listen_ratio), -delta for skips. Stored in SongScoreCache.
Cooldown: exponential decay based on time since last play.

## FORMATS SUPPORTED
MP3, FLAC, OGG, WAV, M4A, AAC, OPUS, ALAC, WMA, MP4, M4B (symphonia all features)

## BUILD/TEST
```
cargo build --release          # ~2m43s on i3-1115G4
cargo test --workspace         # 8/8 pass (6 unit + 2 doctests)
cargo run                      # dev mode
```
Zero compiler warnings. Only external warning: ashpd v0.8.1 future-incompat (rfd dependency, not our code).

## FILE SIZES (lines)
app.rs:~550 | state.rs:~120 | config.rs:~100 | theme.rs:~200 | scanner.rs:~50
lib.rs(playback):~750 | decoder.rs:~300 | resampler.rs:~90 | output.rs:~170 | queue.rs:~110
now_playing.rs:~330 | library.rs:~260 | queue.rs(view):~160 | settings.rs:~220
sidebar.rs:~220 | seek_bar.rs:~180 | art_canvas.rs:~97 | song_row.rs:~137
lib.rs(recommender):~215 | models.rs:~220

## KNOWN LIMITATIONS
- No gapless crossfade yet (PreloadNext is a no-op hint)
- Library not virtualized (renders all rows; fine up to ~5000 songs)
- Album art in library grid is placeholder rectangles (cover art only shown in Now Playing)
- No playlist management (only queue + smart radio)
- SF Pro fonts must be installed on the system (falls back to default otherwise)

[don't waste tokens on writing long walkthrough and all , try to save tokens while not compromising with quality of code and try to provide  the most compact and efficient response ]