//! Download coordinator, file storage, and offline library tracking.

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::tagger::tag_audio_file;
use crate::platform::paths::{
    get_connector_dir, get_lyrics_dir, get_songs_dir, get_thumbnails_art_dir,
    get_thumbnails_image_dir,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedItem {
    pub file_name: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: Option<u32>,
    pub file_size: u64,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub animated_art_path: Option<String>,
    pub lyrics_path: Option<String>,
    pub video_id: Option<String>,
    pub date_added_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadParams {
    pub url: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: Option<u32>,
    pub thumbnail_url: Option<String>,
    pub animated_art_url: Option<String>,
    pub lyrics_text: Option<String>,
    pub lyrics_ttml: Option<String>,
    pub video_id: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadMeta {
    pub album: Option<String>,
    pub duration_sec: Option<u32>,
    pub source: Option<String>,
    pub video_id: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub fn sanitize_stem(raw: &str) -> String {
    let illegal = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    let clean: String = raw
        .chars()
        .map(|c| if illegal.contains(&c) || c.is_control() { ' ' } else { c })
        .collect();
    let collapsed = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().trim_matches('.').to_string()
}

pub struct DownloadManager {
    http: reqwest::Client,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { http }
    }

    /// Downloads an audio track, saves metadata, lyrics, and embeds tags.
    pub async fn download_track(&self, params: DownloadParams) -> anyhow::Result<DownloadedItem> {
        let title_clean = sanitize_stem(&params.title);
        let artist_clean = sanitize_stem(&params.artist);
        let file_stem = format!("{artist_clean} - {title_clean}");

        // 1. Fetch audio stream bytes
        let resp = self.http.get(&params.url).send().await?;
        anyhow::ensure!(resp.status().is_success(), "Failed to download audio: HTTP {}", resp.status());
        let audio_bytes = resp.bytes().await?;

        // 2. Determine file extension
        let ext = if params.url.contains(".flac") {
            "flac"
        } else if params.url.contains(".mp3") {
            "mp3"
        } else if params.url.contains(".wav") {
            "wav"
        } else {
            "m4a"
        };

        let songs_dir = get_songs_dir();
        let file_name = format!("{file_stem}.{ext}");
        let file_path = songs_dir.join(&file_name);
        fs::write(&file_path, &audio_bytes)?;

        // 3. Download and save thumbnail if provided
        let mut thumbnail_path = None;
        let mut cover_bytes = None;
        if let Some(ref thumb_url) = params.thumbnail_url
            && let Ok(thumb_resp) = self.http.get(thumb_url).send().await
                && thumb_resp.status().is_success()
                    && let Ok(t_bytes) = thumb_resp.bytes().await {
                        let t_ext = if t_bytes.starts_with(&[0xff, 0xd8]) {
                            "jpg"
                        } else {
                            "png"
                        };
                        let t_path = get_thumbnails_image_dir().join(format!("{file_stem}.{t_ext}"));
                        let _ = fs::write(&t_path, &t_bytes);
                        thumbnail_path = Some(t_path.to_string_lossy().to_string());
                        cover_bytes = Some(t_bytes.to_vec());
                    }

        // 4. Download animated artwork if provided
        let mut animated_art_path = None;
        if let Some(ref anim_url) = params.animated_art_url
            && !anim_url.is_empty()
                && let Ok(anim_resp) = self.http.get(anim_url).send().await
                    && anim_resp.status().is_success()
                        && let Ok(a_bytes) = anim_resp.bytes().await {
                            let a_path = get_thumbnails_art_dir().join(format!("{file_stem}.mp4"));
                            let _ = fs::write(&a_path, &a_bytes);
                            animated_art_path = Some(a_path.to_string_lossy().to_string());
                        }

        // 5. Tag audio file if M4A
        if ext == "m4a" {
            let _ = tag_audio_file(
                &file_path,
                &params.title,
                &params.artist,
                params.album.as_deref(),
                cover_bytes.as_deref(),
            );
        }

        // 6. Save lyrics: prefer TTML if provided, else LRC
        let mut lyrics_path = None;
        if let Some(ref ttml) = params.lyrics_ttml {
            if !ttml.is_empty() {
                let l_path = get_lyrics_dir().join(format!("{file_stem}.ttml"));
                let _ = fs::write(&l_path, ttml);
                lyrics_path = Some(l_path.to_string_lossy().to_string());
            }
        } else if let Some(ref lrc) = params.lyrics_text
            && !lrc.is_empty() {
                let l_path = get_lyrics_dir().join(format!("{file_stem}.lrc"));
                let _ = fs::write(&l_path, lrc);
                lyrics_path = Some(l_path.to_string_lossy().to_string());
            }

        // 7. Save connector metadata JSON
        let meta = DownloadMeta {
            album: params.album.clone(),
            duration_sec: params.duration_sec,
            source: params.source.clone(),
            video_id: params.video_id.clone(),
            thumbnail_url: params.thumbnail_url.clone(),
        };
        if let Ok(meta_json) = serde_json::to_string_pretty(&meta) {
            let meta_file = get_connector_dir().join(format!("{file_stem}.json"));
            let _ = fs::write(&meta_file, meta_json);
        }

        let date_added_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(DownloadedItem {
            file_name,
            title: params.title,
            artist: params.artist,
            album: params.album,
            duration_sec: params.duration_sec,
            file_size: audio_bytes.len() as u64,
            file_path: file_path.to_string_lossy().to_string(),
            thumbnail_path,
            animated_art_path,
            lyrics_path,
            video_id: params.video_id,
            date_added_ms,
        })
    }

    /// Lists all downloaded audio files and their metadata.
    pub fn list_downloads(&self) -> Vec<DownloadedItem> {
        let mut items = Vec::new();
        let songs_dir = get_songs_dir();
        if let Ok(entries) = fs::read_dir(&songs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if matches!(ext.as_str(), "m4a" | "mp3" | "flac" | "wav" | "webm" | "ogg") {
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let date_added_ms = entry.metadata()
                            .and_then(|m| m.created().or_else(|_| m.modified()))
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);

                        // Parse artist - title from stem
                        let (artist, title) = if let Some((a, t)) = stem.split_once(" - ") {
                            (a.trim().to_string(), t.trim().to_string())
                        } else {
                            ("Unknown Artist".to_string(), stem.to_string())
                        };

                        // Check for connector metadata JSON
                        let meta_path = get_connector_dir().join(format!("{stem}.json"));
                        let mut album = None;
                        let mut duration_sec = None;
                        let mut video_id = None;
                        if meta_path.exists()
                            && let Ok(content) = fs::read_to_string(&meta_path)
                                && let Ok(meta) = serde_json::from_str::<DownloadMeta>(&content) {
                                    album = meta.album;
                                    duration_sec = meta.duration_sec;
                                    video_id = meta.video_id;
                                }

                        // Check for thumbnail
                        let thumb_image = get_thumbnails_image_dir().join(format!("{stem}.jpg"));
                        let thumb_image_png = get_thumbnails_image_dir().join(format!("{stem}.png"));
                        let thumbnail_path = if thumb_image.exists() {
                            Some(thumb_image.to_string_lossy().to_string())
                        } else if thumb_image_png.exists() {
                            Some(thumb_image_png.to_string_lossy().to_string())
                        } else {
                            None
                        };

                        items.push(DownloadedItem {
                            file_name,
                            title,
                            artist,
                            album,
                            duration_sec,
                            file_size,
                            file_path: path.to_string_lossy().to_string(),
                            thumbnail_path,
                            animated_art_path: None,
                            lyrics_path: None,
                            video_id,
                            date_added_ms,
                        });
                    }
                }
            }
        }
        items
    }

    /// Deletes a downloaded track and its associated thumbnails, lyrics, and metadata.
    pub fn delete_download(&self, file_name: &str) -> anyhow::Result<()> {
        let songs_dir = get_songs_dir();
        let target_file = songs_dir.join(file_name);
        if target_file.exists() {
            fs::remove_file(target_file)?;
        }

        let stem = Path::new(file_name).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !stem.is_empty() {
            let _ = fs::remove_file(get_connector_dir().join(format!("{stem}.json")));
            let _ = fs::remove_file(get_lyrics_dir().join(format!("{stem}.lrc")));
            let _ = fs::remove_file(get_lyrics_dir().join(format!("{stem}.ttml")));
            let _ = fs::remove_file(get_thumbnails_image_dir().join(format!("{stem}.jpg")));
            let _ = fs::remove_file(get_thumbnails_image_dir().join(format!("{stem}.png")));
            let _ = fs::remove_file(get_thumbnails_art_dir().join(format!("{stem}.mp4")));
        }
        Ok(())
    }

    /// Checks if a track is already downloaded on disk.
    pub fn check_exists(&self, title: &str, artist: &str) -> bool {
        let stem = format!("{} - {}", sanitize_stem(artist), sanitize_stem(title));
        let songs_dir = get_songs_dir();
        for ext in &["m4a", "flac", "mp3", "wav"] {
            if songs_dir.join(format!("{stem}.{ext}")).exists() {
                return true;
            }
        }
        false
    }
}
