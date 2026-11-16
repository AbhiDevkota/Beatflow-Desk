use super::model::{Card, Link, Thumb, Track};

pub struct Explore {
    pub new_albums: Vec<Card>,
    pub charts: Vec<Card>,
    pub top: Vec<Track>,
}

pub struct SearchResults {
    pub query: String,
    pub tracks: Vec<Track>,
    pub albums: Vec<Card>,
    pub artists: Vec<Card>,
    pub playlists: Vec<Card>,
}

/// An album or playlist page.
pub struct Collection {
    pub id: String,
    pub is_album: bool,
    /// Album's playlist ID for saving it to the account library.
    pub save_id: Option<String>,
    pub title: String,
    pub subtitle: String,
    /// Album artists / playlist owner named in the subtitle, clickable.
    pub artists: Vec<super::model::Link>,
    pub description: Option<String>,
    pub thumbs: Vec<Thumb>,
    pub tracks: Vec<Track>,
    /// Each track's playlist entry id (`setVideoId`), for removing it; empty for albums.
    pub set_ids: Vec<Option<String>>,
    /// How many songs the playlist's header counts (unplayable ones too, so it can be more
    /// than `tracks`); kept in step with removals and unlikes made here.
    pub count: Option<u64>,
    /// The account's own playlist (editable), not one saved from someone else.
    pub owned: bool,
}

impl Collection {
    /// The subtitle, with the song count for a playlist.
    pub fn byline(&self) -> String {
        match self.count {
            Some(n) if self.subtitle.is_empty() => format!("{n} songs"),
            Some(n) => format!("{} â€¢ {n} songs", self.subtitle),
            None => self.subtitle.clone(),
        }
    }

    /// Take rows out (sorted indices), and out of the count.
    pub fn remove_rows(&mut self, rows: &[usize]) {
        for &i in rows.iter().rev() {
            if i < self.tracks.len() {
                self.tracks.remove(i);
                if i < self.set_ids.len() {
                    self.set_ids.remove(i);
                }
                self.count = self.count.map(|n| n.saturating_sub(1));
            }
        }
    }
}

pub struct ArtistPage {
    pub id: String,
    pub name: String,
    pub banner: Vec<Thumb>,
    pub subscribers: Option<u64>,
    pub description: Option<String>,
    pub top: Vec<Track>,
    /// "Top songs" playlist behind the shelf's Show all button.
    pub top_playlist: Option<String>,
    pub albums: Vec<Card>,
    pub playlists: Vec<Card>,
    pub similar: Vec<Card>,
}

pub struct Library {
    pub playlists: Vec<Card>,
    pub albums: Vec<Card>,
    /// That half didn't load (YouTube Music refused it, say): the one shown stays.
    pub playlists_failed: bool,
    pub albums_failed: bool,
}

/// The signed-in account's channels.
pub struct Channels {
    pub list: Vec<super::ytm::Channel>,
    /// The page id of the one requests go as (`None`: the account's own).
    pub current: Option<String>,
    /// [`ytmapi::Api::epoch`] when listed: a switch picked from here applies only then.
    pub epoch: u64,
}
