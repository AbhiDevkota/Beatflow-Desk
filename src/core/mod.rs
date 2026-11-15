//! BeatFlow framework-independent application core.

pub mod app_coordinator;
pub mod downloads;
pub mod library;
pub mod lyrics;
pub mod playback;
pub mod queue;
pub mod search;
pub mod state;

pub use app_coordinator::AppCoordinator;
pub use downloads::DownloadManager;
pub use library::LibraryStore;
pub use lyrics::LyricsAggregator;
pub use playback::{AudioPlayer, BeatflowEngine};
pub use queue::QueueTimeline;
pub use state::{AppSettings, NowPlaying, PlayerStatus, QueueEntry, RepeatMode, Track};
