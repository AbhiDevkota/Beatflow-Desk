//! The Home page's wiring (was in main.rs): the default shelves shown while
//! the feed loads, Top Picks, Recently Played (the signed-in history), the
//! dynamic shelves, the mood tiles, and the play callbacks.

use slint::{ComponentHandle, Model, ModelRc};

use crate::app::App;
use crate::backend::model::ShelfKind;
use crate::backend::ytm;
use crate::data;
use crate::state::TrackItem;

const RECENT_LIMIT: usize = 20;
/// Per-shelf take counts, matching what the web shelves show.
const CARDS_PER_SHELF: usize = 12;
const COLUMN_LIMIT: usize = 6; // columns of 4 rows → up to 24 rows

pub fn wire(app: &App) {
    seed_defaults(app);
    wire_callbacks(app);
    fetch_recently_played(app);
    fetch_live(app);
}

// ------------------------------------------------------------------ defaults

/// The shelves shown while `ytm::home` loads, roughly what the feed answers
/// with (top picks are hard-coded ids; the thumbnail patcher fills their art).
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

    let default_top_picks = vec![
        shelf_item("Starboy", "The Weeknd", "34Na4j8AVgA"),
        shelf_item("Blinding Lights", "The Weeknd", "4NRXx6U8ABQ"),
        shelf_item("Save Your Tears", "The Weeknd", "XXYlFuWEuKI"),
        shelf_item("Die For You", "The Weeknd", "uPD0QOGChkM"),
        shelf_item("After Hours", "The Weeknd", "ygTZZpVHNmA"),
    ];
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

    let default_moods = vec![
        mood("Energy", 0x1e, 0xbd, 0x38, 0xe2, 0xf6, 0xa2),
        mood("Relax", 0x31, 0xaf, 0xf3, 0xb4, 0xe9, 0xff),
        mood("Feeling Blue", 0x25, 0x3a, 0xb2, 0x6c, 0xc4, 0xff),
        mood("Feel Good", 0xf2, 0xa9, 0x00, 0xff, 0xe8, 0xa0),
        mood("Focus", 0xf2, 0x84, 0x00, 0xff, 0xd6, 0xa4),
        mood("Heartbreak", 0xea, 0x35, 0x20, 0xff, 0xa6, 0x90),
        mood("Love", 0xf3, 0x4c, 0xa0, 0xff, 0xb8, 0xdc),
    ];

    let home = app_window.global::<crate::HomeData>();
    home.set_top_picks(data::model(default_top_picks.clone()));
    home.set_has_recently_played(false);
    home.set_recently_played(crate::Shelf {
        title: "Recently Played".into(),
        strapline: "".into(),
        layout: 1,
        cards: ModelRc::default(),
        columns: ModelRc::default(),
    });
    home.set_moods(data::model(default_moods));
    home.set_has_lead_shelf(true);
    home.set_lead_shelf(crate::Shelf {
        title: "New Releases".into(),
        strapline: "".into(),
        layout: 0,
        cards: data::model(default_new_releases.clone()),
        columns: ModelRc::default(),
    });
    home.set_shelves(data::model(vec![
        crate::Shelf {
            title: "Trending".into(),
            strapline: "".into(),
            layout: 1,
            cards: ModelRc::default(),
            columns: data::model(data::tracks_to_columns(default_quick_picks, 4)),
        },
    ]));

    // Fill the default shelves' art in the background.
    let patch_app = app.clone();
    app.rt.spawn(async move {
        let vids = [
            "34Na4j8AVgA", "4NRXx6U8ABQ", "XXYlFuWEuKI", "uPD0QOGChkM", "ygTZZpVHNmA",
            "eVli-tstM5E", "d5gf9dXbPi0", "cF1Na4AIecM", "1PIAA3KGIVA", "q3zKK4vzyEk",
            "H5v3kku4y6Q", "kTJczUoc26U", "mRD0-GxqHVo", "TUVcZfQe-Kw",
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
                let home = app_window.global::<crate::HomeData>();
                patch_home_art(home.get_top_picks(), &vid, &raw);
                if home.get_has_lead_shelf() {
                    let mut lead = home.get_lead_shelf();
                    if patch_shelf_item_model(&mut lead, &vid, &raw) {
                        home.set_lead_shelf(lead);
                    }
                }
                let shelves = home.get_shelves();
                for i in 0..shelves.row_count() {
                    let Some(mut shelf) = shelves.row_data(i) else { continue };
                    if patch_shelf_item_model(&mut shelf, &vid, &raw) {
                        shelves.set_row_data(i, shelf);
                    }
                }
            });
        }
    });
}

fn patch_shelf_item_model(shelf: &mut crate::Shelf, vid: &str, raw: &crate::images::RawImage) -> bool {
    let mut changed = false;
    for c in 0..shelf.cards.row_count() {
        if let Some(mut item) = shelf.cards.row_data(c) {
            if item.video_id == vid {
                item.thumbnail = raw.to_slint();
                item.has_thumbnail = true;
                shelf.cards.set_row_data(c, item);
                changed = true;
            }
        }
    }
    for c in 0..shelf.columns.row_count() {
        if let Some(col) = shelf.columns.row_data(c) {
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
            shelf.columns.set_row_data(c, col);
        }
    }
    changed
}

/// Patch a flat cards model (top picks, new releases) by video id.
fn patch_home_art(model: ModelRc<crate::ShelfItem>, vid: &str, raw: &crate::images::RawImage) {
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

#[allow(clippy::too_many_arguments)]
fn mood(
    title: &str,
    br: u8, bg: u8, bb: u8,
    sr: u8, sg: u8, sb: u8,
) -> crate::MoodItem {
    crate::MoodItem {
        key: title.into(),
        title: title.into(),
        background: slint::Color::from_rgb_u8(br, bg, bb),
        stroke: slint::Color::from_rgb_u8(sr, sg, sb),
    }
}

// ----------------------------------------------------------------- callbacks

fn wire_callbacks(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let home = app_window.global::<crate::HomeData>();

    // A single item (Top Picks card): play it, or search the title when it
    // has no video id (a mood/playlist card).
    {
        let app = app.clone();
        home.on_play(move |item| {
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

    // A shelf row: the queue becomes that shelf from the clicked item on.
    {
        let app = app.clone();
        home.on_play_shelf(move |shelf, item| {
            data::play_shelf(&app, &shelf, &item);
        });
    }

    // A mood tile: search its title, play the first match (the mood station's
    // stand-in until mood radios are wired).
    {
        let app = app.clone();
        home.on_play_mood(move |title| {
            play_first_search(&app, title.to_string());
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

/// The account's listening history, newest first (signed out: skipped).
fn fetch_recently_played(app: &App) {
    let app = app.clone();
    let rt = app.rt.clone();
    rt.spawn(async move {
        let Ok(history) = ytm::history(&app.api).await else { return };
        let http = app.http.clone();
        let cache = app.state.image_cache.clone();
        let mut items = Vec::new();
        for t in history.iter().take(RECENT_LIMIT) {
            items.push(data::track_to_item(&http, &cache, t).await);
        }
        if items.is_empty() {
            return;
        }
        // The TrackShelf's 4-row columns; the UI structs are built inside the
        // event-loop closure (the runtime can't hold `slint::Image`s).
        let shelf = data::PendingShelf {
            title: "Recently Played".into(),
            strapline: String::new(),
            layout: 1,
            cards: Vec::new(),
            columns: data::pending_columns(items, 4),
        };
        let weak = app.weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let home = app_window.global::<crate::HomeData>();
            home.set_recently_played(shelf.into_shelf());
            home.set_has_recently_played(true);
        });
    });
}

/// The live feed: the first card shelf becomes Top Picks, everything after it
/// renders as dynamic shelves (HomePage's slice order: shelf, moods, rest).
fn fetch_live(app: &App) {
    let app = app.clone();
    let rt = app.rt.clone();
    rt.spawn(async move {
        let _ = app.api.ensure_visitor().await;
        let Ok(home_page) = ytm::home(&app.api, None, false).await else { return };
        let http = app.http.clone();
        let cache = app.state.image_cache.clone();

        let mut top_picks: Vec<data::PendingItem> = Vec::new();
        let mut shelves: Vec<data::PendingShelf> = Vec::new();

        for shelf in &home_page.shelves {
            match &shelf.kind {
                ShelfKind::Cards(cards) => {
                    if cards.is_empty() {
                        continue;
                    }
                    // The lead card shelf is Top Picks (Android's lead row);
                    // it doesn't repeat as a dynamic shelf.
                    if top_picks.is_empty() {
                        for c in cards.iter().take(6) {
                            top_picks.push(data::card_to_item(&http, &cache, c).await);
                        }
                        continue;
                    }
                    let mut items = Vec::new();
                    for c in cards.iter().take(CARDS_PER_SHELF) {
                        items.push(data::card_to_item(&http, &cache, c).await);
                    }
                    let all_round = cards.iter().all(|c| c.round);
                    shelves.push(data::PendingShelf {
                        title: shelf.title.clone(),
                        strapline: shelf.strapline.clone().unwrap_or_default(),
                        layout: if all_round { 2 } else { 0 },
                        cards: items,
                        columns: Vec::new(),
                    });
                }
                ShelfKind::Songs { tracks, rows } => {
                    if tracks.is_empty() {
                        continue;
                    }
                    let mut items = Vec::new();
                    for t in tracks.iter().take(COLUMN_LIMIT * 4) {
                        items.push(data::track_to_item(&http, &cache, t).await);
                    }
                    shelves.push(data::PendingShelf {
                        title: shelf.title.clone(),
                        strapline: shelf.strapline.clone().unwrap_or_default(),
                        layout: 1,
                        cards: Vec::new(),
                        columns: data::pending_columns(items, *rows as usize),
                    });
                }
            }
        }

        if top_picks.is_empty() && shelves.is_empty() {
            return;
        }

        // The UI structs (and their `slint::Image`s) are built here, on the
        // UI thread.
        let weak = app.weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app_window) = weak.upgrade() else { return };
            let home = app_window.global::<crate::HomeData>();
            if !top_picks.is_empty() {
                home.set_top_picks(data::model(data::into_items(top_picks)));
            }
            if !shelves.is_empty() {
                let mut ui_shelves: Vec<_> = shelves.into_iter().map(data::PendingShelf::into_shelf).collect();
                let lead = ui_shelves.remove(0);
                home.set_lead_shelf(lead);
                home.set_has_lead_shelf(true);
                home.set_shelves(data::model(ui_shelves));
            }
        });
    });
}
