// Backend modules adapted from the reference/rust branch (BitChord).
// These provide audio playback, streaming, search (innertube), lyrics, downloads.
// UI bindings to Slint are wired in main.rs.

pub mod audio;
pub mod hires;
pub mod innertube;
pub mod jsc;
pub mod model;
pub mod opus;
pub mod proxy;
pub mod stream;
pub mod types;
pub mod webmusic;
pub mod ytm;
pub mod ytmapi;
pub use types::*;
pub mod lyrics;
pub mod downloads;
// pub mod images;
// pub mod innertube;
