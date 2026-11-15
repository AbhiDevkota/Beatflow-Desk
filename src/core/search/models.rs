//! Music discovery and search models.

use serde::{Deserialize, Serialize};
use crate::core::state::{AlbumRef, ArtistRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Song,
    Video,
    Album,
    Playlist,
    #[serde(rename = "spotify-playlist")]
    SpotifyPlaylist,
    Artist,
    Episode,
    Podcast,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShelfLayout {
    Tracks,
    Square,
    Video,
    Circle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicItem {
    pub kind: ItemKind,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub artists: Vec<ArtistRef>,
    pub album: Option<AlbumRef>,
    pub thumbnail: Option<String>,
    pub wide: bool,
    pub duration_sec: Option<u32>,
    pub explicit: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shelf {
    pub title: String,
    pub strapline: Option<String>,
    pub layout: ShelfLayout,
    pub items: Vec<MusicItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub top: Vec<MusicItem>,
    pub songs: Vec<MusicItem>,
    pub albums: Vec<MusicItem>,
    pub artists: Vec<MusicItem>,
    pub playlists: Vec<MusicItem>,
    pub videos: Vec<MusicItem>,
}
