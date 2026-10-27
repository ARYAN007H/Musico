<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/Offline_First-9d8cff?style=for-the-badge" />
</p>

<h1 align="center">🎵 Musico</h1>

<p align="center">
  <strong>A fast, offline-first music player for Linux that learns your taste.</strong><br>
  <em>Tauri shell · pure-Rust audio engine · on-device recommendations · zero cloud.</em>
</p>

<p align="center">
  <code>513ms cold start</code> · <code>~138 MB RAM</code> · <code>Smart Radio</code> · <code>Synced lyrics</code> · <code>MPRIS</code>
</p>

<p align="center">
  🌐 <a href="https://aryan007h.github.io/Musico/"><b>landing site</b></a> · <a href="https://github.com/ARYAN007H/musico/releases/latest"><b>download v0.3.0</b></a>
</p>

---

## ✨ What is Musico?

Musico plays your local library with instant response times, learns what you like through an
**offline recommendation engine** (no cloud, no tracking), and wraps it all in a **proper desktop
shell** — sidebar navigation, a now-playing dock, a real list-based library — finished in a soft
neumorphic style with light/dark themes and six color palettes.

No accounts. No telemetry. Your music never leaves your machine.

## 🚀 Features

- **Real engine, real controls** — Symphonia decode → Rubato resample → lock-free ring buffer →
  CPAL output. Progress, seek, and track changes are driven by actual engine events, not timers.
- **Smart Radio** — when the queue runs dry, Musico picks the next track from on-device audio
  analysis: MFCC/chroma feature vectors, cosine similarity, session mood-lock, play/skip affinity,
  and cooldown. Every skip teaches it.
- **Incremental library scan** — tags, durations, and cover art extracted per-file and cached;
  restarting the app re-scans in **milliseconds** and only re-analyzes changed files.
- **A library that behaves like one** — instant search, multi-key sort, list & grid views,
  hover-quick "add to queue", and liked songs that persist behind a ♥ Liked filter.
- **Synced lyrics** — drop a `.lrc` next to an audio file and Now Playing becomes karaoke.
- **MPRIS integration** — media keys, `playerctl`, desktop widgets, and Bluetooth controls work
  out of the box (`org.mpris.MediaPlayer2.musico`).
- **10-band EQ + ReplayGain normalization** — presets (Bass Boost, Vocal, Rock, Classical…) or
  your own curve; loudness matched across tracks.
- **Listening stats** — plays, hours, skip rate, streaks, and top songs/artists.
- **Smart shuffle & repeat** — shuffle / Smart Radio / repeat-one / repeat-all with proper
  queue-first semantics.
- **Light & dark themes, 6 palettes** — Nebula · Sakura · Aurora · Ocean · Ember · Mono.
  Keyboard-accessible throughout (visible focus, full ARIA labels, reduced-motion aware).

### Keyboard Shortcuts

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `← / →` | Seek ±5 seconds |
| `↑ / ↓` | Volume |
| `N` / `P` | Next / previous track |
| `S` | Cycle shuffle → Smart Radio |
| `R` | Cycle repeat |

### Format Support

MP3 · FLAC · OGG · WAV · M4A · AAC · OPUS · ALAC · WMA · MP4 · M4B
via [Symphonia](https://github.com/pdeljanov/Symphonia).

## 📊 Measured (v0.3.0, release build)

| Metric | Value |
|---|---|
| Cold start to ready | **513 ms** |
| RAM (idle) | **~138 MB** |
| CPU (idle) | ~1% |
| Library scan (289 files, warm) | < 1 s |
| First index (289 files) | ~5 min, background, non-blocking |

Measured on Intel i3-1115G4 (2C/4T), Hyprland/Wayland. Playback figures pending on-device
profiling; the idle footprint is the background-app baseline that matters.

## 📦 Install

```bash
curl -LO https://github.com/ARYAN007H/Musico/releases/latest/download/musico-linux-x86_64
chmod +x musico-linux-x86_64
./musico-linux-x86_64
```

Or move it into your `PATH` as `musico`.

## 🛠️ Build from Source

**Prerequisites (Linux):** Rust 1.78+, WebKitGTK 4.1, ALSA, and standard build tooling.

```bash
# Arch
sudo pacman -S rust alsa-lib webkit2gtk-4.1
# Ubuntu / Debian
sudo apt install rustc cargo libasound2-dev libwebkit2gtk-4.1-dev libgtk-3-dev pkg-config
# Fedora
sudo dnf install rust cargo alsa-lib-devel webkit2gtk4.1-devel
```

```bash
git clone https://github.com/ARYAN007H/Musico.git
cd Musico
cargo run -p musico-tauri        # dev
cargo build --release -p musico-tauri
./target/release/musico-tauri
```

```bash
cargo test --workspace           # unit + engine e2e (audio device needed for e2e)
```

## 🏗️ Architecture

```
┌────────────────────────────────────────────────────────────┐
│ frontend/            desktop shell — vanilla JS/CSS        │
│   sidebar nav + now-playing dock, light/dark, 6        │
│   palettes; js/app.js views & state;                   │
│   js/bridge.js Tauri IPC + stateful browser mock       │
├────────────────────────────────────────────────────────────┤
│ src-tauri/           Tauri 2 shell                         │
│   event_bridge.rs    engine events → webview (50 ms poll)  │
│   scanner.rs         incremental scan + cover cache        │
│   mpris.rs           D-Bus MPRIS2 server                   │
│   config.rs          settings persistence                  │
│   lyrics.rs          LRC parser                            │
├───────────────────────┬────────────────────────────────────┤
│ musico_playback       │ musico_recommender                 │
│ Symphonia → Rubato →  │ RustFFT MFCC/chroma features       │
│ ringbuf → CPAL        │ sled persistence, cosine + session │
│ EQ · ReplayGain       │ scoring, play/skip affinity        │
└───────────────────────┴────────────────────────────────────┘
```

Song ids are a stable hash of the file path — shared between the library, the queue, and the
recommender — so listening history survives re-scans and re-indexing.

## ⚙️ Configuration

Settings: `~/.config/musico/settings.json` (written automatically).
Cover art cache: `~/.cache/musico/covers/` (256 px JPEGs — RAM stays flat as libraries grow).
Recommendation DB: `~/.local/share/musico/db/` (sled). Logs: `~/.local/share/musico/musico.log`.

## 📜 License

MIT.

---

<p align="center">
  <strong>Musico</strong> — because a music player should be a background citizen, not a browser.<br>
  <em>Made with 🦀 and ❤️ on Linux.</em>
</p>
