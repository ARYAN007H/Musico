<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" />
  <img src="https://img.shields.io/badge/Offline-First-9d8cff?style=for-the-badge" />
</p>

<h1 align="center">🎵 Musico</h1>

<p align="center">
  <strong>A blazing-fast, offline-first music player for Linux</strong><br>
  <em>Built with pure Rust — zero Electron, zero web views, zero compromises.</em>
</p>

<p align="center">
  <code>Instant playback</code> · <code>Smart Radio</code> · <code>Ambient UI</code> · <code>Auto-Update</code> · <code>< 30 MB RAM</code>
</p>

---

## ✨ What is Musico?

Musico is a **native Linux desktop music player** designed for people who care about performance and aesthetics equally. It plays your local music library with instant response times, learns your taste through an offline recommendation engine, and wraps it all in a gorgeous dark-mode interface inspired by the Celestia Shell.

No accounts. No cloud. No tracking. Just you and your music.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    musico_app (GUI)                      │
│  Iced 0.12 · Celestia Shell theme · SVG icons           │
│  Keyboard shortcuts · Settings persistence              │
├────────────────────┬────────────────────────────────────┤
│  musico_playback   │       musico_recommender           │
│  ─────────────────  │       ──────────────────           │
│  Symphonia decoder │       RustFFT feature extraction   │
│  Rubato resampler  │       Sled embedded database       │
│  CPAL output       │       Cosine similarity search     │
│  Lock-free ringbuf │       Session-aware scoring        │
└────────────────────┴────────────────────────────────────┘
```

Three independent crates, zero C dependencies (aside from ALSA), one workspace.

| Crate | Role |
|---|---|
| **`musico_app`** | Iced GUI, navigation, views, theming, keyboard shortcuts |
| **`musico_playback`** | Decode → resample → ring buffer → CPAL output pipeline |
| **`musico_recommender`** | Audio feature extraction, listening history, recommendations |

---

## 🚀 Features

### Performance
- **Instant playback** — click a song, hear it immediately. The recommender runs async on a background thread — never blocks the audio pipeline.
- **Instant seek** — `AtomicBool` flush signal tells the CPAL callback to discard stale samples on the very next invocation. No more hearing old audio after scrubbing.
- **Zero-allocation hot path** — pre-allocated `Vec` buffers are reused across every decode → resample → push cycle. No heap allocations per packet.
- **Lock-free audio** — the decoder thread pushes samples through a `ringbuf` ring buffer. The CPAL callback pops them. No mutexes on the audio thread.

### Smart Music
- **Smart Radio** — when the queue is empty, Musico auto-fills the next track from its recommendation engine, using cosine similarity on MFCC/chroma feature vectors + session mood-lock + listening history affinity.
- **Three shuffle modes** — Off → Shuffle → Smart Radio, cycled with the `S` key.
- **Repeat modes** — Off → All → One, cycled with the `R` key.
- **Skip detection** — songs listened to < 30% are marked as skips, reducing their future recommendation score.

### UI / UX
- **Ambient Glow** — album art's dominant color is extracted and used to tint the Now Playing background and the play button's glow shadow.
- **6 Color Palettes** — Nebula (purple) · Sakura (pink) · Aurora (green) · Ocean (blue) · Ember (orange) · Mono (grayscale). Switch instantly in Settings.
- **4 Font Personalities** — Classic (SF Pro) · Playful (Comfortaa) · Techno (JetBrains Mono) · Cozy (Nunito). Each adjusts border radii for a cohesive feel.
- **Responsive layout** — compact icon-rail sidebar at <700px, standard at 700-1100px, wide with queue side-panel at >1100px.
- **Auto-update** — check for updates from Settings, downloads and installs automatically from GitHub Releases. Your data is never touched.
- **Native folder picker** — `rfd` opens your OS file dialog to select your music folder. No more editing config files.
- **Settings persistence** — palette, font mode, volume, and view mode are saved to `~/.config/musico/settings.json`.

### Keyboard Shortcuts

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `← / →` | Seek ±5 seconds |
| `↑ / ↓` | Volume ±5% |
| `N` | Next track |
| `P` | Previous track |
| `S` | Cycle shuffle mode |
| `R` | Cycle repeat mode |
| `Esc` | Clear search / go to Now Playing |

### Format Support

MP3 · FLAC · OGG · WAV · M4A · AAC · OPUS · ALAC · WMA · MP4 · M4B

Powered by [Symphonia](https://github.com/pdeljanov/Symphonia) with the `all` features flag — every format Symphonia supports, Musico supports.

---

## 📦 Install

### Pre-built Binary (recommended)

Download the latest release from [GitHub Releases](https://github.com/ARYAN007H/Musico/releases):

```bash
# Download
curl -LO https://github.com/ARYAN007H/Musico/releases/latest/download/musico-linux-x86_64

# Make executable
chmod +x musico-linux-x86_64

# Run
./musico-linux-x86_64
```

Or move it to your PATH:
```bash
sudo mv musico-linux-x86_64 /usr/local/bin/musico
musico
```

> **Auto-update**: Once running, go to Settings → Check for Updates to update in-place.

### Build from Source

#### Prerequisites

```bash
# Arch / Manjaro
sudo pacman -S rust alsa-lib

# Ubuntu / Debian
sudo apt install rustc cargo libasound2-dev pkg-config libfontconfig1-dev

# Fedora
sudo dnf install rust cargo alsa-lib-devel
```

#### Build & Run

```bash
git clone https://github.com/ARYAN007H/Musico.git
cd Musico

# Debug (fast compile, slower runtime)
cargo run

# Release (slow compile, blazing runtime)
cargo build --release
./target/release/musico_app
```

### Run Tests

```bash
cargo test --workspace
```

---

## 📂 Project Structure

```
Musico/
├── musico_app/                 # GUI application
│   └── src/
│       ├── app.rs              # Iced Application — message handling, commands
│       ├── state.rs            # Global app state, enums
│       ├── config.rs           # Settings persistence (~/.config/musico/)
│       ├── scanner.rs          # Recursive folder scanner + indexer
│       ├── theme.rs            # Celestia Shell design tokens & styles
│       ├── icons.rs            # Inline SVG icon constants
│       ├── components/
│       │   ├── sidebar.rs      # Navigation sidebar with logo + mini player
│       │   ├── seek_bar.rs     # Canvas-based seek bar with drag support
│       │   ├── art_canvas.rs   # Album art widget + dominant color extraction
│       │   └── song_row.rs     # Reusable song list item
│       └── views/
│           ├── now_playing.rs  # Full-screen player with ambient glow
│           ├── library.rs      # Grid/list library browser with search
│           ├── queue.rs        # Upcoming queue + recommendations
│           └── settings.rs     # Folder picker, accent colors, shortcuts
│
├── musico_playback/            # Audio engine
│   └── src/
│       ├── lib.rs              # PlaybackEngine + decoder thread loop
│       ├── decoder.rs          # Symphonia packet-by-packet decoder
│       ├── resampler.rs        # Rubato SincFixedIn resampler
│       ├── output.rs           # CPAL stream + ring buffer + flush
│       ├── queue.rs            # PlaybackQueue with shuffle support
│       ├── events.rs           # Command/Event enums for IPC
│       ├── state.rs            # PlaybackState, SongInfo, PlaybackStatus
│       └── error.rs            # PlaybackError enum
│
└── musico_recommender/         # Recommendation engine
    └── src/
        ├── lib.rs              # MusicRecommender API
        ├── models.rs           # SongRecord, FeatureVector, PlayEvent
        ├── analysis.rs         # RustFFT-based MFCC/chroma extraction
        └── recommender.rs      # Cosine similarity + session scoring
```

---

## 🎨 Design System

Musico uses the **Celestia Shell** design language:

| Token | Value | Usage |
|---|---|---|
| `BASE` | `#040409` | Window background |
| `SURFACE` | `#0e0f16` | Panels, cards |
| `ELEVATED` | `#161721` | Hover states, inputs |
| `ACCENT_NEBULA` | `#9d8cff` | Default palette accent |
| `TEXT_PRIMARY` | `#e2e4f0` | Headings, titles |
| `TEXT_MUTED` | `#4a4d63` | Captions, placeholders |

Six color palettes: **Nebula** · **Sakura** · **Aurora** · **Ocean** · **Ember** · **Mono**
Four font modes: **Classic** · **Playful** · **Techno** · **Cozy**

---

## 🔧 Configuration

Settings are stored at `~/.config/musico/settings.json`:

```json
{
  "music_folder": "/home/user/Music",
  "palette_id": "nebula",
  "font_mode": "classic",
  "volume": 0.85,
  "library_view_mode": "grid"
}
```

The recommendation database lives at `~/.local/share/musico/db/` (Sled).

---

## 🧠 How Recommendations Work

1. **Index** — when you scan a folder, each song is decoded and a 30-dimensional feature vector is extracted (13 MFCCs + 12 chroma bins + 5 scalar features).
2. **Listen** — every play event records duration, skip status, and timestamp.
3. **Score** — when you play a song, Musico finds candidates by:
   - **Cosine similarity** between the current song's feature vector and all others
   - **Session mood-lock** — an EMA centroid of recently played songs biases toward the current vibe
   - **Affinity** — songs you've replayed get boosted, songs you've skipped get penalized
   - **Cooldown** — recently played songs are suppressed to avoid repetition
4. **Rank** — the final score is a weighted blend of all four signals, sorted descending.

All computation is local. No data leaves your machine.

---

## 📊 Resource Usage

Tested on **Intel i3-1115G4** (2C/4T, 3.0 GHz):

| Metric | Value |
|---|---|
| Cold start | ~1.2s |
| Click-to-sound | < 100ms |
| Seek latency | < 50ms |
| RAM (idle) | ~25 MB |
| RAM (playing) | ~35 MB |
| CPU (playing) | < 3% |

---

## 📜 License

This project is licensed under the MIT License.

---

<p align="center">
  <strong>Musico</strong> — because music players shouldn't need 500 MB of RAM.<br>
  <em>Made with 🦀 and ❤️ on Linux.</em>
</p>
