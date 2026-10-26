# MUSICO — AI Context

> **MAINTENANCE RULE:** Update this file whenever you make structural changes (new files, new
> commands, changed APIs, completed plan items). Keep it dense — no walkthroughs, no code dumps.
> This is the single source of truth; `context.md` is a stub pointing here.

## IDENTITY
- Native Linux offline music player. No accounts, no cloud, no tracking.
- **Stack:** Rust workspace + Tauri 2 shell + vanilla JS/CSS frontend (no framework, no bundler).
- Perf targets: <35MB RAM, <100ms click-to-sound. Test machine: i3-1115G4.
- Design language: "Celestia Shell" → dark neumorphic UI. Base `#040409`, accent default Nebula `#9d8cff`.

## HISTORY / DECISIONS
- v0.1–0.2 was a **Iced 0.12 GUI** (`musico_app`) — mature: real scanner, config persistence, MPRIS,
  synced lyrics, EQ UI, sleep timer, stats, playlists, auto-update. Tests 8/8 pass.
- Current effort (commit `66c56b7`): **rewrite UI as Tauri + web frontend**.
- Frontend is plain HTML/CSS/JS in `frontend/`, loaded directly (`frontendDist: "../frontend"`) —
  no dev server, no npm. Keep it vanilla (no React/bundler) unless a view genuinely demands it.

## VISION (confirmed with Aryan, 2026-10-27)
- Tauri shell REPLACES Iced. `musico_app` is a port source ONLY (scanner.rs, config.rs, covers.rs,
  lyrics.rs, mpris.rs, timer.rs, updater.rs) — delete at parity. Do not invest in the Iced app.
- Audience: Aryan's daily driver first, then a public Linux release — polish/packaging still matter.
- Core identity features: **Smart Radio/recs, synced lyrics, playlists + stats, EQ + normalization.**
  Sleep timer and auto-update are NOT core (M3/cut).
- UI: responsive desktop — compact ~520px → mid → wide with queue side-panel >1100px.
- Standing constraint (revised 2026-10-27): RAM soft cap ~200MB, 400MB+ = bug. A music app is a
  background process — every feature must respect this. Measure at C1 and C14.
- No story/branding talk — the goal is a daily-driver app.

## RELEASE STATE
**v0.3.0 committed (26d3c74, 2026-10-27). All 14 chunks done. Measured: 513ms cold start,
~138MB RAM idle, ~1% idle CPU (release build). Tests: 20 across workspace, all green.
Next-up candidates (not started): playback-mode profiling, catch_unwind guard on decoder
thread, playlist CRUD editor, tauri-plugin-updater, release tag + GitHub publish.**

## CURRENT STATUS (2026-10-27, post C6)
**DONE & VERIFIED:**
- C1 Real playback: `src-tauri/src/event_bridge.rs` drains engine events → `playback-event`
  Tauri events (tagged JSON, 50ms poll); auto-advance; next/previous/play_next/clear_queue;
  queue-based play_song; frontend consumes events (fake simulation deleted); queue view live.
  Verified: engine e2e tests (real audio device) + 17 browser assertions.
- C2 Real scan: `src-tauri/src/scanner.rs` — probe-only metadata (tags/duration/cover via
  symphonia), stable path-hash ids, disk cover cache ~/.cache/musico/covers (256px jpg),
  async scan thread → scan-progress/library-updated events, get_cover command → base64 data
  URL, art in vinyl+mini player. Real run: 289/289 songs, 0 skipped.
- C3 Persistence: `src-tauri/src/config.rs` AppSettings → ~/.config/musico/settings.json (all
  serde defaults); volume applied to engine at boot; scan CACHE (~/.cache/musico/library.json,
  mtime+size keyed) → startup scan is instant (289 songs, 0 rescanned); auto-scan at launch
  when folder configured; rfd folder picker (pick_folder/set_music_folder commands); frontend
  applies + persists palette/toggles/pills/slider/view-mode.
- C4 Semantics: advance policy in event_bridge (queue → repeat-one → shuffle/random → stop);
  set_playback_modes command persists shuffle/repeat; keyboard S/R/N/P/Space/arrows wired;
  mini-player click navigates; playing-row highlight fixed (dataset ids).
- C5 Recommender: `MusicRecommender` in AppState (Arc<Mutex>, sled at ~/.local/share/musico/db);
  background indexer thread after scan (index_song self-dedups by path; index-progress/
  index-complete events); real get_recommendations (cosine + session; fallback spread when
  empty). **IDs unified**: musico_recommender/vector_store.rs `stable_song_id(path)` = path
  hash — same as SongMeta.id, so plays/skips log against the right song. One-time migration
  clears UUID-era DB (marker file .stable-ids).
- C6 Smart Radio: advance() logs listens (skip if ratio<0.3) on SongEnded, calls
  on_song_changed (session EMA) on every play, and when shuffle_mode==2 + smart_recs →
  plays top recommendation instead of random.

**TESTING HOW-TO (all green):**
- `cargo test --workspace` — engine e2e (real audio device, /tmp/musico-test-tone.wav via
  python sine gen), scanner metadata test, recommender unit tests.
- Browser logic tests: `python3 -m http.server 8137 -d frontend` → drive `App.*` / dispatch
  `App.handlePlaybackEvent(...)` via Playwright — Bridge mocks are stateful (queue works).
- `scripts/webdriver_e2e.py` — REAL-app WebDriver e2e via tauri-driver; needs WebKitWebDriver
  binary (NOT in Arch webkit2gtk-4.1; Ubuntu/CI has webkit2gtk-driver). Runs in CI.
- Real-run verification: RUST_LOG=info ./target/debug/musico-tauri → log lines; UI screenshot
  via `grim` (Computer Use MCP unavailable in this session).
- pkill gotcha: `pkill -f musico-tauri` kills the calling shell too (cmdline match) — use `pkill -x`.

**DONE & VERIFIED (C7–C9 added 2026-10-27 late):**
- C7 Stats/playlists: `get_stats`/`get_playlists` commands; Library Insights view (plays/hours/
  skips/completion/streak/top songs/top artists); top songs resolve titles via library and are
  clickable. Playlist CRUD editor deferred.
- C8 Lyrics: `src-tauri/src/lyrics.rs` (LRC parser ported), sidecar `.lrc` next to audio file,
  get_lyrics command; Now Playing karaoke panel with .active/.past highlight on PositionUpdate.
- C9 EQ/norm: set_eq (10-band) + set_normalization commands; EQ vertical sliders + 6 presets
  in Settings → Audio; ReplayGain: on every track start, if normalization=="Track", gain =
  10^((−18 − replay_gain_db)/20) applied via set_norm_gain (event_bridge play_and_track).

**DONE & VERIFIED (C10–C12 added 2026-10-27 night):**
- C10: window default 1100x720 (min 460x640); wide-layout media query ≥1100px in layout.css
  (content max-widths + queue side panel via `#view-now-playing.active ~ #view-queue:not(.active)`).
- C11 MPRIS: `src-tauri/src/mpris.rs` (zbus 4 + tokio port of the Iced version) —
  `org.mpris.MediaPlayer2.musico` VERIFIED LIVE on session bus (busctl). command_loop maps
  D-Bus → engine (advance policy for next); event_bridge update_mpris() mirrors state every
  event; art_url = file:// cover cache. Initial "Stopped" pushed at startup.
- C12: Bridge.invoke catches Tauri rejections → toast (App.showError, #musico-toast) + engine
  Error events → toast; logs append to ~/.local/share/musico/musico.log (env_logger Pipe;
  RUST_LOG=stderr for dev); get_recommendations uses try_lock → fallback spread when indexer
  holds the lock. **Indexer rewritten: analysis (analyze_file) runs WITHOUT the lock; only
  the DB write locks** — fixes the startup-hang bug (analysis held Mutex ~5s/file and
  wedged get_recommendations at boot). Full library indexed: 289/289 songs.
- C13 partial: release.yml updated for musico-tauri (webkit2gtk deps, cargo test without
  audio-device tests); version 0.3.0 everywhere.

**DARK MODE + INSTALL (2026-10-27 late):**
- `[data-theme="dark"]` token block in tokens.css (Celestia dark surfaces); Light/Dark pills in
  Settings → Themes; persisted as `theme` in AppSettings (serde default "light").
- **Tauri v2 global API: invoke is `window.__TAURI__.core.invoke`, NOT `.invoke`** — bridge.js
  handles both shapes. Event listen is `__TAURI__.event.listen` (unchanged).
- Installed to device: `~/.local/bin/musico` (release binary), desktop entry +
  hicolor SVG icon installed. Old Iced binary + entry REMOVED. Shared sled DB kept.
  Browser previews cache JS aggressively — cache-bust with ?v= when testing CSS/JS edits.

**KNOWN LIMITATIONS / NEXT:**
- Engine-thread panic = silent death (no catch_unwind in musico_playback decoder loop) —
  acceptable for now; the event bridge would keep polling a dead engine.
- App window tested at ~980px tiles; ≥1100px media query unverified visually (no window
  resize without input automation) — standard CSS, low risk.
- Release-mode RAM measurement pending (C14).
- Testing hygiene: only ONE app instance at a time (sled DB lock + audio device); UI logic
  via `python3 -m http.server 8137 -d frontend` + Playwright; `pkill -x musico-tauri` (NOT
  -f, which kills the calling shell); never run `scripts/webdriver_e2e.py` while an instance
  is wanted — it pkills first.

## WORKSPACE LAYOUT
```
Cargo.toml                  workspace: musico_app, musico_playback, musico_recommender, src-tauri
src-tauri/                  Tauri 2 shell (the active app)
  tauri.conf.json           window 520x820 min 380x640, frontendDist ../frontend
  src/main.rs               builder, plugin(shell), manage(AppState), 16 registered commands
  src/state.rs              AppState{engine,library,music_folder,queue,settings} all Mutex;
                            SongMeta{id,title,artist,album,duration_secs,file_path} (no cover);
                            AppSettings{palette,art_tint,smart_recs,eq_enabled,eq_preset,
                            normalization,crossfade_enabled,crossfade_duration}
  src/commands.rs           get_library, scan_library, search_library, play_song, pause, resume,
                            seek, set_volume, toggle_mute, get_playback_state, get_queue,
                            add_to_queue, remove_from_queue, get_settings, save_settings,
                            get_recommendations(stub)
frontend/
  index.html                496ln; views: view-now-playing/-library/-queue/-playlists/-settings;
                            mini-player w/ --mini-progress CSS var; #btn-scan unhandled
  js/bridge.js              Bridge.invoke(cmd,args) → window.__TAURI__.invoke, mock fallback
                            (DEMO_SONGS) for standalone browser preview; Bridge.listen()
  js/app.js                 568ln App module: state obj, init(), navigateTo(), filterLibrary(),
                            sortSongs, transport handlers, fake progressSimulation, renderers,
                            generateWaveform(seed) fake
  js/knob.js                Knob.setVolume(v) — rotary volume
  js/vinyl.js               Vinyl.setPlaying(bool), Vinyl.setAlbumArt(...) — spinning vinyl
  js/seek.js                Seek — canvas seek bar (returns {})
  css/tokens.css            palette via [data-palette] attr on <html>; fonts, radii
  css/neumorphic|layout|now-playing|library|queue|settings|animations.css
musico_app/src/             (PORT SOURCE, Iced) scanner.rs real metadata+cover extraction,
                            config.rs ~/.config/musico/settings.json persistence, covers.rs,
                            lyrics.rs LRC parser, mpris.rs, timer.rs, updater.rs, stats
musico_playback/src/        lib.rs PlaybackEngine; decoder.rs symphonia; resampler.rs rubato;
                            output.rs cpal+ringbuf+flush; queue.rs PlaybackQueue; events.rs; state.rs
musico_recommender/src/     lib.rs MusicRecommender; models.rs; analysis.rs RustFFT MFCC/chroma;
                            recommender.rs cosine+session EMA+affinity+cooldown
```

## BACKEND API CHEATSHEET

**PlaybackEngine** (`musico_playback::PlaybackEngine`):
- `new() -> Result<Self>` | `play(SongInfo) pause() resume() stop() seek(f32) set_volume(f32)
  toggle_mute() -> Result<(), PlaybackError>`
- `poll_events() -> Vec<PlaybackEvent>` — MUST be drained by Tauri side (currently never called)
- `state() -> PlaybackState{status, position_secs, duration_secs, volume, muted, current_song}`
- `set_eq(enabled, [f32;10])`, `set_norm_gain(f32)`, `set_crossfade(CrossfadeConfig)` — exist, unwired
- `preload_next(SongInfo)` — gapless hint, no-op-ish
- Threading: GUI → cmd_tx → decoder thread → ring buffer → CPAL callback (lock-free, AtomicU32 vol)

**PlaybackEvent** variants: `Playing(SongInfo)`, `Paused{position_secs}`, `Resumed`, `Stopped`,
`Seeked{position_secs}`, `PositionUpdate{position_secs}` (~250ms), `SongEnded{song_id,
listened_secs, duration_secs}`, `BufferingStarted/Complete`, `Error{...}` (check events.rs exact shapes)

**PlaybackQueue** (`musico_playback::queue`): `push_back/push_front(SongInfo)`, `next() previous()
-> Option<SongInfo>`, `remove_at(usize)`, `shuffle()`, `peek_next()`, `iter()`, `current_queue()`

**MusicRecommender** (`musico_recommender`): `new(db_path:&str)` (sled at
~/.local/share/musico/db/), `index_song(file_path)`, `index_songs_batch(paths, cb?)`,
`get_recommendations(RecommendationRequest) -> Vec<RecommendedSong>`, `log_listen(...)`,
`on_song_changed(id)`, `get_all_songs() -> Vec<SongRecord>`, `get_stats() -> ListeningStats`,
`get_playlists/save_playlist/delete_playlist`. NOT thread-safe wrapper — use `Arc<Mutex<>>`.

**Data shapes:**
- `SongRecord{id, file_path, title, artist, album, duration_secs:u32, feature_vector:FeatureVector,
  indexed_at, replay_gain_db}` — FeatureVector = mfcc[13]+chroma[12]+5 scalars (centroid, rolloff,
  zcr, rms, tempo)
- `RecommendedSong{record, final_score, similarity_score, session_match, affinity, cooldown}`
- `SongInfo{id, file_path, title, artist, album, duration_secs:f32, cover_art:Option<Vec<u8>>}`
- `SongMeta` (Tauri DTO) = SongInfo minus cover_art; convert via `From<SongMeta> for SongInfo`

## FRONTEND CONVENTIONS
- Modules are IIFEs exposing globals: `App`, `Bridge`, `Knob`, `Vinyl`, `Seek`. Script order in
  index.html matters (bridge → knob/vinyl/seek → app).
- Bridge pattern: every Tauri command should have a mock fallback in bridge.js `mockCommand()` so
  index.html can be previewed standalone in a browser.
- CSS: tokens in tokens.css (`--accent` etc.), palettes via `html[data-palette="nebula|sakura|
  aurora|ocean|ember|mono"]`. Neumorphic = soft dual shadows, no hard borders.
- State lives in `App.state` (module-private object). Re-render = innerHTML rebuild of containers.
- Music folder default: `$HOME/Music` (`dirs_or_default()` in state.rs).

## BUILD PLAN — CHUNKS (test each chunk before moving on)
**M1 — Daily-driver core:**
- C1 Real playback wiring: event_bridge.rs drains engine.poll_events() → `playback-event` Tauri
  events; auto-advance on SongEnded; next/previous/play_next/clear_queue commands; queue-based
  play_song (routes through PlaybackQueue so previous() works); frontend consumes events, fake
  progress simulation DELETED; queue view renders real queue; withGlobalTauri enabled.
  TEST: real progress, next/prev, auto-advance, RAM baseline measured.
- C2 Real scan: port musico_app/scanner.rs (symphonia tags/duration/cover art), async + progress
  events, wire #btn-scan, cover_art → base64 in SongMeta, art in vinyl/song rows/mini player.
- C3 Persistence: port config.rs → src-tauri (load in AppState::new, save on save_settings);
  wire toggles/swatches/knob; rfd folder picker; auto-load library on start.
- C4 Controls pass: seek drag wired, shuffle/repeat semantics real (S/R keys), keyboard shortcuts
  verified, mini-player progress line, playing-row highlight.
**M2 — Identity:**
- C5 Recommender indexing: Arc<Mutex<MusicRecommender>> in AppState; background index_songs_batch
  on scan with progress events; real get_recommendations.
- C6 Smart Radio: log_listen on SongEnded (<0.3 ratio = skip), on_song_changed, auto-fill queue
  from recommendations when empty + smart mode.
- C7 Playlists + stats views (recommender APIs exist).
- C8 Synced lyrics: port lyrics.rs (LRC sidecar), karaoke display.
- C9 EQ + normalization UI → set_eq/set_norm_gain; ReplayGain via SongRecord.replay_gain_db.
**M3 — Production:**
- C10 Responsive wide layout (>1100px queue side-panel), window defaults/state persist.
- C11 MPRIS (port mpris.rs), app icon/desktop entry, single-instance.
- C12 Robustness: error toasts, corrupt-file skip in scan, file logging.
- C13 Release: tauri bundling (AppImage/deb), tauri-plugin-updater, CI, README refresh, v0.3.0.
- C14 Perf audit: RAM/CPU/startup measured vs 200MB budget; fix regressions; numbers in README.

## CONVENTIONS (Rust)
- Tauri state access: `state.field.lock().unwrap_or_else(|e| e.into_inner())` (poison-tolerant).
- Commands return `Result<_, String>` via `.map_err(|e| e.to_string())` for fallible ops.
- Don't block IPC commands with heavy work — spawn + emit events instead.
- Zero-alloc hot path in playback (pre-allocated buffers) — don't add allocations there.
- Edition 2021, rust-version 1.78. Tauri v2 API (`tauri::State`, `AppHandle`, `Emitter` trait for emit).

## BUILD/TEST
```
cargo build --release          # ~2m43s on i3-1115G4
cargo test --workspace         # 8/8 pass (Iced-era tests)
cargo tauri dev                # Tauri dev (no beforeDevCommand needed — static frontend)
cargo tauri build              # bundle
# standalone UI preview: open frontend/index.html in a browser (Bridge mock data kicks in)
```
Known external warning: ashpd v0.8.1 future-incompat (via rfd, not our code).
