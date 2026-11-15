//! Spotify authentication and user profile loading.

use std::fs;
use serde_json::Value;
use crate::platform::paths::get_spotify_auth_path;

pub fn load_spotify_auth() -> Option<Value> {
    let path = get_spotify_auth_path();
    if path.exists()
        && let Ok(content) = fs::read_to_string(&path)
            && let Ok(val) = serde_json::from_str::<Value>(&content) {
                let has_sp_dc = val
                    .get("sp_dc")
                    .and_then(|c| c.as_str())
                    .map(|c| !c.is_empty())
                    .unwrap_or(false);
                if has_sp_dc {
                    return Some(val);
                }
            }
    None
}

pub fn save_spotify_auth(val: &Value) {
    let has_sp_dc = val
        .get("sp_dc")
        .and_then(|c| c.as_str())
        .map(|c| !c.is_empty())
        .unwrap_or(false);
    if !has_sp_dc {
        log::warn!("[BeatFlow Spotify Auth] Refusing to save spotify_auth without sp_dc cookie");
        return;
    }
    let path = get_spotify_auth_path();
    if let Ok(content) = serde_json::to_string_pretty(val) {
        let _ = fs::write(&path, content);
    }
}
