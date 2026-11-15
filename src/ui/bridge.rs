//! Thread-safe async bridge between background Tokio engine tasks and Slint UI event loop.

use slint::{Model, ModelRc, SharedString, VecModel};
use super::{
    AppWindow, HomeTrackData, LyricLineData, QueueItemData, ShelfCardData,
};

#[derive(Clone)]
pub struct UiBridge {
    app_weak: slint::Weak<AppWindow>,
}

impl UiBridge {
    pub fn new(app_weak: slint::Weak<AppWindow>) -> Self {
        Self { app_weak }
    }

    /// Helper to format milliseconds to m:ss or -m:ss
    pub fn format_duration(ms: u64) -> String {
        let total_secs = ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}:{:02}", mins, secs)
    }

    /// Helper to format remaining time as -m:ss
    pub fn format_remaining(elapsed_ms: u64, total_ms: u64) -> String {
        if total_ms > elapsed_ms {
            let rem_ms = total_ms - elapsed_ms;
            format!("-{}", Self::format_duration(rem_ms))
        } else {
            "-0:00".to_string()
        }
    }

    /// Update playback state: playing/paused, title, artist, album, quality badge
    pub fn update_playback(
        &self,
        is_playing: bool,
        track_id: &str,
        title: &str,
        artist: &str,
        album: &str,
        quality: &str,
    ) {
        let track_id = SharedString::from(track_id);
        let title = SharedString::from(title);
        let artist = SharedString::from(artist);
        let album = SharedString::from(album);
        let quality = SharedString::from(quality);

        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            app.set_is_playing(is_playing);
            app.set_active_track_id(track_id);
            app.set_track_title(title);
            app.set_artist_name(artist);
            app.set_album_title(album);
            app.set_quality_badge(quality);
        });
    }

    /// Update progress bar and time labels
    pub fn update_progress(&self, position_ms: u64, duration_ms: u64) {
        let elapsed = SharedString::from(Self::format_duration(position_ms));
        let remaining = SharedString::from(Self::format_remaining(position_ms, duration_ms));
        let ratio = if duration_ms > 0 {
            (position_ms as f32 / duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            app.set_progress_ratio(ratio);
            app.set_time_elapsed(elapsed);
            app.set_time_remaining(remaining);
        });
    }

    /// Update shuffle and repeat states
    pub fn update_transport_modes(&self, is_shuffle: bool, repeat_mode: i32) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            app.set_is_shuffle(is_shuffle);
            app.set_repeat_mode(repeat_mode);
        });
    }

    /// Update favorite status
    pub fn update_favorite(&self, is_favorite: bool) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            app.set_is_favorite(is_favorite);
        });
    }

    /// Update home recommendations & recent tracks
    pub fn set_home_data(&self, picks: Vec<ShelfCardData>, recent: Vec<HomeTrackData>) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            let picks_model = ModelRc::new(VecModel::from(picks));
            let recent_model = ModelRc::new(VecModel::from(recent));
            app.set_home_top_picks(picks_model);
            app.set_home_recent_tracks(recent_model);
        });
    }

    /// Update search suggestions
    pub fn set_search_suggestions(&self, suggestions: Vec<String>) {
        let slint_strings: Vec<SharedString> = suggestions.into_iter().map(SharedString::from).collect();

        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            let model = ModelRc::new(VecModel::from(slint_strings));
            app.set_search_suggestions(model);
        });
    }

    /// Update search results list
    pub fn set_search_results(&self, results: Vec<HomeTrackData>) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            let model = ModelRc::new(VecModel::from(results));
            app.set_search_results(model);
        });
    }

    /// Update Up Next queue list
    pub fn set_queue_items(&self, queue: Vec<QueueItemData>) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            let model = ModelRc::new(VecModel::from(queue));
            app.set_queue_items(model);
        });
    }

    /// Update lyrics lines
    pub fn set_lyrics_lines(&self, lines: Vec<LyricLineData>) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            let model = ModelRc::new(VecModel::from(lines));
            app.set_lyrics_lines(model);
        });
    }

    /// Highlight active lyric line based on current playback timestamp
    pub fn sync_lyrics_timestamp(&self, pos_ms: u64) {
        let _ = self.app_weak.upgrade_in_event_loop(move |app| {
            let model = app.get_lyrics_lines();
            let count = model.row_count();
            if count == 0 {
                return;
            }

            // Find line closest to pos_ms without exceeding
            let mut active_idx = 0;
            for i in 0..count {
                if let Some(row) = model.row_data(i) {
                    if (row.time_ms as u64) <= pos_ms {
                        active_idx = i;
                    } else {
                        break;
                    }
                }
            }

            // Update row active flags if needed
            for i in 0..count {
                if let Some(mut row) = model.row_data(i) {
                    let should_be_active = i == active_idx;
                    if row.is_active != should_be_active {
                        row.is_active = should_be_active;
                        model.set_row_data(i, row);
                    }
                }
            }
        });
    }
}
