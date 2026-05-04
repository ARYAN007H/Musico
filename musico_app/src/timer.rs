//! Sleep timer — stop/fade-out playback after a user-defined duration.
//!
//! Checked on every PlaybackTick (~250 ms). When the timer fires,
//! it gradually reduces volume over 30 s, then pauses playback.

use std::time::{Duration, Instant};

/// Sleep timer state.
#[derive(Debug, Clone)]
pub struct SleepTimer {
    /// When the timer was started.
    started_at: Instant,
    /// Total timer duration.
    total_duration: Duration,
    /// Duration of the fade-out at the end (default 30 s).
    fade_duration: Duration,
}

/// Timer status returned on each tick.
#[derive(Debug, Clone, PartialEq)]
pub enum TimerStatus {
    /// Timer is active, not yet fading.
    Active {
        remaining: Duration,
    },
    /// Timer is in the fade-out phase.
    Fading {
        /// Volume multiplier [0.0, 1.0] to apply.
        volume_factor: f32,
        remaining: Duration,
    },
    /// Timer has expired — pause playback.
    Expired,
}

/// Preset durations for quick selection.
pub const TIMER_PRESETS: &[(u64, &str)] = &[
    (15, "15 min"),
    (30, "30 min"),
    (45, "45 min"),
    (60, "1 hour"),
    (90, "1.5 hours"),
    (120, "2 hours"),
];

impl SleepTimer {
    /// Creates a new sleep timer that fires after `minutes`.
    pub fn new(minutes: u64) -> Self {
        let total = Duration::from_secs(minutes * 60);
        let fade = Duration::from_secs(30.min(minutes * 60 / 2));
        Self {
            started_at: Instant::now(),
            total_duration: total,
            fade_duration: fade,
        }
    }

    /// Returns the total duration of this timer.
    pub fn total_minutes(&self) -> u64 {
        self.total_duration.as_secs() / 60
    }

    /// Check the timer status. Call on every PlaybackTick.
    pub fn tick(&self) -> TimerStatus {
        let elapsed = self.started_at.elapsed();
        if elapsed >= self.total_duration {
            return TimerStatus::Expired;
        }

        let remaining = self.total_duration - elapsed;

        if remaining <= self.fade_duration {
            // We're in the fade-out window.
            let fade_progress = 1.0 - (remaining.as_secs_f32() / self.fade_duration.as_secs_f32());
            let volume_factor = 1.0 - fade_progress; // 1.0 → 0.0
            TimerStatus::Fading {
                volume_factor,
                remaining,
            }
        } else {
            TimerStatus::Active { remaining }
        }
    }

    /// Remaining time as a formatted string "MM:SS".
    pub fn remaining_display(&self) -> String {
        let elapsed = self.started_at.elapsed();
        if elapsed >= self.total_duration {
            return "0:00".to_string();
        }
        let remaining = self.total_duration - elapsed;
        let mins = remaining.as_secs() / 60;
        let secs = remaining.as_secs() % 60;
        format!("{}:{:02}", mins, secs)
    }
}
