//! Native audio playback engine powered by Rodio & Symphonia.
//!
//! Provides hardware audio output, gapless/smooth stream decoding, playhead tracking,
//! volume adjustment, seeking, and pause/resume.

use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Arc<Mutex<Option<Sink>>>,
    current_bytes: Arc<Mutex<Option<Vec<u8>>>>,
    position_ms: Arc<AtomicU64>,
    duration_ms: Arc<AtomicU64>,
    volume: Arc<Mutex<f32>>,
    is_paused: Arc<AtomicBool>,
    is_playing: Arc<AtomicBool>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                log::error!("[BeatFlow Audio] Failed to initialize default audio output device: {}", e);
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle: handle,
            sink: Arc::new(Mutex::new(None)),
            current_bytes: Arc::new(Mutex::new(None)),
            position_ms: Arc::new(AtomicU64::new(0)),
            duration_ms: Arc::new(AtomicU64::new(0)),
            volume: Arc::new(Mutex::new(1.0)),
            is_paused: Arc::new(AtomicBool::new(false)),
            is_playing: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Loads and starts playback of audio from in-memory bytes (FLAC, MP3, AAC, WAV, Vorbis).
    pub fn load_bytes(&self, data: Vec<u8>) -> anyhow::Result<()> {
        let handle = self.stream_handle.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio output device available"))?;

        let cursor = Cursor::new(data.clone());
        let source = Decoder::new(cursor)?;

        // Recreate sink for clean playback
        let new_sink = Sink::try_new(handle)?;
        let vol = *self.volume.lock();
        new_sink.set_volume(vol);
        new_sink.append(source);
        new_sink.play();

        {
            let mut s = self.sink.lock();
            if let Some(old) = s.take() {
                old.stop();
            }
            *s = Some(new_sink);
        }

        *self.current_bytes.lock() = Some(data);
        self.position_ms.store(0, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// Plays a local file from disk.
    pub fn play_file(&self, path: &Path) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        self.load_bytes(bytes)
    }

    /// Pauses playback.
    pub fn pause(&self) {
        if let Some(ref sink) = *self.sink.lock() {
            sink.pause();
            self.is_paused.store(true, Ordering::SeqCst);
            self.is_playing.store(false, Ordering::SeqCst);
        }
    }

    /// Resumes playback.
    pub fn resume(&self) {
        if let Some(ref sink) = *self.sink.lock() {
            sink.play();
            self.is_paused.store(false, Ordering::SeqCst);
            self.is_playing.store(true, Ordering::SeqCst);
        }
    }

    /// Toggles play/pause.
    pub fn toggle_play(&self) {
        if self.is_paused.load(Ordering::SeqCst) {
            self.resume();
        } else if self.is_playing.load(Ordering::SeqCst) {
            self.pause();
        }
    }

    /// Stops playback and releases audio resources.
    pub fn stop(&self) {
        if let Some(sink) = self.sink.lock().take() {
            sink.stop();
        }
        self.position_ms.store(0, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        self.is_playing.store(false, Ordering::SeqCst);
    }

    /// Seeks to a given time offset.
    pub fn seek(&self, target_ms: u64) -> anyhow::Result<()> {
        let bytes_opt = self.current_bytes.lock().clone();
        if let Some(bytes) = bytes_opt {
            let handle = self.stream_handle.as_ref()
                .ok_or_else(|| anyhow::anyhow!("No audio output device available"))?;

            let cursor = Cursor::new(bytes);
            let source = Decoder::new(cursor)?;

            let new_sink = Sink::try_new(handle)?;
            let vol = *self.volume.lock();
            new_sink.set_volume(vol);

            let seek_duration = Duration::from_millis(target_ms);
            let _ = new_sink.try_seek(seek_duration);
            new_sink.append(source);

            if self.is_paused.load(Ordering::SeqCst) {
                new_sink.pause();
            } else {
                new_sink.play();
            }

            {
                let mut s = self.sink.lock();
                if let Some(old) = s.take() {
                    old.stop();
                }
                *s = Some(new_sink);
            }

            self.position_ms.store(target_ms, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Sets playback volume (0.0 to 1.0).
    pub fn set_volume(&self, vol: f32) {
        let clamped = vol.clamp(0.0, 1.0);
        *self.volume.lock() = clamped;
        if let Some(ref sink) = *self.sink.lock() {
            sink.set_volume(clamped);
        }
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock()
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn get_position_ms(&self) -> u64 {
        self.position_ms.load(Ordering::SeqCst)
    }

    pub fn set_position_ms(&self, ms: u64) {
        self.position_ms.store(ms, Ordering::SeqCst);
    }

    pub fn get_duration_ms(&self) -> u64 {
        self.duration_ms.load(Ordering::SeqCst)
    }

    pub fn set_duration_ms(&self, ms: u64) {
        self.duration_ms.store(ms, Ordering::SeqCst);
    }
}
