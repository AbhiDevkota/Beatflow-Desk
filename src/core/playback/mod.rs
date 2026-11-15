//! Playback subsystems: native audio output, Hi-Res addons, and DASH stream parsing.

pub mod dash;
pub mod engine;
pub mod hires;
pub mod hires_engine;

pub use dash::DolbyPlan;
pub use engine::AudioPlayer;
pub use hires::{Candidate, DownloadSource, InstalledAddon, StreamInfo};
pub use hires_engine::{BeatflowEngine, TrackPayload};
