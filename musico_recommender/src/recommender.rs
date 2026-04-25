//! Final recommendation pipeline — combines vector similarity, session context,
//! play-history affinity, cooldown decay, and a discovery bonus into a ranked
//! list with weighted random sampling.

use rand::Rng;

use crate::errors::RecommenderError;
use crate::history;
use crate::models::{RecommendationRequest, RecommendedSong, Store};
use crate::session;
use crate::vector_store;

/// Runs the full recommendation pipeline.
///
/// Uses the in-memory `vector_cache` for similarity search (zero sled I/O for
/// the scan), and reads affinity/cooldown from the pre-computed `scores` tree
/// (O(1) per candidate).
///
/// # Pipeline
///
/// 1. Guard: ≥ 3 songs (O(1) counter read).
/// 2. Top 50 similar via in-memory cache.
/// 3. Filter excluded IDs.
/// 4. Score: similarity × 0.35 + session × 0.30 + affinity × 0.20 + discovery × 0.15.
/// 5. Gate: suppress cooldown > 0.7.
/// 6. Weighted random sample from top 10 by score².
pub(crate) fn recommend(
    store: &Store,
    req: &RecommendationRequest,
    vector_cache: &[(String, Vec<f32>)],
) -> Result<Vec<RecommendedSong>, RecommenderError> {
    // Step 1: O(1) song count guard.
    let total = store.song_count().map_err(RecommenderError::DbError)?;
    if total < 3 {
        return Err(RecommenderError::InsufficientLibrary);
    }

    // Step 2: Top 50 similar songs from in-memory cache.
    let similar = vector_store::find_similar_cached(store, &req.current_song_id, 50, vector_cache)?;

    // Step 3: Exclusion set.
    let exclude_set: std::collections::HashSet<&str> =
        req.exclude_ids.iter().map(|s| s.as_str()).collect();

    let mut candidates: Vec<RecommendedSong> = Vec::new();

    for sim in similar {
        if exclude_set.contains(sim.record.id.as_str()) {
            continue;
        }

        // Step 4: Multi-signal scoring (all O(1) reads from scores tree).
        let similarity_score = sim.score;
        let session_match = session::get_session_similarity(&req.session_state, &sim.record);
        let raw_affinity = history::get_song_affinity(store, &sim.record.id)?;
        let affinity = (raw_affinity + 5.0) / 10.0;
        let cooldown = history::get_cooldown_score(store, &sim.record.id)?;

        let play_count = history::play_count(store, &sim.record.id)?;
        let discovery_bonus: f32 = if play_count == 0 { 1.0 } else { 0.0 };

        let final_score = (similarity_score * 0.35)
            + (session_match * 0.30)
            + (affinity * 0.20)
            + (discovery_bonus * 0.15);

        // Step 5: Cooldown gate.
        if cooldown > 0.7 {
            continue;
        }

        candidates.push(RecommendedSong {
            record: sim.record,
            final_score,
            similarity_score,
            session_match,
            affinity,
            cooldown,
        });
    }

    // Sort by final_score descending.
    candidates.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Step 6: Weighted random sampling from top 10.
    let pool_size = candidates.len().min(10);
    if pool_size == 0 {
        return Ok(Vec::new());
    }

    let top_pool = &candidates[..pool_size];
    Ok(weighted_sample(top_pool, req.top_n))
}

/// Weighted random sampling without replacement.  Selection probability is
/// proportional to `final_score²`.
fn weighted_sample(pool: &[RecommendedSong], n: usize) -> Vec<RecommendedSong> {
    let mut rng = rand::thread_rng();
    let mut available: Vec<(usize, f32)> = pool
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.final_score * s.final_score))
        .collect();

    let mut selected = Vec::with_capacity(n);

    for _ in 0..n {
        if available.is_empty() {
            break;
        }
        let total_weight: f32 = available.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            if let Some(&(idx, _)) = available.first() {
                selected.push(pool[idx].clone());
                available.remove(0);
            }
            continue;
        }

        let threshold = rng.gen::<f32>() * total_weight;
        let mut cumulative = 0.0_f32;
        let mut pick = 0;
        for (j, &(_, w)) in available.iter().enumerate() {
            cumulative += w;
            if cumulative >= threshold {
                pick = j;
                break;
            }
        }

        let (idx, _) = available.remove(pick);
        selected.push(pool[idx].clone());
    }

    selected
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::vector_store::cosine_similarity;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0_f32, 0.0, 0.0];
        let b = vec![1.0_f32, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6, "Expected 1.0, got {sim}");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0_f32, 0.0, 0.0];
        let b = vec![0.0_f32, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6, "Expected 0.0, got {sim}");
    }

    #[test]
    fn test_cooldown_zero_for_unplayed() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().to_str().unwrap()).unwrap();
        let score = history::get_cooldown_score(&store, "nonexistent-song-id").unwrap();
        assert!(score.abs() < 1e-6, "Expected 0.0, got {score}");
    }

    #[test]
    fn test_strong_skip_negative_affinity() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().to_str().unwrap()).unwrap();

        let song_id = "test-song-001";
        let event = PlayEvent {
            event_id: Uuid::new_v4().to_string(),
            song_id: song_id.to_string(),
            session_id: "sess-001".to_string(),
            started_at: Utc::now(),
            listened_secs: 3,
            song_duration_secs: 240,
            was_skipped: true,
            listen_ratio: 3.0 / 240.0,
        };

        history::log_play_event(&store, &event).unwrap();
        let affinity = history::get_song_affinity(&store, song_id).unwrap();
        assert!(affinity < 0.0, "Expected negative affinity, got {affinity}");
    }

    #[test]
    fn test_implicit_like_positive_affinity() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().to_str().unwrap()).unwrap();

        let song_id = "test-song-002";
        let event = PlayEvent {
            event_id: Uuid::new_v4().to_string(),
            song_id: song_id.to_string(),
            session_id: "sess-001".to_string(),
            started_at: Utc::now(),
            listened_secs: 200,
            song_duration_secs: 240,
            was_skipped: false,
            listen_ratio: 200.0 / 240.0,
        };

        history::log_play_event(&store, &event).unwrap();
        let affinity = history::get_song_affinity(&store, song_id).unwrap();
        assert!(affinity > 0.0, "Expected positive affinity, got {affinity}");
    }

    #[test]
    fn test_weighted_sample_count() {
        let dummy_fv = FeatureVector {
            version: 1,
            mfcc: [0.5; 13],
            spectral_centroid: 0.5,
            spectral_rolloff: 0.5,
            zero_crossing_rate: 0.5,
            rms_energy: 0.5,
            tempo_bpm: 0.5,
            chroma: [0.5; 12],
        };
        let dummy_record = SongRecord {
            id: "dummy".to_string(),
            file_path: "/dev/null".to_string(),
            title: "Test".to_string(),
            artist: String::new(),
            album: String::new(),
            duration_secs: 180,
            feature_vector: dummy_fv,
            indexed_at: Utc::now(),
        };

        let pool: Vec<RecommendedSong> = (0..10)
            .map(|i| RecommendedSong {
                record: SongRecord { id: format!("song-{i}"), ..dummy_record.clone() },
                final_score: 1.0 - (i as f32 * 0.05),
                similarity_score: 0.9,
                session_match: 0.8,
                affinity: 0.5,
                cooldown: 0.1,
            })
            .collect();

        let result = weighted_sample(&pool, 5);
        assert_eq!(result.len(), 5, "Expected 5 samples, got {}", result.len());
    }
}
