//! BeatFlow's native shell (Slint) — entry point only.
//!
//! The window's logic lives beside it, one module per concern:
//!
//! | module        | role |
//! |---------------|------|
//! | `app`         | the shared context (window handle, audio, API, HTTP, runtime, state) |
//! | `state`       | the playback queue + generation counter |
//! | `images`      | artwork fetch/cache |
//! | `playback`    | stream resolution, transport callbacks, timers |
//! | `search`      | suggestions, grouped results, row playback |
//! | `home`        | Top Picks, Recently Played, dynamic shelves, moods |
//! | `explore`     | the New page's albums/charts/playlists |
//! | `chrome`      | Windows Mica/DWM window chrome |
//!
//! The UI tree is `ui/app-window.slint` (compiled in build.rs); the Rust side
//! only wires its globals.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod artist;
mod backend;
mod chrome;
mod collection;
mod data;
mod downloads;
mod explore;
mod home;
mod images;
mod library;
mod playback;
mod search;
mod settings;
mod state;

use std::sync::Arc;

use crossbeam_channel::unbounded;
use slint::ComponentHandle;
use tokio::runtime::Runtime;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_window = AppWindow::new()?;

    // Tokio runtime for async stream resolution/download.
    let rt = Arc::new(Runtime::new()?);

    // Audio engine.
    let (events_tx, events_rx) = unbounded::<crate::backend::audio::AudioEvent>();
    let audio = Arc::new(crate::backend::audio::spawn(
        1.0,
        false,
        None,
        events_tx,
        || {
            let _ = slint::invoke_from_event_loop(|| {});
        },
    ));

    // Shared playback state.
    let playback_state = crate::state::PlaybackState::new(images::new_cache());

    // Default player metadata.
    {
        let player = app_window.global::<Player>();
        player.set_title("No Track".into());
        player.set_artist("Select a song to play".into());
        player.set_duration(0.0);
        player.set_status(0);
    }

    // The context every module wires against.
    let app = app::App::new(
        &app_window,
        audio,
        Arc::new(crate::backend::ytmapi::Api::new(std::env::temp_dir().as_path())),
        reqwest::Client::new(),
        rt,
        playback_state,
    );

    // Window callbacks + transport + the three timers.
    let _timers = playback::wire(&app, events_rx);

    // Pages.
    search::wire(&app);
    home::wire(&app);
    explore::wire(&app);
    collection::wire(&app);
    artist::wire(&app);
    library::wire(&app);
    downloads::wire(&app);
    settings::wire(&app);

    app_window.show()?;

    // Mica + DWM shadow/corners once the window exists.
    chrome::apply(&app_window);

    app_window.run()?;
    Ok(())
}
