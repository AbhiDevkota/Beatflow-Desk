//! Mapping YouTube Music's models onto the Slint structs the UI renders
//! (the mappers the page modules share).
//!
//! Thumbnails are fetched before a shelf model is built, so rows appear with
//! their art in one shot — the same order main.rs used before the split.
//!
//! The fetch runs on the tokio runtime, which can't hold Slint types
//! (`slint::Image` isn't `Send`), so the mappers hand back "pending" rows
//! whose art is still the cache's shared `RawImage`. `invoke_from_event_loop`
//! closures on the UI thread then turn them into the real structs via the
//! `into_*` conversions below.

use std::sync::Arc;

use slint::{Model, ModelRc, VecModel};

use crate::app::App;
use crate::backend::model::{self, Card, Target, Track};
use crate::images;
use crate::state::TrackItem;

/// Square art URL at the CDN size the UI draws (240px covers every slot the
/// shelves use), falling back to the video still.
pub fn thumb_url(thumbs: &[model::Thumb], video_id: &str) -> String {
    if let Some(u) = model::pick(thumbs, 240) {
        model::sized(u, 240)
    } else if !video_id.is_empty() {
        format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg")
    } else {
        String::new()
    }
}

// ------------------------------------------------------------ pending shapes

/// A `ShelfItem` whose art is still a cache-shared raw image, built off the
/// UI thread; `into_item` makes the UI struct once back on it.
pub struct PendingItem {
    pub title: String,
    pub subtitle: String,
    pub video_id: String,
    pub target_id: String,
    pub target_type: String,
    pub thumbnail: Option<Arc<images::RawImage>>,
    pub round: bool,
}

impl PendingItem {
    /// UI thread only: this is where the `slint::Image` is created.
    pub fn into_item(self) -> crate::ShelfItem {
        crate::ShelfItem {
            title: self.title.into(),
            subtitle: self.subtitle.into(),
            video_id: self.video_id.into(),
            thumbnail: self.thumbnail.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
            has_thumbnail: self.thumbnail.is_some(),
            round: self.round,
            target_id: self.target_id.into(),
            target_type: self.target_type.into(),
        }
    }
}

/// A batch of pending rows, converted on the UI thread.
pub fn into_items(items: Vec<PendingItem>) -> Vec<crate::ShelfItem> {
    items.into_iter().map(PendingItem::into_item).collect()
}

/// A `SearchResult` before the UI thread converts it (same reason).
pub struct PendingRow {
    pub title: String,
    pub artist: String,
    pub video_id: String,
    pub thumbnail: Option<Arc<images::RawImage>>,
}

impl PendingRow {
    /// UI thread only: this is where the `slint::Image` is created.
    pub fn into_row(self) -> crate::SearchResult {
        crate::SearchResult {
            title: self.title.into(),
            artist: self.artist.into(),
            video_id: self.video_id.into(),
            thumbnail: self.thumbnail.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
            has_thumbnail: self.thumbnail.is_some(),
        }
    }
}

/// A batch of pending rows, converted on the UI thread.
pub fn into_rows(rows: Vec<PendingRow>) -> Vec<crate::SearchResult> {
    rows.into_iter().map(PendingRow::into_row).collect()
}

/// A `Shelf` before the UI thread converts it: cards for the square/circle/
/// video layouts, and the `tracks` layout's row-sized chunks.
pub struct PendingShelf {
    pub title: String,
    pub strapline: String,
    pub layout: i32,
    pub cards: Vec<PendingItem>,
    pub columns: Vec<Vec<PendingItem>>,
}

impl PendingShelf {
    /// UI thread only: the models and their `slint::Image`s are built here.
    pub fn into_shelf(self) -> crate::Shelf {
        crate::Shelf {
            title: self.title.into(),
            strapline: self.strapline.into(),
            layout: self.layout,
            cards: if self.cards.is_empty() {
                ModelRc::default()
            } else {
                model(into_items(self.cards))
            },
            columns: if self.columns.is_empty() {
                ModelRc::default()
            } else {
                model(self.columns.into_iter().map(|chunk| crate::ShelfColumn {
                    items: model(into_items(chunk)),
                }).collect())
            },
        }
    }
}

// ------------------------------------------------------------------- mappers

/// Fetch a track's thumbnail and shape the shelf item.
pub async fn track_to_item(http: &reqwest::Client, cache: &images::ImageCache, t: &Track) -> PendingItem {
    let url = thumb_url(&t.thumbs, &t.id);
    PendingItem {
        title: t.title.clone(),
        subtitle: t.artist_line(),
        video_id: t.id.clone(),
        target_id: t.id.clone(),
        target_type: "song".to_string(),
        thumbnail: images::get_or_fetch_image(http, cache, &url).await,
        round: false,
    }
}

/// Fetch a card's thumbnail and shape the shelf item (albums, playlists,
/// artists — the last keeps `round` for the circular lockup).
pub async fn card_to_item(http: &reqwest::Client, cache: &images::ImageCache, c: &Card) -> PendingItem {
    let (video_id, target_id, target_type) = match &c.target {
        Target::Song(t) => (t.id.clone(), t.id.clone(), "song".to_string()),
        Target::Album(id) => (String::new(), id.clone(), "album".to_string()),
        Target::Playlist(id) => (String::new(), id.clone(), "playlist".to_string()),
        Target::Artist(id) => (String::new(), id.clone(), "artist".to_string()),
    };
    let url = thumb_url(&c.thumbs, &video_id);
    PendingItem {
        title: c.title.clone(),
        subtitle: c.subtitle.clone(),
        video_id,
        target_id,
        target_type,
        thumbnail: images::get_or_fetch_image(http, cache, &url).await,
        round: c.round,
    }
}

/// A search row (songs/videos): fetched thumbnail included.
pub async fn track_to_row(http: &reqwest::Client, cache: &images::ImageCache, t: &Track) -> PendingRow {
    let url = thumb_url(&t.thumbs, &t.id);
    PendingRow {
        title: t.title.clone(),
        artist: t.artist_line(),
        video_id: t.id.clone(),
        thumbnail: images::get_or_fetch_image(http, cache, &url).await,
    }
}

/// A plain track item for the playback queue.
pub fn track_to_queue_item(t: &Track) -> TrackItem {
    TrackItem {
        video_id: t.id.clone(),
        title: t.title.clone(),
        artist: t.artist_line(),
        duration: t.duration.unwrap_or(0) as f32,
    }
}

// ------------------------------------------------------------ tracks layout

/// Split items into the `tracks` shelf's fixed-height chunks (`rows` each,
/// TrackLockupsShelf's 4 rows), flowing horizontally.
fn chunks<T>(items: Vec<T>, rows: usize) -> Vec<Vec<T>> {
    let rows = rows.clamp(1, 8);
    let mut out = Vec::new();
    let mut chunk = Vec::new();
    for item in items {
        chunk.push(item);
        if chunk.len() == rows {
            out.push(std::mem::take(&mut chunk));
        }
    }
    if !chunk.is_empty() {
        out.push(chunk);
    }
    out
}

/// Lay song rows out the way the web's `tracks` shelf does — the UI-thread
/// version for the seeded defaults. Data fetched on the runtime uses
/// `pending_columns` instead and converts via `PendingShelf::into_shelf`.
pub fn tracks_to_columns(items: Vec<crate::ShelfItem>, rows: usize) -> Vec<crate::ShelfColumn> {
    chunks(items, rows).into_iter().map(|chunk| crate::ShelfColumn {
        items: model(chunk),
    }).collect()
}

/// The runtime's counterpart of `tracks_to_columns`: pending chunks that
/// `PendingShelf::into_shelf` turns into `ShelfColumn`s on the UI thread.
pub fn pending_columns(items: Vec<PendingItem>, rows: usize) -> Vec<Vec<PendingItem>> {
    chunks(items, rows)
}

// ------------------------------------------------------------------ playback

/// Every playable entry in a shelf, in render order (cards, then columns),
/// for "play the shelf from here" (playList's behaviour).
pub fn shelf_to_queue(shelf: &crate::Shelf) -> Vec<TrackItem> {
    let mut tracks = Vec::new();
    if shelf.layout == 1 {
        for col in shelf.columns.iter() {
            for item in col.items.iter() {
                if !item.video_id.is_empty() {
                    tracks.push(TrackItem {
                        video_id: item.video_id.to_string(),
                        title: item.title.to_string(),
                        artist: item.subtitle.to_string(),
                        duration: 0.0,
                    });
                }
            }
        }
    } else {
        for item in shelf.cards.iter() {
            if !item.video_id.is_empty() {
                tracks.push(TrackItem {
                    video_id: item.video_id.to_string(),
                    title: item.title.to_string(),
                    artist: item.subtitle.to_string(),
                    duration: 0.0,
                });
            }
        }
    }
    tracks
}

/// Play a shelf row: the queue becomes the shelf from the clicked item on,
/// as the web's `playList(tracks, i, …)` does. Items without a video id
/// (album/playlist cards) fall back to a title search.
pub fn play_shelf(app: &App, shelf: &crate::Shelf, item: &crate::ShelfItem) {
    if item.video_id.is_empty() {
        let app = app.clone();
        let title = item.title.to_string();
        let subtitle = item.subtitle.to_string();
        let rt = app.rt.clone();
        rt.spawn(async move {
            let query = format!("{title} {subtitle}");
            let _ = app.api.ensure_visitor().await;
            let Ok(tracks) = crate::backend::ytm::search_songs(&app.api, &query, false).await else {
                return;
            };
            let queue: Vec<TrackItem> = tracks.iter().map(track_to_queue_item).collect();
            app.state.set_queue(queue);
            let Some(first) = tracks.first() else { return };
            let first = TrackItem {
                video_id: first.id.clone(),
                title: first.title.clone(),
                artist: first.artist_line(),
                duration: first.duration.unwrap_or(0) as f32,
            };
            // Back to the UI thread: playback touches the window.
            let ui_app = app.clone();
            let _ = slint::invoke_from_event_loop(move || {
                crate::playback::play_one_off(&ui_app, first);
            });
        });
        return;
    }

    let queue = shelf_to_queue(shelf);
    app.state.set_queue(queue);
    crate::playback::play_one_off(
        app,
        TrackItem {
            video_id: item.video_id.to_string(),
            title: item.title.to_string(),
            artist: item.subtitle.to_string(),
            duration: 0.0,
        },
    );
}

// -------------------------------------------------------------------- models

/// Wrap items in a `VecModel` for a global's property.
pub fn model<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items))
}
