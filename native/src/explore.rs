//! The New (Explore) page's wiring (was in main.rs): new albums & singles,
//! the Top Charts song columns, and featured playlists.

use slint::{ComponentHandle, Model, ModelRc};

use crate::app::App;
use crate::backend::ytm;
use crate::data;
use crate::state::TrackItem;

const CARDS_PER_SHELF: usize = 12;
const COLUMN_LIMIT: usize = 6;

pub fn wire(app: &App) {
    seed_defaults(app);
    wire_callbacks(app);
    fetch_live(app);
}

// ------------------------------------------------------------------ defaults

fn seed_defaults(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");

    let shelf_item = |title: &str, subtitle: &str, video_id: &str| crate::ShelfItem {
        title: title.into(),
        subtitle: subtitle.into(),
        video_id: video_id.into(),
        thumbnail: Default::default(),
        has_thumbnail: false,
        round: false,
        target_id: video_id.into(),
        target_type: "song".into(),
    };

    let default_new_releases = vec![
        shelf_item("Espresso", "Sabrina Carpenter", "eVli-tstM5E"),
        shelf_item("BIRDS OF A FEATHER", "Billie Eilish", "d5gf9dXbPi0"),
        shelf_item("Please Please Please", "Sabrina Carpenter", "cF1Na4AIecM"),
        shelf_item("Good Luck, Babe!", "Chappell Roan", "1PIAA3KGIVA"),
        shelf_item("Fortnight", "Taylor Swift ft. Post Malone", "q3zKK4vzyEk"),
    ];
    let default_quick_picks = vec![
        shelf_item("As It Was", "Harry Styles", "H5v3kku4y6Q"),
        shelf_item("Stay", "The Kid LAROI, Justin Bieber", "kTJczUoc26U"),
        shelf_item("Heat Waves", "Glass Animals", "mRD0-GxqHVo"),
        shelf_item("Levitating", "Dua Lipa", "TUVcZfQe-Kw"),
    ];
    let default_top_picks = vec![
        shelf_item("Starboy", "The Weeknd", "34Na4j8AVgA"),
        shelf_item("Blinding Lights", "The Weeknd", "4NRXx6U8ABQ"),
        shelf_item("Save Your Tears", "The Weeknd", "XXYlFuWEuKI"),
        shelf_item("Die For You", "The Weeknd", "uPD0QOGChkM"),
        shelf_item("After Hours", "The Weeknd", "ygTZZpVHNmA"),
    ];

    let new_data = app_window.global::<crate::NewData>();
    new_data.set_new_albums(data::model(default_new_releases.clone()));
    new_data.set_top_charts(crate::Shelf {
        title: "Top Charts".into(),
        strapline: "".into(),
        layout: 1,
        cards: ModelRc::default(),
        columns: data::model(data::tracks_to_columns(default_quick_picks, 4)),
    });
    new_data.set_playlists(data::model(default_top_picks.clone()));

    // Background thumbnail fill for the defaults.
    let patch_app = app.clone();
    app.rt.spawn(async move {
        let vids = [
            "eVli-tstM5E", "d5gf9dXbPi0", "cF1Na4AIecM", "1PIAA3KGIVA", "q3zKK4vzyEk",
            "H5v3kku4y6Q", "kTJczUoc26U", "mRD0-GxqHVo", "TUVcZfQe-Kw",
            "34Na4j8AVgA", "4NRXx6U8ABQ", "XXYlFuWEuKI", "uPD0QOGChkM", "ygTZZpVHNmA",
        ];
        for vid in vids {
            let Some(raw) = crate::images::get_video_thumbnail(
                &patch_app.http,
                &patch_app.state.image_cache,
                vid,
            )
            .await
            else {
                continue;
            };
            let weak = patch_app.weak.clone();
            let vid = vid.to_string();
            let _ = slint::invoke_from_event_loop(move || {
                let Some(app_window) = weak.upgrade() else { return };
                let new_data = app_window.global::<crate::NewData>();
                patch_flat(new_data.get_new_albums(), &vid, &raw);
                patch_flat(new_data.get_playlists(), &vid, &raw);
                let charts = new_data.get_top_charts();
                let mut changed = false;
                for c in 0..charts.columns.row_count() {
                    if let Some(col) = charts.columns.row_data(c) {
                        for r in 0..col.items.row_count() {
                            if let Some(mut item) = col.items.row_data(r) {
                                if item.video_id == vid {
                                    item.thumbnail = raw.to_slint();
                                    item.has_thumbnail = true;
                                    col.items.set_row_data(r, item);
                                    changed = true;
                                }
                            }
                        }
                        charts.columns.set_row_data(c, col);
                    }
                }
                if changed {
                    new_data.set_top_charts(charts);
                }
            });
        }
    });
}

fn patch_flat(model: ModelRc<crate::ShelfItem>, vid: &str, raw: &crate::images::RawImage) {
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.video_id == vid {
                item.thumbnail = raw.to_slint();
                item.has_thumbnail = true;
                model.set_row_data(i, item);
            }
        }
    }
}

// ----------------------------------------------------------------- callbacks

fn wire_callbacks(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let new_data = app_window.global::<crate::NewData>();

    {
        let app = app.clone();
        new_data.on_play(move |item| {
            let title = item.title.to_string();
            let subtitle = item.subtitle.to_string();
            let video_id = item.video_id.to_string();
            if !video_id.is_empty() {
                crate::playback::play_one_off(
                    &app,
                    TrackItem { video_id, title, artist: subtitle, duration: 0.0 },
                );
            } else {
                play_first_search(&app, format!("{title} {subtitle}"));
            }
        });
    }

    {
        let app = app.clone();
        new_data.on_play_shelf(move |shelf, item| {
            data::play_shelf(&app, &shelf, &item);
        });
    }
}

fn play_first_search(app: &App, query: String) {
    let app = app.clone();
    let rt = app.rt.clone();
    rt.spawn(async move {
        let _ = app.api.ensure_visitor().await;
        let Ok(tracks) = ytm::search_songs(&app.api, &query, false).await else {
            return;
        };
        if let Some(first) = tracks.first() {
            let first = TrackItem {
                video_id: first.id.clone(),
                title: first.title.clone(),
                artist: first.artist_line(),
                duration: first.duration.unwrap_or(0) as f32,
            };
            let queue: Vec<TrackItem> = tracks.iter().map(data::track_to_queue_item).collect();
            app.state.set_queue(queue);
            let ui_app = app.clone();
            let _ = slint::invoke_from_event_loop(move || {
                crate::playback::play_one_off(&ui_app, first);
            });
        }
    });
}

// -------------------------------------------------------------- data loading

fn fetch_live(app: &App) {
    let app = app.clone();
    let rt = app.rt.clone();
    rt.spawn(async move {
        let _ = app.api.ensure_visitor().await;
        let http = app.http.clone();
        let cache = app.state.image_cache.clone();
        let api = app.api.clone();
        let weak = app.weak.clone();

        // New albums & singles
        if let Ok(cards) = ytm::new_albums(&api, false).await {
            let mut items = Vec::new();
            for c in cards.iter().take(CARDS_PER_SHELF) {
                items.push(data::card_to_item(&http, &cache, c).await);
            }
            if !items.is_empty() {
                // The UI structs are built inside the event-loop closure
                // (the runtime can't hold `slint::Image`s).
                let weak = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app_window) = weak.upgrade() {
                        app_window
                            .global::<crate::NewData>()
                            .set_new_albums(data::model(data::into_items(items)));
                    }
                });
            }
        }

        // Charts: Top Charts columns + featured playlists
        if let Ok(charts) = ytm::charts(&api).await {
            let mut chart_items = Vec::new();
            for t in charts.top.iter().take(COLUMN_LIMIT * 4) {
                chart_items.push(data::track_to_item(&http, &cache, t).await);
            }
            let mut playlist_items = Vec::new();
            for c in charts.playlists.iter().take(CARDS_PER_SHELF) {
                playlist_items.push(data::card_to_item(&http, &cache, c).await);
            }

            let weak = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app_window) = weak.upgrade() {
                    let new_data = app_window.global::<crate::NewData>();
                    if !chart_items.is_empty() {
                        new_data.set_top_charts(data::PendingShelf {
                            title: "Top Charts".into(),
                            strapline: String::new(),
                            layout: 1,
                            cards: Vec::new(),
                            columns: data::pending_columns(chart_items, 4),
                        }.into_shelf());
                    }
                    if !playlist_items.is_empty() {
                        new_data.set_playlists(data::model(data::into_items(playlist_items)));
                    }
                }
            });
        }
    });
}
