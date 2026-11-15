//! Playback queue timeline management, shuffling, and navigation.

use rand::seq::SliceRandom;
use crate::core::state::{QueueEntry, QueueSource, RepeatMode, Track};

pub const BACK_RESTARTS_AFTER_SEC: u64 = 3;

#[derive(Clone, Debug, Default)]
pub struct QueueTimeline {
    pub entries: Vec<QueueEntry>,
    pub index: isize,
    pub original_order: Vec<QueueEntry>,
    pub is_shuffled: bool,
    pub repeat_mode: RepeatMode,
}

impl QueueTimeline {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: -1,
            original_order: Vec::new(),
            is_shuffled: false,
            repeat_mode: RepeatMode::Off,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.original_order.clear();
        self.index = -1;
    }

    pub fn is_shuffle(&self) -> bool {
        self.is_shuffled
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        if self.is_shuffled != shuffle {
            self.toggle_shuffle();
        }
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn current_track(&self) -> Option<&Track> {
        if self.index >= 0 && (self.index as usize) < self.entries.len() {
            Some(&self.entries[self.index as usize].track)
        } else {
            None
        }
    }

    pub fn current_entry(&self) -> Option<&QueueEntry> {
        if self.index >= 0 && (self.index as usize) < self.entries.len() {
            Some(&self.entries[self.index as usize])
        } else {
            None
        }
    }

    /// Plays a collection of tracks (an album or playlist) starting at `start_index`.
    pub fn play_tracks(&mut self, tracks: Vec<Track>, start_index: usize, source: QueueSource) {
        let entries: Vec<QueueEntry> = tracks
            .into_iter()
            .map(|track| QueueEntry {
                track,
                source: source.clone(),
            })
            .collect();

        self.original_order = entries.clone();
        self.entries = entries;
        self.index = if start_index < self.entries.len() {
            start_index as isize
        } else {
            0
        };
        self.is_shuffled = false;
    }

    /// Plays a single track on its own.
    pub fn play_track(&mut self, track: Track, source: QueueSource) {
        let entry = QueueEntry { track, source };
        self.original_order = vec![entry.clone()];
        self.entries = vec![entry];
        self.index = 0;
        self.is_shuffled = false;
    }

    /// Inserts tracks immediately after the current playing track ("Play Next").
    pub fn play_next(&mut self, tracks: Vec<Track>, source: QueueSource) {
        let new_entries: Vec<QueueEntry> = tracks
            .into_iter()
            .map(|t| QueueEntry {
                track: t,
                source: source.clone(),
            })
            .collect();

        let insert_pos = if self.index >= 0 {
            (self.index as usize) + 1
        } else {
            0
        };

        for (i, entry) in new_entries.into_iter().enumerate() {
            if insert_pos + i <= self.entries.len() {
                self.entries.insert(insert_pos + i, entry);
            } else {
                self.entries.push(entry);
            }
        }
    }

    /// Appends tracks to the end of the upcoming queue ("Add to Queue").
    pub fn add_to_queue(&mut self, tracks: Vec<Track>, source: QueueSource) {
        let new_entries: Vec<QueueEntry> = tracks
            .into_iter()
            .map(|t| QueueEntry {
                track: t,
                source: source.clone(),
            })
            .collect();

        self.entries.extend(new_entries);
    }

    /// Advances to the next track respecting RepeatMode.
    pub fn next(&mut self, repeat: RepeatMode) -> Option<&Track> {
        if self.entries.is_empty() {
            return None;
        }

        if repeat == RepeatMode::One {
            return self.current_track();
        }

        let next_idx = self.index + 1;
        if (next_idx as usize) < self.entries.len() {
            self.index = next_idx;
            self.current_track()
        } else if repeat == RepeatMode::All {
            self.index = 0;
            self.current_track()
        } else {
            None
        }
    }

    /// Navigates to the previous track or restarts the current track if beyond 3 seconds.
    pub fn previous(&mut self, current_position_sec: u64) -> Option<&Track> {
        if self.entries.is_empty() {
            return None;
        }

        if current_position_sec >= BACK_RESTARTS_AFTER_SEC || self.index <= 0 {
            self.current_track()
        } else {
            self.index -= 1;
            self.current_track()
        }
    }

    /// Jumps directly to a specific track index.
    pub fn jump_to(&mut self, target_idx: usize) -> Option<&Track> {
        if target_idx < self.entries.len() {
            self.index = target_idx as isize;
            self.current_track()
        } else {
            None
        }
    }

    /// Removes an entry at the given index.
    pub fn remove_at(&mut self, target_idx: usize) {
        if target_idx < self.entries.len() {
            self.entries.remove(target_idx);
            if (self.index as usize) >= target_idx && self.index > 0 {
                self.index -= 1;
            }
        }
    }

    /// Moves an entry from one position to another in the upcoming queue.
    pub fn move_entry(&mut self, from: usize, to: usize) {
        if from < self.entries.len() && to < self.entries.len() && from != to {
            let entry = self.entries.remove(from);
            self.entries.insert(to, entry);
        }
    }

    /// Clears upcoming hand-queued tracks.
    pub fn clear_up_next(&mut self) {
        if self.index >= 0 && (self.index as usize) + 1 < self.entries.len() {
            self.entries.truncate((self.index as usize) + 1);
        }
    }

    /// Toggles shuffle mode.
    pub fn toggle_shuffle(&mut self) {
        if self.entries.len() <= 1 {
            self.is_shuffled = !self.is_shuffled;
            return;
        }

        if !self.is_shuffled {
            // Enable shuffle
            let current_entry = self.current_entry().cloned();
            let mut remaining = self.entries.clone();
            if self.index >= 0 && (self.index as usize) < remaining.len() {
                remaining.remove(self.index as usize);
            }

            let mut rng = rand::rng();
            remaining.shuffle(&mut rng);

            let mut shuffled = Vec::with_capacity(remaining.len() + 1);
            if let Some(curr) = current_entry {
                shuffled.push(curr);
                self.index = 0;
            }
            shuffled.extend(remaining);

            self.entries = shuffled;
            self.is_shuffled = true;
        } else {
            // Restore linear order
            let current_track_id = self.current_track().map(|t| t.video_id.clone());
            self.entries = self.original_order.clone();
            self.is_shuffled = false;

            if let Some(id) = current_track_id
                && let Some(pos) = self.entries.iter().position(|e| e.track.video_id == id) {
                    self.index = pos as isize;
                }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_track(id: &str, title: &str) -> Track {
        Track {
            video_id: id.to_string(),
            title: title.to_string(),
            artists: vec![],
            album: None,
            duration_sec: Some(180),
            explicit: false,
            thumbnail: None,
            index: None,
            plays: None,
            is_video: None,
            source_quality: None,
            source_name: None,
        }
    }

    #[test]
    fn test_queue_navigation() {
        let mut queue = QueueTimeline::new();
        let tracks = vec![
            test_track("1", "Song 1"),
            test_track("2", "Song 2"),
            test_track("3", "Song 3"),
        ];
        let source = QueueSource {
            title: "Test".to_string(),
            source_type: "album".to_string(),
            id: None,
        };

        queue.play_tracks(tracks, 0, source);
        assert_eq!(queue.current_track().unwrap().title, "Song 1");

        let next = queue.next(RepeatMode::Off).unwrap();
        assert_eq!(next.title, "Song 2");

        let prev = queue.previous(1).unwrap();
        assert_eq!(prev.title, "Song 1");
    }

    #[test]
    fn test_repeat_one() {
        let mut queue = QueueTimeline::new();
        let tracks = vec![test_track("1", "Song 1"), test_track("2", "Song 2")];
        let source = QueueSource {
            title: "Test".to_string(),
            source_type: "album".to_string(),
            id: None,
        };
        queue.play_tracks(tracks, 0, source);

        let next = queue.next(RepeatMode::One).unwrap();
        assert_eq!(next.title, "Song 1");
    }
}
