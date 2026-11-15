//! Downloads management and metadata tagging.

pub mod manager;
pub mod tagger;

pub use manager::{DownloadManager, DownloadParams, DownloadedItem, sanitize_stem};
pub use tagger::tag_audio_file;
