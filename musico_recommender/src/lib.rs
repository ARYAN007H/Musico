//! # Musico — Offline Music Recommendation Engine
//!
//! A self-contained Rust library crate for analysing audio files, computing
//! feature vectors, tracking listening sessions, and producing personalised
//! song recommendations.  100 % offline — zero network calls.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use musico_recommender::MusicRecommender;
//!
//! let mut engine = MusicRecommender::new("/tmp/musico_db").unwrap();
//!
//! // Index some songs
//! let song = engine.index_song("/music/track.flac").unwrap();
//!
//! // Later, when the user starts playing a song:
//! engine.on_song_changed(&song.id).unwrap();
//!
//! // Get recommendations based on what's playing
//! let recs = engine.get_recommendations(&song.id, 5).unwrap();
//! ```

pub mod errors;
pub mod extractor;
pub mod history;
pub mod models;
pub mod playlists;
pub mod recommender;
pub mod session;
pub mod stats;
pub mod vector_store;

pub use errors::RecommenderError;
pub use models::*;
pub use stats::ListeningStats;
pub use playlists::SmartPlaylist;

use chrono::Utc;
use models::Store;
use uuid::Uuid;

/// The primary entry-point for the Musico recommendation engine.
///
/// Wraps the sled database (with separate trees for songs, events, sessions,
/// and scores), an in-memory vector cache for fast similarity search, and the
/// current session state.
pub struct MusicRecommender {
    /// Internal database handle with named trees.
    store: Store,
    /// Current listening session.
    session: SessionState,
    /// In-memory cache of `(song_id, weighted_feature_vec)` for O(N) similarity
    /// search without touching sled.  ~600 KB for 5 000 songs.
    vector_cache: Vec<(String, Vec<f32>)>,
}

impl MusicRecommender {
    /// Opens (or creates) the database at `db_path` and initialises the engine.
    ///
    /// Loads the in-memory vector cache from the `songs` tree and resumes the
    /// previous session if one exists.
    pub fn new(db_path: &str) -> Result<Self, RecommenderError> {
        let store = Store::open(db_path).map_err(RecommenderError::DbError)?;

        // Resume or start a session.
        let session_state = match session::load_session(&store)? {
            Some(s) => s,
            None => session::start_session(&store)?,
        };

        // Build the vector cache from all indexed songs.
        let all_songs = vector_store::get_all_songs(&store)?;
        let vector_cache: Vec<(String, Vec<f32>)> = all_songs
            .iter()
            .map(|s| (s.id.clone(), s.feature_vector.to_weighted_vec()))
            .collect();

        Ok(Self {
            store,
            session: session_state,
            vector_cache,
        })
    }

    /// Analyses the audio file at `file_path`, extracts features + metadata in
    /// a single pass, and persists a new [`SongRecord`].
    ///
    /// If the file has already been indexed, returns the existing record
    /// without re-extracting (dedup guard).
    ///
    /// The in-memory vector cache is updated incrementally.
    pub fn index_song(&mut self, file_path: &str) -> Result<SongRecord, RecommenderError> {
        let record = vector_store::index_song(&self.store, file_path)?;

        // Update cache if this is a genuinely new record.
        if !self.vector_cache.iter().any(|(id, _)| id == &record.id) {
            self.vector_cache
                .push((record.id.clone(), record.feature_vector.to_weighted_vec()));
        }

        Ok(record)
    }

    /// Batch-indexes multiple files with a single sled flush at the end.
    ///
    /// `progress_cb` is called after each file with `(completed, total)`.
    /// Errors on individual files are collected and skipped — successfully
    /// indexed songs are returned.
    pub fn index_songs_batch(
        &mut self,
        paths: &[&str],
        progress_cb: impl Fn(usize, usize),
    ) -> Result<Vec<SongRecord>, RecommenderError> {
        let total = paths.len();
        let mut records = Vec::with_capacity(total);

        for (i, path) in paths.iter().enumerate() {
            match vector_store::index_song_no_flush(&self.store, path) {
                Ok(record) => {
                    if !self.vector_cache.iter().any(|(id, _)| id == &record.id) {
                        self.vector_cache
                            .push((record.id.clone(), record.feature_vector.to_weighted_vec()));
                    }
                    records.push(record);
                }
                Err(_) => { /* skip failed files */ }
            }
            progress_cb(i + 1, total);
        }

        // Single flush for the entire batch.
        self.store.songs.flush().map_err(RecommenderError::DbError)?;
        self.store.db.flush().map_err(RecommenderError::DbError)?;

        Ok(records)
    }

    /// Produces up to `top_n` recommended songs based on the currently playing
    /// track, session context, play history, and cooldown state.
    pub fn get_recommendations(
        &mut self,
        current_song_id: &str,
        top_n: usize,
    ) -> Result<Vec<RecommendedSong>, RecommenderError> {
        let req = RecommendationRequest {
            current_song_id: current_song_id.to_string(),
            session_state: self.session.clone(),
            top_n,
            exclude_ids: self.session.song_history.clone(),
        };
        recommender::recommend(&self.store, &req, &self.vector_cache)
    }

    /// Logs a play event for the given song.
    ///
    /// Skip classification is computed automatically from the listen ratio.
    /// The score cache for this song is updated atomically.
    pub fn log_listen(
        &self,
        song_id: &str,
        listened_secs: u32,
        duration_secs: u32,
    ) -> Result<(), RecommenderError> {
        let ratio = if duration_secs > 0 {
            (listened_secs as f32 / duration_secs as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let event = PlayEvent {
            event_id: Uuid::new_v4().to_string(),
            song_id: song_id.to_string(),
            session_id: self.session.session_id.clone(),
            started_at: Utc::now(),
            listened_secs,
            song_duration_secs: duration_secs,
            was_skipped: history::is_skip(ratio),
            listen_ratio: ratio,
        };

        history::log_play_event(&self.store, &event)
    }

    /// Notifies the engine that playback has transitioned to a new song.
    ///
    /// Updates the session centroid via EMA — responsive to mood shifts
    /// within 2–3 songs.
    pub fn on_song_changed(&mut self, new_song_id: &str) -> Result<(), RecommenderError> {
        session::update_session(&self.store, &mut self.session, new_song_id)
    }

    /// Returns all indexed [`SongRecord`]s.
    pub fn get_all_songs(&self) -> Result<Vec<SongRecord>, RecommenderError> {
        vector_store::get_all_songs(&self.store)
    }

    /// Looks up a single song by UUID. Returns `Ok(None)` if not found.
    pub fn get_song_by_id(&self, id: &str) -> Result<Option<SongRecord>, RecommenderError> {
        vector_store::get_song_by_id(&self.store, id)
    }

    /// Indexes a song from a pre-computed [`AnalysisResult`], skipping the
    /// decode step.  Used by the scanner to separate analysis (no lock) from
    /// DB writes (lock held briefly).
    pub fn index_from_result(
        &mut self,
        file_path: &str,
        result: models::AnalysisResult,
    ) -> Result<SongRecord, RecommenderError> {
        let record = vector_store::index_from_result(&self.store, file_path, result)?;
        if !self.vector_cache.iter().any(|(id, _)| id == &record.id) {
            self.vector_cache
                .push((record.id.clone(), record.feature_vector.to_weighted_vec()));
        }
        Ok(record)
    }

    /// Clears all indexed songs and path indices.  Use before a full re-scan.
    pub fn clear_song_index(&mut self) -> Result<(), RecommenderError> {
        // Remove all path:* keys from the default tree.
        let mut path_keys = Vec::new();
        for entry in self.store.db.iter() {
            let (key, _) = entry.map_err(RecommenderError::DbError)?;
            if key.starts_with(b"path:") {
                path_keys.push(key);
            }
        }
        for key in path_keys {
            self.store.db.remove(key).map_err(RecommenderError::DbError)?;
        }
        // Clear the songs tree.
        self.store.songs.clear().map_err(RecommenderError::DbError)?;
        // Reset song count.
        self.store.db.insert(b"meta:song_count", &0u64.to_le_bytes())
            .map_err(RecommenderError::DbError)?;
        // Clear in-memory vector cache.
        self.vector_cache.clear();
        self.store.db.flush().map_err(RecommenderError::DbError)?;
        Ok(())
    }

    /// Starts a brand-new listening session, resetting the centroid and history.
    pub fn new_session(&mut self) -> Result<(), RecommenderError> {
        self.session = session::start_session(&self.store)?;
        Ok(())
    }

    /// Returns a reference to the current [`SessionState`].
    pub fn current_session(&self) -> &SessionState {
        &self.session
    }

    /// Returns a reference to the underlying sled database handle.
    pub fn db(&self) -> &sled::Db {
        &self.store.db
    }

    /// Compute listening statistics from the event history.
    pub fn get_stats(&self) -> Result<ListeningStats, RecommenderError> {
        stats::get_stats(&self.store)
    }

    /// Load all saved smart playlists.
    pub fn get_playlists(&self) -> Result<Vec<SmartPlaylist>, RecommenderError> {
        playlists::load_playlists(&self.store)
    }

    /// Save a smart playlist.
    pub fn save_playlist(&self, playlist: &SmartPlaylist) -> Result<(), RecommenderError> {
        playlists::save_playlist(&self.store, playlist)
    }

    /// Delete a smart playlist by ID.
    pub fn delete_playlist(&self, playlist_id: &str) -> Result<(), RecommenderError> {
        playlists::delete_playlist(&self.store, playlist_id)
    }
}
