// Backend modules adapted from the reference/rust branch (BitChord).
// These provide audio playback, streaming, search (innertube), lyrics, downloads.
// UI bindings to Slint are wired in main.rs.

pub mod audio;
pub mod hires;
pub mod model;
pub mod opus;
pub mod proxy;
pub mod stream;
// Deferred (require ytm/ytmapi/webmusic and many deps):
// pub mod dash;
// pub mod downloads;
// pub mod images;
// pub mod innertube;
// pub mod lyrics;
