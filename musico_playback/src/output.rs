//! CPAL output stream and ring buffer bridge.
//!
//! Sets up the audio device, creates a lock-free ring buffer, and runs
//! the CPAL callback that pops samples and applies volume scaling.

use crate::error::PlaybackError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Manages the CPAL output stream and the producer half of the ring buffer.
///
/// The consumer half lives inside the CPAL audio callback.
pub struct AudioOutput {
    /// Kept alive — dropping the stream silences output.
    _stream: cpal::Stream,
    /// Producer side: the decoder thread pushes samples here.
    producer: HeapProducer<f32>,
    /// Device sample rate in Hz.
    sample_rate: u32,
    /// Number of output channels.
    channels: usize,
    /// Shared volume as an `AtomicU32` (bit-cast f32). The CPAL callback
    /// reads this without locking.
    volume: Arc<AtomicU32>,
    /// Shared mute flag.
    muted: Arc<std::sync::atomic::AtomicBool>,
}

impl AudioOutput {
    /// Initialises the default audio output device and stream.
    ///
    /// Creates a ring buffer of ~2 seconds capacity and starts the stream.
    pub fn new() -> Result<Self, PlaybackError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackError::NoOutputDevice)?;

        log::info!("Audio device: {:?}", device.name().unwrap_or_default());

        let supported = device
            .supported_output_configs()
            .map_err(|e| PlaybackError::StreamBuild(format!("{e}")))?;

        // Collect configs, prefer f32.
        let configs: Vec<_> = supported.collect();
        let chosen = configs
            .iter()
            .find(|c| c.sample_format() == cpal::SampleFormat::F32)
            .or_else(|| configs.iter().find(|c| c.sample_format() == cpal::SampleFormat::I16))
            .or_else(|| configs.first())
            .ok_or_else(|| PlaybackError::StreamBuild("no supported config".into()))?;

        // Choose sample rate: prefer 48000, then 44100, then device default.
        let min_sr = chosen.min_sample_rate().0;
        let max_sr = chosen.max_sample_rate().0;
        let sr = if (min_sr..=max_sr).contains(&48000) {
            48000
        } else if (min_sr..=max_sr).contains(&44100) {
            44100
        } else {
            max_sr
        };

        let config = chosen.with_sample_rate(cpal::SampleRate(sr));
        let channels = config.channels() as usize;
        let sample_rate = sr;

        log::info!("Output config: {sample_rate} Hz, {channels} ch, {:?}", config.sample_format());

        // Ring buffer: 2 seconds of audio.
        let buf_size = (sample_rate as usize) * channels * 2;
        let rb = HeapRb::<f32>::new(buf_size);
        let (producer, consumer) = rb.split();

        let volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
        let muted = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let vol_clone = Arc::clone(&volume);
        let muted_clone = Arc::clone(&muted);

        let stream = Self::build_stream(&device, &config.into(), consumer, vol_clone, muted_clone)?;
        stream.play().map_err(|e| PlaybackError::StreamBuild(format!("{e}")))?;

        Ok(Self {
            _stream: stream,
            producer,
            sample_rate,
            channels,
            volume,
            muted,
        })
    }

    /// Returns a mutable reference to the ring buffer producer.
    pub fn producer(&mut self) -> &mut HeapProducer<f32> {
        &mut self.producer
    }

    /// Device sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Number of output channels.
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Sets the volume (0.0–1.0) atomically for the CPAL callback.
    pub fn set_volume(&self, v: f32) {
        self.volume.store(f32::to_bits(v), Ordering::Relaxed);
    }

    /// Sets the muted flag atomically.
    pub fn set_muted(&self, m: bool) {
        self.muted.store(m, Ordering::Relaxed);
    }

    fn build_stream(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        mut consumer: HeapConsumer<f32>,
        volume: Arc<AtomicU32>,
        muted: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<cpal::Stream, PlaybackError> {
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let vol = if muted.load(Ordering::Relaxed) {
                        0.0
                    } else {
                        f32::from_bits(volume.load(Ordering::Relaxed))
                    };
                    for sample in data.iter_mut() {
                        *sample = match consumer.pop() {
                            Some(s) => s * vol,
                            None => 0.0, // underrun — fill silence
                        };
                    }
                },
                |err| log::error!("CPAL stream error: {err}"),
                None,
            )
            .map_err(|e| PlaybackError::StreamBuild(format!("{e}")))?;
        Ok(stream)
    }
}
