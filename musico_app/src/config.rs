//! Settings persistence — saves/loads user preferences to ~/.config/musico/settings.json

use iced::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_DIR_NAME: &str = "musico";
const SETTINGS_FILE: &str = "settings.json";

/// Persisted user settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub music_folder: Option<PathBuf>,
    pub accent_color_hex: String,
    pub volume: f32,
    pub library_view_mode: String, // "grid" | "list"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            music_folder: dirs::audio_dir(),
            accent_color_hex: "#9d8cff".to_string(),
            volume: 1.0,
            library_view_mode: "grid".to_string(),
        }
    }
}

impl AppConfig {
    /// Returns the config directory path (~/.config/musico/).
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join(APP_DIR_NAME))
    }

    /// Returns the full path to the settings file.
    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join(SETTINGS_FILE))
    }

    /// Loads settings from disk, returning defaults if not found.
    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Saves settings to disk.
    pub fn save(&self) {
        let dir = match Self::config_dir() {
            Some(d) => d,
            None => return,
        };

        if let Err(e) = std::fs::create_dir_all(&dir) {
            log::warn!("Failed to create config dir: {e}");
            return;
        }

        let path = dir.join(SETTINGS_FILE);
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log::warn!("Failed to write settings: {e}");
                }
            }
            Err(e) => log::warn!("Failed to serialize settings: {e}"),
        }
    }

    /// Convert accent hex to iced Color.
    pub fn accent_color(&self) -> Color {
        color_from_hex(&self.accent_color_hex)
    }
}

fn color_from_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return Color::from_rgb(0.616, 0.549, 1.0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(157);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(140);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    Color::from_rgb8(r, g, b)
}

/// Convert iced Color to hex string.
pub fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
