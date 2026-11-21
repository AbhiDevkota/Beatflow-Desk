//! Album and Playlist page wiring: fetching real album/playlist collections from YouTube Music,
//! mapping tracks, downloading artwork with radiosity blur cache, and driving queue playback.

use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, ModelRc};

use crate::app::App;
use crate::data;
use crate::images;
use crate::playback;
use crate::state::TrackItem;

static ACTIVE_COLLECTION_TRACKS: Mutex<Vec<TrackItem>> = Mutex::new(Vec::new());

fn format_duration(secs: Option<u32>) -> String {
    match secs {
        Some(s) => format!("{}:{:02}", s / 60, s % 60),
        None => String::new(),
    }
}

struct PendingCollectionTrack {
    index: i32,
    title: String,
    artist: String,
    duration: String,
    video_id: String,
    thumbnail: Option<Arc<images::RawImage>>,
    explicit: bool,
    plays: String,
}

pub fn wire(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let coll_data = app_window.global::<crate::CollectionData>();
    let ui = app_window.global::<crate::Ui>();

    // Open collection from anywhere (Search / Home / Explore / Artist)
    {
        let app = app.clone();
        ui.on_open_collection(move |id, is_album| {
            open_collection(&app, id.to_string(), is_album);
        });
    }

    // Play specific track by index
    {
        let app = app.clone();
        coll_data.on_play_track(move |idx| {
            let tracks = ACTIVE_COLLECTION_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                let track_idx = if idx > 0 { (idx - 1) as usize } else { 0 };
                playback::play_queue_from(&app, tracks, track_idx);
            }
        });
    }

    // Play all from beginning
    {
        let app = app.clone();
        coll_data.on_play_all(move || {
            let tracks = ACTIVE_COLLECTION_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Shuffle all
    {
        let app = app.clone();
        coll_data.on_shuffle_all(move || {
            let mut tracks = ACTIVE_COLLECTION_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                // Fisher-Yates shuffle
                for i in (1..tracks.len()).rev() {
                    let j = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as usize) % (i + 1);
                    tracks.swap(i, j);
                }
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Back to previous route
    {
        let weak = app.weak.clone();
        coll_data.on_back(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let ui = app_window.global::<crate::Ui>();
            let prev = ui.get_previous_route();
            ui.set_active_route(if prev.is_empty() { "Home".into() } else { prev });
        });
    }
}

pub fn open_collection(app: &App, id: String, is_album: bool) {
    let Some(app_window) = app.weak.upgrade() else { return };
    let ui = app_window.global::<crate::Ui>();
    let prev = ui.get_active_route();
    if prev != "Album" && prev != "Playlist" {
        ui.set_previous_route(prev);
    }
    ui.set_active_route(if is_album { "Album".into() } else { "Playlist".into() });

    let coll_data = app_window.global::<crate::CollectionData>();
    coll_data.set_id(id.clone().into());
    coll_data.set_title("Loading...".into());
    coll_data.set_subtitle("".into());
    coll_data.set_byline("".into());
    coll_data.set_description("".into());
    coll_data.set_has_thumbnail(false);
    coll_data.set_is_album(is_album);
    coll_data.set_tracks(ModelRc::default());
    coll_data.set_more(ModelRc::default());
    coll_data.set_loading(true);

    let app = app.clone();
    app.rt.spawn(async move {
        let _ = app.api.ensure_visitor().await;

        let result = if is_album {
            crate::backend::ytm::album(&app.api, &id).await
        } else {
            crate::backend::ytm::playlist(&app.api, &id, false, Some(100)).await
        };

        let Ok(collection) = result else {
            log::warn!("Failed to load collection {id}");
            return;
        };

        // Cache-shared thumbnail fetch
        let thumb_url = if let Some(t) = crate::backend::model::pick(&collection.thumbs, 544) {
            crate::backend::model::sized(t, 544)
        } else {
            String::new()
        };
        let raw_thumb = images::get_or_fetch_image(&app.http, &app.state.image_cache, &thumb_url).await;

        // Shape tracks
        let mut queue_tracks = Vec::new();
        let mut pending_tracks = Vec::new();

        for (i, t) in collection.tracks.iter().enumerate() {
            let duration_str = format_duration(t.duration);
            let artist_str = t.artist_line();

            queue_tracks.push(TrackItem {
                video_id: t.id.clone(),
                title: t.title.clone(),
                artist: if artist_str.is_empty() { collection.subtitle.clone() } else { artist_str.clone() },
                duration: t.duration.unwrap_or(0) as f32,
            });

            // Small thumb for playlists
            let track_thumb_url = if !is_album {
                crate::data::thumb_url(&t.thumbs, &t.id)
            } else {
                String::new()
            };
            let track_raw = if !track_thumb_url.is_empty() {
                images::get_or_fetch_image(&app.http, &app.state.image_cache, &track_thumb_url).await
            } else {
                None
            };

            pending_tracks.push(PendingCollectionTrack {
                index: (i + 1) as i32,
                title: t.title.clone(),
                artist: artist_str,
                duration: duration_str,
                video_id: t.id.clone(),
                thumbnail: track_raw,
                explicit: false,
                plays: t.plays.map(|p| format!("{p} plays")).unwrap_or_default(),
            });
        }

        *ACTIVE_COLLECTION_TRACKS.lock().unwrap() = queue_tracks;

        let title = collection.title.clone();
        let subtitle = if is_album {
            collection.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
        } else {
            collection.subtitle.clone()
        };
        let byline = collection.byline();
        let description = collection.description.clone().unwrap_or_default();

        let weak = app.weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let coll_data = app_window.global::<crate::CollectionData>();
            coll_data.set_title(title.into());
            coll_data.set_subtitle(subtitle.into());
            coll_data.set_byline(byline.into());
            coll_data.set_description(description.into());
            if let Some(ref r) = raw_thumb {
                coll_data.set_thumbnail(r.to_slint());
                coll_data.set_has_thumbnail(true);
            }
            let ui_tracks: Vec<crate::CollectionTrack> = pending_tracks.into_iter().map(|t| {
                crate::CollectionTrack {
                    index: t.index,
                    title: t.title.into(),
                    artist: t.artist.into(),
                    duration: t.duration.into(),
                    video_id: t.video_id.into(),
                    thumbnail: t.thumbnail.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
                    has_thumbnail: t.thumbnail.is_some(),
                    explicit: t.explicit,
                    plays: t.plays.into(),
                }
            }).collect();
            coll_data.set_tracks(data::model(ui_tracks));
            coll_data.set_loading(false);
        });
    });
}
