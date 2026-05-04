//! Shared data structures used across the recommendation engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Internal database handle
// ---------------------------------------------------------------------------

/// Wraps the sled database and its named trees for data isolation.
/// Each data domain gets its own B-tree so scans never cross domains.
#[derive(Clone)]
pub(crate) struct Store {
    /// Raw sled handle (default tree holds metadata counters).
    pub db: sled::Db,
    /// `{song_id}` → bincode `SongRecord`
    pub songs: sled::Tree,
    /// `{song_id}:{event_id}` → bincode `PlayEvent`
    pub events: sled::Tree,
    /// `current` → bincode `SessionState`
    pub sessions: sled::Tree,
    /// `{song_id}` → bincode `SongScoreCache`
    pub scores: sled::Tree,
}

impl Store {
    pub fn open(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let songs = db.open_tree("songs")?;
        let events = db.open_tree("events")?;
        let sessions = db.open_tree("sessions")?;
        let scores = db.open_tree("scores")?;
        Ok(Self { db, songs, events, sessions, scores })
    }

    /// O(1) song count via a counter in the default tree.
    pub fn song_count(&self) -> Result<usize, sled::Error> {
        match self.db.get(b"meta:song_count")? {
            Some(bytes) => {
                let arr: [u8; 8] = bytes.as_ref().try_into().unwrap_or([0u8; 8]);
                Ok(u64::from_le_bytes(arr) as usize)
            }
            None => Ok(0),
        }
    }

    /// Atomically increment the song counter.
    pub fn increment_song_count(&self) -> Result<(), sled::Error> {
        let count = (self.song_count()? + 1) as u64;
        self.db.insert(b"meta:song_count", &count.to_le_bytes())?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Audio feature vector
// ---------------------------------------------------------------------------

/// A normalised (all values in `[0.0, 1.0]`) feature vector extracted from an
/// audio file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureVector {
    /// Schema version — bump when extraction logic changes.
    pub version: u8,
    /// Mel-Frequency Cepstral Coefficients 1-13.
    pub mfcc: [f32; 13],
    /// Weighted mean of frequencies — brightness indicator.
    pub spectral_centroid: f32,
    /// Frequency below which 85 % of spectral energy is concentrated.
    pub spectral_rolloff: f32,
    /// Rate of sign-changes — noisiness indicator.
    pub zero_crossing_rate: f32,
    /// Root-mean-square energy — perceived loudness.
    pub rms_energy: f32,
    /// Estimated tempo in BPM (autocorrelation method).
    pub tempo_bpm: f32,
    /// 12-bin chroma vector — harmonic / key profile.
    pub chroma: [f32; 12],
}

impl FeatureVector {
    /// Current extraction schema version.
    pub const CURRENT_VERSION: u8 = 1;

    /// Flatten with dimensional weighting: MFCC×2, chroma×1.5, scalars×1.
    pub fn to_weighted_vec(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(30);
        for &c in &self.mfcc { v.push(c * 2.0); }
        for &c in &self.chroma { v.push(c * 1.5); }
        v.push(self.spectral_centroid);
        v.push(self.spectral_rolloff);
        v.push(self.zero_crossing_rate);
        v.push(self.rms_energy);
        v.push(self.tempo_bpm);
        v
    }

    /// Flatten without weighting (raw values).
    pub fn to_flat_vec(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(30);
        for &c in &self.mfcc { v.push(c); }
        for &c in &self.chroma { v.push(c); }
        v.push(self.spectral_centroid);
        v.push(self.spectral_rolloff);
        v.push(self.zero_crossing_rate);
        v.push(self.rms_energy);
        v.push(self.tempo_bpm);
        v
    }
}

// ---------------------------------------------------------------------------
// Song record
// ---------------------------------------------------------------------------

/// A fully indexed song record. Stored in the `songs` tree keyed by UUID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongRecord {
    pub id: String,
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u32,
    pub feature_vector: FeatureVector,
    pub indexed_at: DateTime<Utc>,
    /// Track loudness in dB RMS, used for ReplayGain normalization.
    /// Target: -18.0 dBFS. Gain = 10^((target - track_db) / 20).
    #[serde(default)]
    pub replay_gain_db: f32,
}

// ---------------------------------------------------------------------------
// Play event
// ---------------------------------------------------------------------------

/// A single play-event capturing how much of a track the user listened to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayEvent {
    pub event_id: String,
    pub song_id: String,
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub listened_secs: u32,
    pub song_duration_secs: u32,
    pub was_skipped: bool,
    pub listen_ratio: f32,
}

// ---------------------------------------------------------------------------
// Session state
// ---------------------------------------------------------------------------

/// Tracks the current listening session with an EMA centroid for mood-lock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub song_history: Vec<String>,
    pub centroid: Option<Vec<f32>>,
}

// ---------------------------------------------------------------------------
// Recommendation types
// ---------------------------------------------------------------------------

/// Parameters for a recommendation request.
#[derive(Debug, Clone)]
pub struct RecommendationRequest {
    pub current_song_id: String,
    pub session_state: SessionState,
    pub top_n: usize,
    pub exclude_ids: Vec<String>,
}

/// A scored recommendation candidate.
#[derive(Debug, Clone)]
pub struct RecommendedSong {
    pub record: SongRecord,
    pub final_score: f32,
    pub similarity_score: f32,
    pub session_match: f32,
    pub affinity: f32,
    pub cooldown: f32,
}

// ---------------------------------------------------------------------------
// Score cache (internal)
// ---------------------------------------------------------------------------

/// Pre-computed scores for a song, stored in the `scores` tree.
/// Eliminates the need to scan events on every recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SongScoreCache {
    /// Aggregate affinity clamped to [-5, +5].
    pub affinity: f32,
    /// Total number of play events.
    pub total_plays: u32,
    /// Unix timestamps (secs) of all plays, for cooldown computation.
    pub play_timestamps: Vec<i64>,
}

impl Default for SongScoreCache {
    fn default() -> Self {
        Self { affinity: 0.0, total_plays: 0, play_timestamps: Vec::new() }
    }
}

// ---------------------------------------------------------------------------
// Analysis result (internal, combined decode + metadata)
// ---------------------------------------------------------------------------

/// Result of a single-pass audio analysis combining feature extraction and
/// metadata reading.
#[derive(Debug, Clone)]
pub(crate) struct AnalysisResult {
    pub feature_vector: FeatureVector,
    pub duration_secs: u32,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Track loudness in dB RMS for normalization.
    pub rms_db: f32,
}
