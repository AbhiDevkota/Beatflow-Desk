//! The search page's wiring (was in main.rs): suggestions as the field is
//! typed, grouped results on submit (songs + the artists/albums/playlists/
//! videos shelves, in SearchPage.shelvesFor's order), and row playback.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use slint::{ComponentHandle, ModelRc};

use crate::app::App;
use crate::backend::ytm::{self, Filter};
use crate::data;
use crate::state::TrackItem;

pub fn wire(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let search = app_window.global::<crate::Search>();

    // ---- Suggestions (SearchField.tsx's useSuggestions) -------------------
    // Query completions while typing; a newer keystroke wins when two
    // responses race, and an empty field clears the list.
    {
        let app = app.clone();
        let generation = Arc::new(AtomicU64::new(0));
        search.on_suggest(move |text| {
            let text = text.to_string();
            let app = app.clone();
            let gen_flag = generation.clone();
            let my_gen = gen_flag.fetch_add(1, Ordering::SeqCst) + 1;

            if text.trim().is_empty() {
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app_window) = app.weak.upgrade() {
                        app_window
                            .global::<crate::Search>()
                            .set_suggestions(ModelRc::default());
                    }
                });
                return;
            }

            let rt = app.rt.clone();
            rt.spawn(async move {
                let _ = app.api.ensure_visitor().await;
                let Ok(suggs) = ytm::suggestions(&app.api, &text, false).await else {
                    return;
                };
                let items: Vec<crate::SuggestionItem> = suggs
                    .into_iter()
                    .take(8)
                    .map(|s| crate::SuggestionItem {
                        text: s.text.into(),
                    })
                    .collect();
                let weak = app.weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if gen_flag.load(Ordering::SeqCst) != my_gen {
                        return; // a newer keystroke has taken over
                    }
                    if let Some(app_window) = weak.upgrade() {
                        app_window
                            .global::<crate::Search>()
                            .set_suggestions(data::model(items));
                    }
                });
            });
        });
    }

    // ---- Submit: grouped results, as SearchPage.shelvesFor builds them ----
    {
        let app = app.clone();
        search.on_search(move |q| {
            let query = q.to_string();
            // The callback runs per submit, so it can't move fields out of
            // its captured `App` — clone the context into a fresh local that
            // the spawned task owns outright.
            let app = app.clone();
            let weak = app.weak.clone();
            let state = app.state.clone();
            let rt = app.rt.clone();
            rt.spawn(async move {
                let _ = app.api.ensure_visitor().await;
                let api = app.api.clone();
                let http = app.http.clone();
                let cache = app.state.image_cache.clone();

                // The five sources, one call each — joined so the page fills
                // in one update rather than five.
                let (songs, artists, albums, playlists, videos) = tokio::join!(
                    ytm::search_songs(&api, &query, false),
                    ytm::search_cards(&api, &query, Filter::Artists, false),
                    ytm::search_cards(&api, &query, Filter::Albums, false),
                    ytm::search_cards(&api, &query, Filter::Playlists, false),
                    ytm::search_videos(&api, &query, false),
                );

                // Songs: the first three are the Top Results cards (the
                // web's `results.top`); the rest shape the `tracks` shelf's
                // 4-row columns.
                let songs = songs.unwrap_or_default();
                let mut song_rows = Vec::new();
                for t in songs.iter().take(3) {
                    song_rows.push(data::track_to_row(&http, &cache, t).await);
                }
                let mut song_items = Vec::new();
                for t in songs.iter().take(24) {
                    song_items.push(data::track_to_item(&http, &cache, t).await);
                }

                // Videos: the 16:9 `video` shelf.
                let videos = videos.unwrap_or_default();
                let mut video_items = Vec::new();
                for t in videos.iter().take(10) {
                    video_items.push(data::track_to_item(&http, &cache, t).await);
                }

                // The card shelves.
                let mut artist_items = Vec::new();
                for c in artists.unwrap_or_default().iter().take(10) {
                    artist_items.push(data::card_to_item(&http, &cache, c).await);
                }
                let mut album_items = Vec::new();
                for c in albums.unwrap_or_default().iter().take(10) {
                    album_items.push(data::card_to_item(&http, &cache, c).await);
                }
                let mut playlist_items = Vec::new();
                for c in playlists.unwrap_or_default().iter().take(10) {
                    playlist_items.push(data::card_to_item(&http, &cache, c).await);
                }

                // The queue is the songs list, so a row plays its slot onward.
                let queue: Vec<TrackItem> = songs.iter().map(data::track_to_queue_item).collect();
                state.set_queue(queue);

                // The UI structs (and their `slint::Image`s) are built here,
                // on the UI thread.
                let _ = slint::invoke_from_event_loop(move || {
                    let Some(app_window) = weak.upgrade() else { return };
                    let search = app_window.global::<crate::Search>();
                    search.set_top_results(data::model(data::into_rows(song_rows)));
                    search.set_songs(data::PendingShelf {
                        title: "Songs".into(),
                        strapline: String::new(),
                        layout: 1,
                        cards: Vec::new(),
                        columns: data::pending_columns(song_items, 4),
                    }.into_shelf());
                    search.set_videos(data::PendingShelf {
                        title: "Videos".into(),
                        strapline: String::new(),
                        layout: 3,
                        cards: video_items,
                        columns: Vec::new(),
                    }.into_shelf());
                    search.set_artists(data::model(data::into_items(artist_items)));
                    search.set_albums(data::model(data::into_items(album_items)));
                    search.set_playlists(data::model(data::into_items(playlist_items)));
                });
            });
        });
    }

    // ---- Row playback ------------------------------------------------------
    {
        let app = app.clone();
        search.on_play(move |res| {
            let video_id = res.video_id.to_string();
            let title = res.title.to_string();
            let artist = res.artist.to_string();
            app.state.jump_to(&video_id);
            crate::playback::play_track(
                &app,
                video_id,
                title,
                artist,
                std::time::Duration::ZERO,
            );
        });
    }

    // ---- Shelf rows (artists/albums/playlists/songs/videos) ----------------
    // The queue becomes the clicked shelf from the item on (ShelfView's
    // playList); cards without a video id fall back to a title search.
    {
        let app = app.clone();
        search.on_play_shelf(move |shelf, item| {
            data::play_shelf(&app, &shelf, &item);
        });
    }
}
