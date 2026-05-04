use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use musico_recommender::{MusicRecommender, SongRecord};
use tokio::sync::mpsc;

/// Progress update sent from the scanner to the UI.
#[derive(Debug, Clone)]
pub enum ScanProgress {
    /// (done, total, current_filename)
    Progress(usize, usize, String),
    /// Scanning complete — all records.
    Done(Vec<SongRecord>),
}

pub async fn scan_and_index(
    folder: PathBuf,
    recommender: Arc<Mutex<MusicRecommender>>,
    progress_tx: mpsc::Sender<(usize, usize)>,
) -> Vec<SongRecord> {
    // Step 1: Collect audio file paths (no lock needed).
    let paths: Vec<PathBuf> = WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_audio_file(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = paths.len();

    // Step 2: Clear existing index so re-index truly re-scans.
    {
        let mut rec_guard = recommender.lock().unwrap();
        let _ = rec_guard.clear_song_index();
    }

    let mut records = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if let Some(path_str) = path.to_str() {
            // Step 3a: Analyse OUTSIDE the lock (CPU-heavy, ~1-3s per file).
            let analysis = musico_recommender::extractor::analyze_file(path_str);

            // Step 3b: If analysis succeeded, write to DB with a brief lock.
            if let Ok(result) = analysis {
                let mut rec_guard = recommender.lock().unwrap();
                if let Ok(record) = rec_guard.index_from_result(path_str, result) {
                    records.push(record);
                }
                // Lock is dropped here — UI stays responsive.
            } else {
                log::warn!("Failed to analyze: {}", path_str);
            }
        }
        let _ = progress_tx.send((i + 1, total)).await;
    }
    records
}

fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext = ext.to_lowercase();
        matches!(
            ext.as_str(),
            "mp3" | "flac" | "ogg" | "wav" | "m4a" | "aac" | "opus" | "alac" | "wma" | "mp4" | "m4b"
        )
    } else {
        false
    }
}
