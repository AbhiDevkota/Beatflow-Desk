//! Artwork loading and caching (was in main.rs).
//!
//! Images are fetched once, decoded to RGBA, and shared by `Arc` behind a
//! mutex'd `HashMap`. Slint `Image`s are cheap handles into the same pixels,
//! so one cache entry feeds the LCD, the shelves and the search rows.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A decoded RGBA image, shared between the cache and the UI.
#[derive(Clone)]
pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RawImage {
    pub fn to_slint(&self) -> slint::Image {
        let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(self.width, self.height);
        buf.make_mut_bytes().copy_from_slice(&self.rgba);
        slint::Image::from_rgba8(buf)
    }
}

pub type ImageCache = Arc<Mutex<HashMap<String, Arc<RawImage>>>>;

pub fn new_cache() -> ImageCache {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn fetch_image_raw(http: &reqwest::Client, url: &str) -> Option<Arc<RawImage>> {
    let resp = http.get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(Arc::new(RawImage {
        width,
        height,
        rgba: img.into_raw(),
    }))
}

pub async fn get_or_fetch_image(
    http: &reqwest::Client,
    cache: &ImageCache,
    url: &str,
) -> Option<Arc<RawImage>> {
    if url.is_empty() {
        return None;
    }
    {
        if let Some(cached) = cache.lock().unwrap().get(url) {
            return Some(cached.clone());
        }
    }
    if let Some(img) = fetch_image_raw(http, url).await {
        cache.lock().unwrap().insert(url.to_string(), img.clone());
        Some(img)
    } else {
        None
    }
}

/// The `mqdefault` still YouTube serves for every video id.
pub async fn get_video_thumbnail(
    http: &reqwest::Client,
    cache: &ImageCache,
    video_id: &str,
) -> Option<Arc<RawImage>> {
    if video_id.is_empty() {
        return None;
    }
    let url = format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg");
    get_or_fetch_image(http, cache, &url).await
}
