//! Real-time synchronized lyrics parsing and multi-provider fetching.

pub mod clock;
pub mod model;
pub mod parse;
pub mod providers;

pub use clock::LyricClock;
pub use model::{LyricLine, ParsedLyrics, TimedRun};
pub use parse::{parse_lrc, parse_ttml};
pub use providers::{LyricsAggregator, LyricsQuery};
