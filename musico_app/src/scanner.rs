use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use musico_recommender::{MusicRecommender, SongRecord};
use tokio::sync::mpsc;

pub async fn scan_and_index(
    folder: PathBuf,
    recommender: Arc<Mutex<MusicRecommender>>,
    progress_tx: mpsc::Sender<(usize, usize)>,
) -> Vec<SongRecord> {
    let paths: Vec<PathBuf> = WalkDir::new(folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_audio_file(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = paths.len();
    let mut records = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if let Some(path_str) = path.to_str() {
            let record_result = {
                let mut rec_guard = recommender.lock().unwrap();
                rec_guard.index_song(path_str)
            };

            if let Ok(record) = record_result {
                records.push(record);
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
