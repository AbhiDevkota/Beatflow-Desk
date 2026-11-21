//! Artist page wiring: fetching artist page from YouTube Music, loading hero banner,
//! top songs, discography albums and similar artists, and linking to collection pages.

use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, ModelRc};

use crate::app::App;
use crate::data;
use crate::images;
use crate::playback;
use crate::state::TrackItem;

struct PendingArtistTrack {
    index: i32,
    title: String,
    artist: String,
    duration: String,
    video_id: String,
    thumbnail: Option<Arc<images::RawImage>>,
    explicit: bool,
    plays: String,
}

static ACTIVE_ARTIST_TOP_TRACKS: Mutex<Vec<TrackItem>> = Mutex::new(Vec::new());

fn format_duration(secs: Option<u32>) -> String {
    match secs {
        Some(s) => format!("{}:{:02}", s / 60, s % 60),
        None => String::new(),
    }
}

pub fn wire(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let artist_data = app_window.global::<crate::ArtistData>();
    let ui = app_window.global::<crate::Ui>();

    // Open artist from anywhere
    {
        let app = app.clone();
        ui.on_open_artist(move |id| {
            open_artist(&app, id.to_string());
        });
    }

    // Play top song by index
    {
        let app = app.clone();
        artist_data.on_play_top(move |idx| {
            let tracks = ACTIVE_ARTIST_TOP_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                let track_idx = if idx > 0 { (idx - 1) as usize } else { 0 };
                playback::play_queue_from(&app, tracks, track_idx);
            }
        });
    }

    // Play all top songs
    {
        let app = app.clone();
        artist_data.on_play_all(move || {
            let tracks = ACTIVE_ARTIST_TOP_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Open an album card from discography
    {
        let app = app.clone();
        artist_data.on_open_album(move |album_id| {
            crate::collection::open_collection(&app, album_id.to_string(), true);
        });
    }

    // Open a similar artist card
    {
        let app = app.clone();
        artist_data.on_open_artist(move |art_id| {
            open_artist(&app, art_id.to_string());
        });
    }

    // Back to previous route
    {
        let weak = app.weak.clone();
        artist_data.on_back(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let ui = app_window.global::<crate::Ui>();
            let prev = ui.get_previous_route();
            ui.set_active_route(if prev.is_empty() { "Home".into() } else { prev });
        });
    }
}

pub fn open_artist(app: &App, id: String) {
    let Some(app_window) = app.weak.upgrade() else { return };
    let ui = app_window.global::<crate::Ui>();
    let prev = ui.get_active_route();
    if prev != "Artist" {
        ui.set_previous_route(prev);
    }
    ui.set_active_route("Artist".into());

    let artist_data = app_window.global::<crate::ArtistData>();
    artist_data.set_id(id.clone().into());
    artist_data.set_name("Loading...".into());
    artist_data.set_subscribers("".into());
    artist_data.set_description("".into());
    artist_data.set_has_banner(false);
    artist_data.set_has_logo(false);
    artist_data.set_top_songs(ModelRc::default());
    artist_data.set_albums(ModelRc::default());
    artist_data.set_singles(ModelRc::default());
    artist_data.set_similar(ModelRc::default());
    artist_data.set_loading(true);

    let app = app.clone();
    app.rt.spawn(async move {
        let _ = app.api.ensure_visitor().await;

        let Ok(artist) = crate::backend::ytm::artist(&app.api, &id).await else {
            log::warn!("Failed to load artist {id}");
            return;
        };

        // Banner photograph
        let banner_url = if let Some(t) = crate::backend::model::pick(&artist.banner, 1200) {
            crate::backend::model::sized(t, 1200)
        } else {
            String::new()
        };
        let raw_banner = if !banner_url.is_empty() {
            images::get_or_fetch_image(&app.http, &app.state.image_cache, &banner_url).await
        } else {
            None
        };

        // Top songs
        let mut queue_tracks = Vec::new();
        let mut pending_top_songs = Vec::new();

        for (i, t) in artist.top.iter().enumerate() {
            queue_tracks.push(TrackItem {
                video_id: t.id.clone(),
                title: t.title.clone(),
                artist: artist.name.clone(),
                duration: t.duration.unwrap_or(0) as f32,
            });

            let thumb_url = crate::data::thumb_url(&t.thumbs, &t.id);
            let raw_thumb = if !thumb_url.is_empty() {
                images::get_or_fetch_image(&app.http, &app.state.image_cache, &thumb_url).await
            } else {
                None
            };

            pending_top_songs.push(PendingArtistTrack {
                index: (i + 1) as i32,
                title: t.title.clone(),
                artist: artist.name.clone(),
                duration: format_duration(t.duration),
                video_id: t.id.clone(),
                thumbnail: raw_thumb,
                explicit: false,
                plays: t.plays.map(|p| format!("{p} plays")).unwrap_or_default(),
            });
        }

        *ACTIVE_ARTIST_TOP_TRACKS.lock().unwrap() = queue_tracks;

        // Albums
        let mut pending_albums = Vec::new();
        for c in &artist.albums {
            let item = data::card_to_item(&app.http, &app.state.image_cache, c).await;
            pending_albums.push(item);
        }

        // Similar Artists
        let mut pending_similar = Vec::new();
        for c in &artist.similar {
            let item = data::card_to_item(&app.http, &app.state.image_cache, c).await;
            pending_similar.push(item);
        }

        let name = artist.name.clone();
        let subscribers = artist.subscribers.map(|s| format!("{s} subscribers")).unwrap_or_default();
        let description = artist.description.clone().unwrap_or_default();

        let weak = app.weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let artist_data = app_window.global::<crate::ArtistData>();
            artist_data.set_name(name.into());
            artist_data.set_subscribers(subscribers.into());
            artist_data.set_description(description.into());
            if let Some(ref r) = raw_banner {
                artist_data.set_banner(r.to_slint());
                artist_data.set_has_banner(true);
            }
            let ui_top_songs: Vec<crate::CollectionTrack> = pending_top_songs.into_iter().map(|t| {
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
            artist_data.set_top_songs(data::model(ui_top_songs));
            artist_data.set_albums(data::model(data::into_items(pending_albums)));
            artist_data.set_similar(data::model(data::into_items(pending_similar)));
            artist_data.set_loading(false);
        });
    });
}
