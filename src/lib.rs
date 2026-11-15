//! BeatFlow Native Desktop Application Core & Services Library.

pub mod core;
pub mod platform;
pub mod services;

pub use core::{
    AppSettings, AudioPlayer, BeatflowEngine, DownloadManager, LibraryStore, LyricsAggregator,
    NowPlaying, PlayerStatus, QueueTimeline, RepeatMode, Track,
};
