//! Local library persistence: favorites, user playlists, and playback history.

use std::collections::HashSet;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::core::state::Track;
use crate::platform::paths::get_library_path;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPlaylist {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at_ms: u64,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub track: Track,
    pub played_at_ms: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryData {
    pub liked_video_ids: HashSet<String>,
    pub playlists: Vec<UserPlaylist>,
    pub history: Vec<HistoryEntry>,
}

pub struct LibraryStore {
    data: parking_lot::RwLock<LibraryData>,
}

impl Default for LibraryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryStore {
    pub fn new() -> Self {
        let path = get_library_path();
        let data = if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                serde_json::from_str::<LibraryData>(&content).unwrap_or_default()
            } else {
                LibraryData::default()
            }
        } else {
            LibraryData::default()
        };

        Self {
            data: parking_lot::RwLock::new(data),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = get_library_path();
        let data = self.data.read();
        let content = serde_json::to_string_pretty(&*data)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn is_liked(&self, video_id: &str) -> bool {
        self.data.read().liked_video_ids.contains(video_id)
    }

    pub fn toggle_like(&self, video_id: &str) -> bool {
        let mut data = self.data.write();
        let liked = if data.liked_video_ids.contains(video_id) {
            data.liked_video_ids.remove(video_id);
            false
        } else {
            data.liked_video_ids.insert(video_id.to_string());
            true
        };
        drop(data);
        let _ = self.save();
        liked
    }

    pub fn record_history(&self, track: Track) {
        let mut data = self.data.write();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Deduplicate recent if same track played consecutively
        data.history.retain(|h| h.track.video_id != track.video_id);
        data.history.insert(0, HistoryEntry {
            track,
            played_at_ms: now,
        });

        if data.history.len() > 500 {
            data.history.truncate(500);
        }
        drop(data);
        let _ = self.save();
    }

    pub fn get_history(&self) -> Vec<HistoryEntry> {
        self.data.read().history.clone()
    }

    pub fn get_playlists(&self) -> Vec<UserPlaylist> {
        self.data.read().playlists.clone()
    }

    pub fn create_playlist(&self, title: &str, description: Option<&str>) -> UserPlaylist {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let id = format!("playlist_{}", now);
        let pl = UserPlaylist {
            id,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            created_at_ms: now,
            tracks: Vec::new(),
        };

        let mut data = self.data.write();
        data.playlists.push(pl.clone());
        drop(data);
        let _ = self.save();
        pl
    }
}
