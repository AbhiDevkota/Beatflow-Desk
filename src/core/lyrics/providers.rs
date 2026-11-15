//! Multi-provider lyrics aggregator: LRCLib, LrcRed, LyricsPlus, Kugou.

use serde::{Deserialize, Serialize};
use super::model::ParsedLyrics;
use super::parse::{parse_lrc, parse_ttml};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsQuery {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
    pub album: Option<String>,
    pub isrc: Option<String>,
}

pub struct LyricsAggregator {
    http: reqwest::Client,
}

impl Default for LyricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl LyricsAggregator {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .user_agent("BeatFlow/1.9.2")
            .build()
            .unwrap_or_default();
        Self { http }
    }

    /// Fetches lyrics attempting providers in sequence: LrcRed (if ISRC) -> LRCLib -> LyricsPlus.
    pub async fn fetch_lyrics(&self, query: &LyricsQuery) -> Option<ParsedLyrics> {
        // 1. LrcRed if ISRC is available (provides rich word-by-word TTML)
        if let Some(ref isrc) = query.isrc
            && isrc.len() == 12 {
                let url = format!("https://lrc.red/s/{}.ttml", isrc.to_uppercase());
                if let Ok(resp) = self.http.get(&url).send().await
                    && resp.status().is_success()
                        && let Ok(xml) = resp.text().await
                            && let Some(ttml_lyrics) = parse_ttml(&xml) {
                                log::info!("[BeatFlow Lyrics] Successfully fetched TTML lyrics from LrcRed for ISRC {}", isrc);
                                return Some(ttml_lyrics);
                            }
            }

        // 2. LRCLib exact get
        let lrclib_url = format!(
            "https://lrclib.net/api/get?track_name={}&artist_name={}&duration={}",
            crate::core::playback::hires::url_encode(&query.title),
            crate::core::playback::hires::url_encode(&query.artist),
            query.duration_ms / 1000
        );

        if let Ok(resp) = self.http.get(&lrclib_url).send().await
            && resp.status().is_success()
                && let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(synced) = json.get("syncedLyrics").and_then(|v| v.as_str())
                        && !synced.trim().is_empty() {
                            log::info!("[BeatFlow Lyrics] Successfully fetched synced lyrics from LRCLib");
                            return Some(parse_lrc(synced));
                        }
                    if let Some(plain) = json.get("plainLyrics").and_then(|v| v.as_str())
                        && !plain.trim().is_empty() {
                            log::info!("[BeatFlow Lyrics] Successfully fetched plain lyrics from LRCLib");
                            return Some(parse_lrc(plain));
                        }
                }

        // 3. LRCLib search fallback
        let lrclib_search_url = format!(
            "https://lrclib.net/api/search?q={}+{}",
            crate::core::playback::hires::url_encode(&query.title),
            crate::core::playback::hires::url_encode(&query.artist)
        );

        if let Ok(resp) = self.http.get(&lrclib_search_url).send().await
            && resp.status().is_success()
                && let Ok(arr) = resp.json::<Vec<serde_json::Value>>().await {
                    for hit in arr {
                        if let Some(synced) = hit.get("syncedLyrics").and_then(|v| v.as_str())
                            && !synced.trim().is_empty() {
                                log::info!("[BeatFlow Lyrics] Successfully fetched search synced lyrics from LRCLib");
                                return Some(parse_lrc(synced));
                            }
                    }
                }

        None
    }
}
