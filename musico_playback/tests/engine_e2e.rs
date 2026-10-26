//! End-to-end engine test: decode a real WAV → resample (if needed) → CPAL
//! output → event stream. Runs silently (volume 0) but exercises the full
//! pipeline. Requires a working audio device.

use std::time::{Duration, Instant};

use musico_playback::events::PlaybackEvent;
use musico_playback::state::SongInfo;
use musico_playback::PlaybackEngine;

fn test_song(path: &str, duration_secs: f32) -> SongInfo {
    SongInfo {
        id: "test-1".into(),
        file_path: path.into(),
        title: "Test Tone".into(),
        artist: "Engine Test".into(),
        album: "Testing".into(),
        duration_secs,
        cover_art: None,
    }
}

#[test]
fn engine_plays_file_and_emits_position_and_end() {
    let wav = std::env::var("MUSICO_TEST_WAV").unwrap_or_else(|_| "/tmp/musico-test-tone.wav".into());
    if !std::path::Path::new(&wav).exists() {
        panic!(
            "Test WAV missing — run scripts/gen_test_tone.sh or set MUSICO_TEST_WAV"
        );
    }

    let engine = PlaybackEngine::new().expect("engine init (audio device?)");
    engine.set_volume(0.0).expect("set volume");

    // The WAV we generate is ~3s at 44.1kHz.
    let song = test_song(&wav, 3.0);
    engine.play(song).expect("play");

    let mut saw_position_update = false;
    let mut saw_playing = false;
    let mut ended = false;
    let start = Instant::now();

    while start.elapsed() < Duration::from_secs(15) {
        for ev in engine.poll_events() {
            match ev {
                PlaybackEvent::Playing(_) => saw_playing = true,
                PlaybackEvent::PositionUpdate { position_secs, .. } => {
                    if position_secs > 0.5 {
                        saw_position_update = true;
                    }
                }
                PlaybackEvent::SongEnded { .. } => {
                    ended = true;
                    break;
                }
                PlaybackEvent::Error(msg) => panic!("engine error: {msg}"),
                _ => {}
            }
        }
        if ended {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(saw_playing, "never received Playing event");
    assert!(saw_position_update, "position never advanced — decode/output stalled");
    assert!(ended, "song never ended within 15s");

    let state = engine.state();
    assert_eq!(state.status, musico_playback::state::PlaybackStatus::Stopped);
}

#[test]
fn engine_seek_updates_position() {
    let wav = std::env::var("MUSICO_TEST_WAV").unwrap_or_else(|_| "/tmp/musico-test-tone.wav".into());
    if !std::path::Path::new(&wav).exists() {
        panic!("Test WAV missing");
    }

    let engine = PlaybackEngine::new().expect("engine init (audio device?)");
    engine.set_volume(0.0).expect("set volume");
    engine.play(test_song(&wav, 3.0)).expect("play");

    // Let it start
    let start = Instant::now();
    loop {
        if engine.poll_events().iter().any(|e| matches!(e, PlaybackEvent::PositionUpdate { .. })) {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(5), "no position updates after play");
        std::thread::sleep(Duration::from_millis(50));
    }

    engine.seek(2.0).expect("seek");
    let start = Instant::now();
    let mut saw_seeked = false;
    while start.elapsed() < Duration::from_secs(3) {
        for ev in engine.poll_events() {
            match ev {
                PlaybackEvent::Seeked { position_secs } => {
                    assert!((position_secs - 2.0).abs() < 0.5, "seeked to {position_secs}");
                    saw_seeked = true;
                }
                PlaybackEvent::PositionUpdate { position_secs, .. } if position_secs >= 2.0 => {
                    assert!(saw_seeked, "position advanced to target before Seeked event");
                    // Seek verified end-to-end.
                    return;
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("position never reached seek target");
}
