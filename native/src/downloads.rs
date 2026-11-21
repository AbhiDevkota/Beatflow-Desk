//! Downloads view wiring: listing offline audio files from disk,
//! reading metadata, playing local files without internet, and opening folder in Explorer.

use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, ModelRc};

use crate::app::App;
use crate::backend::downloads;
use crate::images;
use crate::playback;
use crate::state::TrackItem;

static DOWNLOADED_TRACKS: Mutex<Vec<TrackItem>> = Mutex::new(Vec::new());

fn format_duration(secs: Option<u32>) -> String {
    match secs {
        Some(s) => format!("{}:{:02}", s / 60, s % 60),
        None => "--:--".into(),
    }
}

struct PendingDownloadedTrack {
    index: i32,
    title: String,
    artist: String,
    duration: String,
    video_id: String,
    thumbnail: Option<Arc<images::RawImage>>,
}

pub fn wire(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let dl_data = app_window.global::<crate::DownloadsData>();

    // Initial setup for downloads storage directory
    let data_dir = std::env::temp_dir();
    downloads::init(&data_dir, "");

    // Play specific track by index
    {
        let app = app.clone();
        dl_data.on_play_track(move |idx| {
            let tracks = DOWNLOADED_TRACKS.lock().unwrap().clone();
            if let Some(_track) = tracks.get(idx as usize).cloned() {
                playback::play_queue_from(&app, tracks, idx as usize);
            }
        });
    }

    // Play all
    {
        let app = app.clone();
        dl_data.on_play_all(move || {
            let tracks = DOWNLOADED_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Shuffle all
    {
        let app = app.clone();
        dl_data.on_shuffle_all(move || {
            let mut tracks = DOWNLOADED_TRACKS.lock().unwrap().clone();
            if !tracks.is_empty() {
                // simple pseudo-shuffle
                let len = tracks.len();
                for i in 0..len {
                    let j = (i * 7 + 3) % len;
                    tracks.swap(i, j);
                }
                playback::play_queue_from(&app, tracks, 0);
            }
        });
    }

    // Open downloads folder in Windows Explorer
    dl_data.on_open_folder(move || {
        downloads::open_folder();
    });

    // Refresh downloads list
    {
        let app = app.clone();
        dl_data.on_refresh(move || {
            refresh_downloads(&app);
        });
    }

    // Load downloads initially
    refresh_downloads(app);
}

pub fn refresh_downloads(app: &App) {
    let weak = app.weak.clone();
    let http = app.http.clone();
    let cache = app.state.image_cache.clone();

    app.rt.spawn(async move {
        let downloaded = downloads::list();
        let count = downloaded.len() as i32;

        let mut track_items = Vec::new();
        let mut pending = Vec::new();

        for (idx, t) in downloaded.iter().enumerate() {
            let artist_str = t.artist_line();
            track_items.push(TrackItem {
                video_id: t.id.clone(),
                title: t.title.clone(),
                artist: artist_str.clone(),
                duration: 0.0,
            });

            // Fetch thumbnail if available
            let thumb = if let Some(first_thumb) = t.thumbs.first() {
                images::get_or_fetch_image(&http, &cache, &first_thumb.url).await
            } else {
                images::get_video_thumbnail(&http, &cache, &t.id).await
            };

            pending.push(PendingDownloadedTrack {
                index: (idx + 1) as i32,
                title: t.title.clone(),
                artist: artist_str,
                duration: format_duration(t.duration),
                video_id: t.id.clone(),
                thumbnail: thumb,
            });
        }

        *DOWNLOADED_TRACKS.lock().unwrap() = track_items;

        let total_size_mb = (count as f32 * 6.5).round();
        let total_size_str = format!("{:.0} MB", total_size_mb);

        let _ = slint::invoke_from_event_loop(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let dl_data = app_window.global::<crate::DownloadsData>();

            let ui_tracks: Vec<crate::CollectionTrack> = pending
                .into_iter()
                .map(|p| crate::CollectionTrack {
                    index: p.index,
                    title: p.title.into(),
                    artist: p.artist.into(),
                    duration: p.duration.into(),
                    video_id: p.video_id.into(),
                    thumbnail: p.thumbnail.as_ref().map(|r| r.to_slint()).unwrap_or_default(),
                    has_thumbnail: p.thumbnail.is_some(),
                    explicit: false,
                    plays: "".into(),
                })
                .collect();

            dl_data.set_tracks(ModelRc::new(slint::VecModel::from(ui_tracks)));
            dl_data.set_count(count);
            dl_data.set_total_size(total_size_str.into());
            dl_data.set_loading(false);
        });
    });
}
