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
    #[serde(default = "default_palette_id")]
    pub palette_id: String,
    #[serde(default = "default_font_mode")]
    pub font_mode: String,
    pub volume: f32,
    pub library_view_mode: String, // "grid" | "list"
    // Legacy field — kept for backward compat on load
    #[serde(default)]
    pub accent_color_hex: String,

    // ── New settings ─────────────────────────────────────────────
    /// EQ enabled state.
    #[serde(default)]
    pub eq_enabled: bool,
    /// Active EQ preset ID.
    #[serde(default = "default_eq_preset")]
    pub eq_preset_id: String,
    /// Normalization mode: "off", "track", or "album".
    #[serde(default = "default_norm_mode")]
    pub normalization_mode: String,
    /// Last-used sleep timer duration in minutes (0 = none).
    #[serde(default)]
    pub last_sleep_timer_mins: u64,
    /// Crossfade enabled.
    #[serde(default)]
    pub crossfade_enabled: bool,
    /// Crossfade duration in seconds.
    #[serde(default = "default_crossfade_duration")]
    pub crossfade_duration: f32,
    /// Crossfade curve: "linear", "equal_power", "overlap".
    #[serde(default = "default_crossfade_curve")]
    pub crossfade_curve: String,
}

fn default_palette_id() -> String { "nebula".to_string() }
fn default_font_mode() -> String { "classic".to_string() }
fn default_eq_preset() -> String { "flat".to_string() }
fn default_norm_mode() -> String { "off".to_string() }
fn default_crossfade_duration() -> f32 { 3.0 }
fn default_crossfade_curve() -> String { "equal_power".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            music_folder: dirs::audio_dir(),
            palette_id: "nebula".to_string(),
            font_mode: "classic".to_string(),
            volume: 1.0,
            library_view_mode: "grid".to_string(),
            accent_color_hex: String::new(),
            eq_enabled: false,
            eq_preset_id: "flat".to_string(),
            normalization_mode: "off".to_string(),
            last_sleep_timer_mins: 0,
            crossfade_enabled: false,
            crossfade_duration: 3.0,
            crossfade_curve: "equal_power".to_string(),
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
            Ok(json) => {
                let mut config: Self = serde_json::from_str(&json).unwrap_or_default();
                // Migrate: if old config has accent_color_hex but no palette_id, default to nebula
                if config.palette_id.is_empty() {
                    config.palette_id = "nebula".to_string();
                }
                if config.font_mode.is_empty() {
                    config.font_mode = "classic".to_string();
                }
                if config.eq_preset_id.is_empty() {
                    config.eq_preset_id = "flat".to_string();
                }
                if config.normalization_mode.is_empty() {
                    config.normalization_mode = "off".to_string();
                }
                config
            }
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

    /// Convert accent hex to iced Color (legacy compat).
    #[allow(dead_code)]
    pub fn accent_color(&self) -> Color {
        if self.accent_color_hex.is_empty() {
            crate::theme::palette_by_id(&self.palette_id).primary
        } else {
            color_from_hex(&self.accent_color_hex)
        }
    }
}

#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
