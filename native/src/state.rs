//! Shared playback state (was in main.rs): the queue, the current track and
//! the generation counter used to discard stale async results.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::images::ImageCache;

#[derive(Clone, Debug)]
pub struct TrackItem {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub duration: f32,
}

#[derive(Clone)]
pub struct PlaybackState {
    /// Bumped on every track switch; async results from an older generation
    /// are dropped instead of stepping on the live one.
    pub generation: Arc<AtomicU64>,
    pub active_shared: Arc<Mutex<Option<Arc<crate::backend::stream::Shared>>>>,
    pub current_track: Arc<Mutex<Option<TrackItem>>>,
    pub queue: Arc<Mutex<Vec<TrackItem>>>,
    pub queue_index: Arc<Mutex<usize>>,
    pub image_cache: ImageCache,
}

impl PlaybackState {
    pub fn new(image_cache: ImageCache) -> Self {
        Self {
            generation: Arc::new(AtomicU64::new(0)),
            active_shared: Arc::new(Mutex::new(None)),
            current_track: Arc::new(Mutex::new(None)),
            queue: Arc::new(Mutex::new(Vec::new())),
            queue_index: Arc::new(Mutex::new(0)),
            image_cache,
        }
    }

    /// The generation for a fresh track, applied to the audio status at once
    /// so events from the previous one are dropped.
    pub fn next_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn set_queue(&self, tracks: Vec<TrackItem>) {
        *self.queue.lock().unwrap() = tracks;
        *self.queue_index.lock().unwrap() = 0;
    }

    /// Point `queue_index` at the first track with this video id, if it's in
    /// the queue (a search row plays its slot, as the web's playList does).
    pub fn jump_to(&self, video_id: &str) {
        let queue = self.queue.lock().unwrap();
        if let Some(pos) = queue.iter().position(|t| t.video_id == video_id) {
            *self.queue_index.lock().unwrap() = pos;
        }
    }
}
