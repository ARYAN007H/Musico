//! Listening session tracking with exponential moving average (EMA) centroid.
//!
//! The centroid is updated incrementally on each song change:
//!   centroid = (1 − α) × old_centroid + α × new_song_vec
//! with α = 0.3, making the centroid responsive to mood shifts within 2–3
//! songs instead of requiring a full 10-song window to rotate.

use chrono::Utc;
use uuid::Uuid;

use crate::errors::RecommenderError;
use crate::models::{SessionState, SongRecord, Store};
use crate::vector_store;

/// Maximum number of song IDs retained in the session history (for exclusion).
const MAX_SESSION_HISTORY: usize = 10;

/// EMA smoothing factor — higher = more responsive to recent songs.
const EMA_ALPHA: f32 = 0.3;

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn persist_session(store: &Store, state: &SessionState) -> Result<(), RecommenderError> {
    let bytes = bincode::serialize(state).map_err(|e| {
        RecommenderError::DbError(sled::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("session serialize: {e}"),
        )))
    })?;
    store
        .sessions
        .insert(b"current", bytes)
        .map_err(RecommenderError::DbError)?;
    Ok(())
}

/// Loads the current session from the `sessions` tree, if one exists.
pub(crate) fn load_session(store: &Store) -> Result<Option<SessionState>, RecommenderError> {
    match store.sessions.get(b"current").map_err(RecommenderError::DbError)? {
        Some(bytes) => {
            let state: SessionState = bincode::deserialize(&bytes)
                .map_err(|e| RecommenderError::DecodeError(format!("session deserialize: {e}")))?;
            Ok(Some(state))
        }
        None => Ok(None),
    }
}

/// Creates a fresh listening session with empty history and no centroid.
pub(crate) fn start_session(store: &Store) -> Result<SessionState, RecommenderError> {
    let state = SessionState {
        session_id: Uuid::new_v4().to_string(),
        started_at: Utc::now(),
        song_history: Vec::new(),
        centroid: None,
    };
    persist_session(store, &state)?;
    Ok(state)
}

/// Adds `song_id` to the session history and updates the centroid via EMA.
///
/// EMA update:
/// - First song: centroid = song_vector
/// - Subsequent: centroid = (1−α) × old + α × new
///
/// This makes the centroid responsive to mood shifts within 2–3 songs.
pub(crate) fn update_session(
    store: &Store,
    state: &mut SessionState,
    song_id: &str,
) -> Result<(), RecommenderError> {
    let record = vector_store::get_song_by_id(store, song_id)?
        .ok_or_else(|| RecommenderError::NotFound(format!("Song not found: {song_id}")))?;

    // Update history (keep last MAX_SESSION_HISTORY for exclusion list).
    state.song_history.push(song_id.to_string());
    if state.song_history.len() > MAX_SESSION_HISTORY {
        state.song_history.remove(0);
    }

    // EMA centroid update (no DB lookups of historical songs needed).
    let new_vec = record.feature_vector.to_weighted_vec();
    state.centroid = Some(match &state.centroid {
        Some(old) => old
            .iter()
            .zip(new_vec.iter())
            .map(|(o, n)| o * (1.0 - EMA_ALPHA) + n * EMA_ALPHA)
            .collect(),
        None => new_vec,
    });

    persist_session(store, state)?;
    Ok(())
}

/// Returns cosine similarity between the candidate and the session centroid.
/// Returns `1.0` if no centroid exists (first song — unconstrained).
pub fn get_session_similarity(state: &SessionState, candidate: &SongRecord) -> f32 {
    match &state.centroid {
        Some(centroid) => {
            let candidate_vec = candidate.feature_vector.to_weighted_vec();
            vector_store::cosine_similarity(centroid, &candidate_vec)
        }
        None => 1.0,
    }
}
