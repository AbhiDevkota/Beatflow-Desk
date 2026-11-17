//! The shared app context (was a pile of per-callback clones in main.rs).
//!
//! Every module wires its callbacks against an `App`, which bundles the Slint
//! window handle, the audio engine, the YouTube Music API, the HTTP client,
//! the tokio runtime and the playback state. It's deliberately cheap to clone:
//! callbacks capture their own copy, exactly like the per-callback field
//! clones main.rs used to spell out by hand.

use std::sync::Arc;

use slint::ComponentHandle;
use tokio::runtime::Runtime;

use crate::backend::audio::AudioHandle;
use crate::backend::ytmapi::Api;
use crate::state::PlaybackState;

#[derive(Clone)]
pub struct App {
    pub weak: slint::Weak<crate::AppWindow>,
    pub audio: Arc<AudioHandle>,
    pub api: Arc<Api>,
    pub http: reqwest::Client,
    pub rt: Arc<Runtime>,
    pub state: PlaybackState,
}

impl App {
    pub fn new(
        app: &crate::AppWindow,
        audio: Arc<AudioHandle>,
        api: Arc<Api>,
        http: reqwest::Client,
        rt: Arc<Runtime>,
        state: PlaybackState,
    ) -> Self {
        Self {
            weak: app.as_weak(),
            audio,
            api,
            http,
            rt,
            state,
        }
    }
}
