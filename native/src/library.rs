//! Library view wiring: fetching user playlists, albums, and liked songs from YouTube Music,
//! mapping cards and track lists into Slint models.

use std::sync::Mutex;
use slint::{ComponentHandle, ModelRc};

use crate::app::App;
use crate::backend::ytm;
use crate::data;
use crate::images;
use crate::playback;
use crate::state::TrackItem;

static LIBRARY_SONGS: Mutex<Vec<TrackItem>> = Mutex::new(Vec::new());

fn format_duration(secs: Option<u32>) -> String {
    match secs {
        Some(s) => format!("{}:{:02}", s / 60, s % 60),
        None => "--:--".into(),
    }
}

pub fn wire(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let lib_data = app_window.global::<crate::LibraryData>();

    // Tab switching
    {
        let weak = app.weak.clone();
        lib_data.on_set_tab(move |tab| {
            if let Some(app) = weak.upgrade() {
                app.global::<crate::LibraryData>().set_active_tab(tab);
            }
        });
    }

    // Play specific song by index
    {
        let app = app.clone();
        lib_data.on_play_song(move |idx| {
            let tracks = LIBRARY_SONGS.lock().unwrap().clone();
            if let Some(_track) = tracks.get(idx as usize).cloned() {
                playback::play_queue_from(&app, tracks, idx as usize);
            }
        });
    }

    // Play all
    {
        let app = app.clone();
        lib_data.on_play_all(move || {
            let tracks = LIBRARY_SONGS.lock().unwrap().clone();
            if !tracks.is_empty() {
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Shuffle all
    {
        let app = app.clone();
        lib_data.on_shuffle_all(move || {
            let mut tracks = LIBRARY_SONGS.lock().unwrap().clone();
            if !tracks.is_empty() {
                let len = tracks.len();
                for i in 0..len {
                    let j = (i * 7 + 3) % len;
                    tracks.swap(i, j);
                }
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Refresh
    {
        let app = app.clone();
        lib_data.on_refresh(move || {
            load_library(&app);
        });
    }

    // Load initial library data
    load_library(app);
}

pub fn load_library(app: &App) {
    let weak = app.weak.clone();
    let api = app.api.clone();
    let http = app.http.clone();
    let cache = app.state.image_cache.clone();

    app.rt.spawn(async move {
        // Try to fetch saved playlists and albums
        let playlists_res = ytm::saved_playlists(&api).await;
        let albums_res = ytm::saved_albums(&api).await;

        let mut playlist_items = Vec::new();
        if let Ok(playlists) = playlists_res {
            for c in playlists.iter().take(20) {
                playlist_items.push(data::card_to_item(&http, &cache, c).await);
            }
        }

        let mut album_items = Vec::new();
        if let Ok(albums) = albums_res {
            for c in albums.iter().take(20) {
                album_items.push(data::card_to_item(&http, &cache, c).await);
            }
        }

        // Also load local offline tracks as initial library songs
        let local_tracks = crate::backend::downloads::list();
        let mut song_items = Vec::new();
        let mut pending_songs = Vec::new();

        for (idx, t) in local_tracks.iter().enumerate() {
            let artist_str = t.artist_line();
            song_items.push(TrackItem {
                video_id: t.id.clone(),
                title: t.title.clone(),
                artist: artist_str.clone(),
                duration: 0.0,
            });

            let thumb = if let Some(first_thumb) = t.thumbs.first() {
                images::get_or_fetch_image(&http, &cache, &first_thumb.url).await
            } else {
                images::get_video_thumbnail(&http, &cache, &t.id).await
            };

            pending_songs.push((
                idx as i32 + 1,
                t.title.clone(),
                artist_str,
                format_duration(t.duration),
                t.id.clone(),
                thumb,
            ));
        }

        *LIBRARY_SONGS.lock().unwrap() = song_items;

        let _ = slint::invoke_from_event_loop(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let lib_data = app_window.global::<crate::LibraryData>();

            let ui_playlists: Vec<crate::ShelfItem> = playlist_items
                .into_iter()
                .map(|p| crate::ShelfItem {
                    title: p.title.into(),
                    subtitle: p.subtitle.into(),
                    video_id: p.video_id.into(),
                    thumbnail: p.thumbnail.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
                    has_thumbnail: p.thumbnail.is_some(),
                    round: false,
                    target_id: p.target_id.into(),
                    target_type: p.target_type.into(),
                })
                .collect();

            let ui_albums: Vec<crate::ShelfItem> = album_items
                .into_iter()
                .map(|p| crate::ShelfItem {
                    title: p.title.into(),
                    subtitle: p.subtitle.into(),
                    video_id: p.video_id.into(),
                    thumbnail: p.thumbnail.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
                    has_thumbnail: p.thumbnail.is_some(),
                    round: false,
                    target_id: p.target_id.into(),
                    target_type: p.target_type.into(),
                })
                .collect();

            let ui_songs: Vec<crate::CollectionTrack> = pending_songs
                .into_iter()
                .map(|(idx, title, artist, duration, video_id, thumb)| crate::CollectionTrack {
                    index: idx,
                    title: title.into(),
                    artist: artist.into(),
                    duration: duration.into(),
                    video_id: video_id.into(),
                    thumbnail: thumb.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
                    has_thumbnail: thumb.is_some(),
                    explicit: false,
                    plays: "".into(),
                })
                .collect();

            lib_data.set_playlists(ModelRc::new(slint::VecModel::from(ui_playlists)));
            lib_data.set_albums(ModelRc::new(slint::VecModel::from(ui_albums)));
            lib_data.set_songs(ModelRc::new(slint::VecModel::from(ui_songs)));
            lib_data.set_loading(false);
        });
    });
}
