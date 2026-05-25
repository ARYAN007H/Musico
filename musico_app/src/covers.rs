// covers.rs — Musico cover art cache manager
// Handles disk caching of scaled-down cover art thumbnails.

use std::path::PathBuf;
use std::fs;
use log;

/// Retrieves the system cache directory for Musico cover thumbnails.
pub fn cover_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("musico")
        .join("covers")
}

/// Gets the absolute path to the cached JPEG thumbnail of a song cover.
pub fn get_cover_path(song_id: &str) -> PathBuf {
    cover_cache_dir().join(format!("{}.jpg", song_id))
}

/// Ensures the cover art for a song is cached as a thumbnail on disk.
/// Probes the audio metadata, resizes the image to 160x160, and saves it.
pub fn ensure_cover_cached(song_id: &str, file_path: &str) -> Option<PathBuf> {
    let cache_path = get_cover_path(song_id);
    if cache_path.exists() {
        return Some(cache_path);
    }

    // Try to extract cover art using a fast symphonia probe
    if let Ok((_decoder, song_info)) = musico_playback::decoder::AudioDecoder::new(file_path) {
        if let Some(art_bytes) = song_info.cover_art {
            let cache_dir = cover_cache_dir();
            if fs::create_dir_all(&cache_dir).is_ok() {
                // Resize image using `image` crate to save RAM and disk space
                if let Ok(img) = image::load_from_memory(&art_bytes) {
                    // Resize to a thumbnail (160x160 is perfect for grid/list)
                    let thumbnail = img.thumbnail(160, 160);
                    if thumbnail.save(&cache_path).is_ok() {
                        log::info!("Cached 160x160 thumbnail for song ID {}", song_id);
                        return Some(cache_path);
                    }
                }
            }
        }
    }
    None
}
