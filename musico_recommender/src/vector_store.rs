//! Song vector storage, cosine-similarity search, and deduplication.
//!
//! Songs are stored in the dedicated `songs` sled tree keyed by UUID.
//! A secondary index `path:{file_path}` in the default tree prevents
//! duplicate indexing of the same file.

use chrono::Utc;
use uuid::Uuid;

use crate::errors::RecommenderError;
use crate::extractor;
use crate::models::{SongRecord, Store};

// ---------------------------------------------------------------------------
// Cosine similarity
// ---------------------------------------------------------------------------

/// Computes cosine similarity between two equal-length vectors.
/// Returns `0.0` if either vector has zero magnitude.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "vectors must be the same length");

    let (mut dot, mut na, mut nb) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (&ai, &bi) in a.iter().zip(b.iter()) {
        dot += ai * bi;
        na += ai * ai;
        nb += bi * bi;
    }
    let denom = (na * nb).sqrt();
    if denom < 1e-8 { 0.0 } else { dot / denom }
}

// ---------------------------------------------------------------------------
// Indexing
// ---------------------------------------------------------------------------

/// Extracts features from `path`, reads metadata in a single pass, and
/// persists a new [`SongRecord`].  Returns the existing record if the file
/// has already been indexed (dedup guard).
pub(crate) fn index_song(store: &Store, path: &str) -> Result<SongRecord, RecommenderError> {
    // ---- Dedup guard: check secondary path index ----
    let path_key = format!("path:{path}");
    if let Some(id_bytes) = store.db.get(path_key.as_bytes()).map_err(RecommenderError::DbError)? {
        let existing_id = String::from_utf8_lossy(&id_bytes).to_string();
        if let Some(rec) = get_song_by_id(store, &existing_id)? {
            return Ok(rec);
        }
        // ID referenced but record missing — fall through to re-index.
    }

    // ---- Single-pass analysis (metadata + features) ----
    let result = extractor::analyze_file(path)?;

    let id = Uuid::new_v4().to_string();
    let record = SongRecord {
        id: id.clone(),
        file_path: path.to_string(),
        title: result.title,
        artist: result.artist,
        album: result.album,
        duration_secs: result.duration_secs,
        feature_vector: result.feature_vector,
        indexed_at: Utc::now(),
        replay_gain_db: result.rms_db,
    };

    let value = bincode::serialize(&record).map_err(|e| {
        RecommenderError::DbError(sled::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("bincode serialize: {e}"),
        )))
    })?;

    // Write to songs tree.
    store
        .songs
        .insert(id.as_bytes(), value)
        .map_err(RecommenderError::DbError)?;

    // Write secondary path index.
    store
        .db
        .insert(path_key.as_bytes(), id.as_bytes())
        .map_err(RecommenderError::DbError)?;

    // Increment counter.
    store.increment_song_count().map_err(RecommenderError::DbError)?;

    store.songs.flush().map_err(RecommenderError::DbError)?;

    Ok(record)
}

/// Indexes a song without flushing — used by batch operations.
pub(crate) fn index_song_no_flush(store: &Store, path: &str) -> Result<SongRecord, RecommenderError> {
    let path_key = format!("path:{path}");
    if let Some(id_bytes) = store.db.get(path_key.as_bytes()).map_err(RecommenderError::DbError)? {
        let existing_id = String::from_utf8_lossy(&id_bytes).to_string();
        if let Some(rec) = get_song_by_id(store, &existing_id)? {
            return Ok(rec);
        }
    }

    let result = extractor::analyze_file(path)?;
    let id = Uuid::new_v4().to_string();
    let record = SongRecord {
        id: id.clone(),
        file_path: path.to_string(),
        title: result.title,
        artist: result.artist,
        album: result.album,
        duration_secs: result.duration_secs,
        feature_vector: result.feature_vector,
        indexed_at: Utc::now(),
        replay_gain_db: result.rms_db,
    };

    let value = bincode::serialize(&record).map_err(|e| {
        RecommenderError::DbError(sled::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("bincode serialize: {e}"),
        )))
    })?;

    store.songs.insert(id.as_bytes(), value).map_err(RecommenderError::DbError)?;
    store.db.insert(path_key.as_bytes(), id.as_bytes()).map_err(RecommenderError::DbError)?;
    store.increment_song_count().map_err(RecommenderError::DbError)?;

    Ok(record)
}

// ---------------------------------------------------------------------------
// Retrieval
// ---------------------------------------------------------------------------

/// Loads every [`SongRecord`] from the `songs` tree.
pub(crate) fn get_all_songs(store: &Store) -> Result<Vec<SongRecord>, RecommenderError> {
    let mut songs = Vec::new();
    for entry in store.songs.iter() {
        let (_, value) = entry.map_err(RecommenderError::DbError)?;
        let record: SongRecord = bincode::deserialize(&value)
            .map_err(|e| RecommenderError::DecodeError(format!("bincode deserialize: {e}")))?;
        songs.push(record);
    }
    Ok(songs)
}

/// Loads a single [`SongRecord`] by UUID.
pub(crate) fn get_song_by_id(store: &Store, song_id: &str) -> Result<Option<SongRecord>, RecommenderError> {
    match store.songs.get(song_id.as_bytes()).map_err(RecommenderError::DbError)? {
        Some(bytes) => {
            let record: SongRecord = bincode::deserialize(&bytes)
                .map_err(|e| RecommenderError::DecodeError(format!("bincode deserialize: {e}")))?;
            Ok(Some(record))
        }
        None => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Similarity search
// ---------------------------------------------------------------------------

/// A similarity result pairing a song with its cosine-similarity score.
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub record: SongRecord,
    pub score: f32,
}

/// Finds the `top_n` most similar songs using the in-memory vector cache.
///
/// `cache` contains `(song_id, weighted_vec)` pairs built at startup and
/// maintained incrementally.  Only the final top-N records are fetched from
/// sled, so this does at most `top_n` disk reads regardless of library size.
pub(crate) fn find_similar_cached(
    store: &Store,
    song_id: &str,
    top_n: usize,
    cache: &[(String, Vec<f32>)],
) -> Result<Vec<SimilarityResult>, RecommenderError> {
    // Find the target vector in the cache.
    let target_vec = cache
        .iter()
        .find(|(id, _)| id == song_id)
        .map(|(_, v)| v.clone())
        .ok_or_else(|| RecommenderError::NotFound(format!("Song not in cache: {song_id}")))?;

    // Score all candidates from cache (in-memory, microseconds).
    let mut scored: Vec<(&str, f32)> = cache
        .iter()
        .filter(|(id, _)| id != song_id)
        .map(|(id, v)| (id.as_str(), cosine_similarity(&target_vec, v)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_n);

    // Fetch only the top-N full records from sled.
    let mut results = Vec::with_capacity(scored.len());
    for (id, score) in scored {
        if let Some(record) = get_song_by_id(store, id)? {
            results.push(SimilarityResult { record, score });
        }
    }

    Ok(results)
}
