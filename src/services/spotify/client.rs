//! Spotify Web API client: user profile, playlists, and track listings.

use serde_json::Value;
use crate::services::network::client;

pub struct SpotifyClient {
    http: &'static reqwest::Client,
}

impl Default for SpotifyClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self { http: client() }
    }

    /// Fetches a Spotify user access token using the `sp_dc` cookie.
    pub async fn get_access_token(&self, sp_dc: &str) -> Option<String> {
        let resp = self.http
            .get("https://open.spotify.com/get_access_token?reason=transport&productType=web_player")
            .header("Cookie", format!("sp_dc={}", sp_dc))
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let json: Value = resp.json().await.ok()?;
        let is_anon = json.get("isAnonymous").and_then(Value::as_bool).unwrap_or(false);
        if is_anon {
            return None;
        }
        json.get("accessToken").and_then(Value::as_str).map(String::from)
    }

    /// Fetches the authenticated user profile.
    pub async fn fetch_profile(&self, access_token: &str) -> Option<(String, Option<String>)> {
        let resp = self.http
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let json: Value = resp.json().await.ok()?;
        let name = json
            .get("display_name")
            .and_then(Value::as_str)
            .or_else(|| json.get("id").and_then(Value::as_str))
            .map(String::from)?;

        let avatar = json
            .get("images")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|img| img.get("url"))
            .and_then(Value::as_str)
            .map(String::from);

        Some((name, avatar))
    }
}
