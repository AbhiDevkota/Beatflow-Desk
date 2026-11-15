//! Synchronized and plain lyrics data structures.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedRun {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub runs: Vec<TimedRun>,
    pub is_instrumental_gap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLyrics {
    pub lines: Vec<LyricLine>,
    pub is_synced: bool,
    pub has_word_sync: bool,
    pub raw_ttml: Option<String>,
}
