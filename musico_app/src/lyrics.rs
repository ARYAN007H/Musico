//! LRC (Lyric) file parser for synced and unsynced lyrics.
//!
//! Supports the standard `[mm:ss.xx] text` format and sidecar `.lrc` files.

use std::path::Path;

/// A single timestamped lyric line.
#[derive(Debug, Clone)]
pub struct LrcLine {
    /// Timestamp in seconds.
    pub time_secs: f32,
    /// The lyric text for this line.
    pub text: String,
}

/// Parsed lyrics — either synced (with timestamps) or unsynced (plain text).
#[derive(Debug, Clone)]
pub enum Lyrics {
    Synced(Vec<LrcLine>),
    Unsynced(String),
    None,
}

/// Parse an LRC string into a list of timed lines.
pub fn parse_lrc(content: &str) -> Vec<LrcLine> {
    let mut lines = Vec::new();
    for raw_line in content.lines() {
        let raw_line = raw_line.trim();
        if raw_line.is_empty() {
            continue;
        }
        // Match pattern: [mm:ss.xx] text  or  [mm:ss] text
        let mut rest = raw_line;
        while let Some(bracket_start) = rest.find('[') {
            let bracket_end = match rest[bracket_start..].find(']') {
                Some(pos) => bracket_start + pos,
                None => break,
            };
            let tag = &rest[bracket_start + 1..bracket_end];
            let text_after = rest[bracket_end + 1..].trim();

            // Try to parse as timestamp mm:ss.xx
            if let Some(time) = parse_timestamp(tag) {
                if !text_after.is_empty() {
                    lines.push(LrcLine {
                        time_secs: time,
                        text: text_after.to_string(),
                    });
                }
                break;
            } else {
                // Skip metadata tags like [ar:Artist]
                rest = &rest[bracket_end + 1..];
            }
        }
    }
    lines.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap_or(std::cmp::Ordering::Equal));
    lines
}

/// Parse a timestamp string like "01:23.45" or "1:23" into seconds.
fn parse_timestamp(s: &str) -> Option<f32> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: f32 = parts[0].parse().ok()?;
    let seconds: f32 = parts[1].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

/// Try to find and load a sidecar .lrc file next to an audio file.
pub fn load_sidecar_lrc(audio_path: &str) -> Lyrics {
    let path = Path::new(audio_path);
    let lrc_path = path.with_extension("lrc");
    if lrc_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&lrc_path) {
            let lines = parse_lrc(&content);
            if !lines.is_empty() {
                return Lyrics::Synced(lines);
            }
            if !content.trim().is_empty() {
                return Lyrics::Unsynced(content);
            }
        }
    }
    Lyrics::None
}

/// Find the index of the current line based on playback position.
pub fn current_line_index(lines: &[LrcLine], position_secs: f32) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }
    // Binary search for the last line whose timestamp <= position
    let mut lo = 0;
    let mut hi = lines.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if lines[mid].time_secs <= position_secs {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo > 0 { Some(lo - 1) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let content = "[00:12.34] Hello world\n[01:00.00] Second line\n";
        let lines = parse_lrc(content);
        assert_eq!(lines.len(), 2);
        assert!((lines[0].time_secs - 12.34).abs() < 0.01);
        assert_eq!(lines[0].text, "Hello world");
        assert!((lines[1].time_secs - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_current_line() {
        let lines = vec![
            LrcLine { time_secs: 0.0, text: "First".into() },
            LrcLine { time_secs: 10.0, text: "Second".into() },
            LrcLine { time_secs: 20.0, text: "Third".into() },
        ];
        assert_eq!(current_line_index(&lines, 5.0), Some(0));
        assert_eq!(current_line_index(&lines, 15.0), Some(1));
        assert_eq!(current_line_index(&lines, 25.0), Some(2));
    }
}
