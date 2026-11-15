//! Typed application domain models and playback state.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtistRef {
    pub name: String,
    pub id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumRef {
    pub name: String,
    pub id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub video_id: String,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub album: Option<AlbumRef>,
    pub duration_sec: Option<u32>,
    pub explicit: bool,
    pub thumbnail: Option<String>,
    pub index: Option<u32>,
    pub plays: Option<String>,
    pub is_video: Option<bool>,
    pub source_quality: Option<String>,
    pub source_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub video_id: String,
    pub canonical_video_id: Option<String>,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub duration_sec: u32,
    pub stream_info: Option<String>,
    pub source_quality: Option<String>,
    pub source_name: Option<String>,
    pub upgraded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlayerStatus {
    #[default]
    Idle,
    Loading,
    Playing,
    Paused,
    Buffering,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueSource {
    pub title: String,
    pub source_type: String,
    pub id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueEntry {
    pub track: Track,
    pub source: QueueSource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub autoplay: bool,
    pub volume: f32,
    pub muted: bool,
    pub crossfade_sec: u32,
    pub automix: bool,
    pub automix_bars: u32,
    pub show_nerd_stats: bool,
    pub prefer_atmos: bool,
    pub auto_upgrade: bool,
    pub theme: String, // "system", "dark", "light"
    pub font: String,  // "sf-pro", "dm-sans", "system"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shuffle: false,
            repeat: RepeatMode::Off,
            autoplay: true,
            volume: 1.0,
            muted: false,
            crossfade_sec: 0,
            automix: false,
            automix_bars: 16,
            show_nerd_stats: false,
            prefer_atmos: true,
            auto_upgrade: true,
            theme: "dark".to_string(),
            font: "sf-pro".to_string(),
        }
    }
}
