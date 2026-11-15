//! Direct native YouTube Music (Innertube) API client.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use super::auth::{compute_youtube_authorization, load_yt_auth};
use crate::services::network::client;

pub const INNERTUBE_CLIENT_NAME: &str = "67";
pub const INNERTUBE_CLIENT_VERSION: &str = "1.20250219.01.00";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchParams {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub body_encoding: Option<String>,
    pub auth: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub struct YouTubeClient {
    http: &'static reqwest::Client,
}

impl Default for YouTubeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeClient {
    pub fn new() -> Self {
        Self { http: client() }
    }

    /// Executes an HTTP request to YouTubei with appropriate client headers and authorization.
    pub async fn execute_request(&self, params: FetchParams, is_youtube: bool) -> anyhow::Result<FetchResult> {
        let method = match params.method.as_deref().unwrap_or("GET").to_uppercase().as_str() {
            "POST" => reqwest::Method::POST,
            "HEAD" => reqwest::Method::HEAD,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            _ => reqwest::Method::GET,
        };

        let mut req = self.http.request(method, &params.url);

        // Apply provided headers
        if let Some(headers) = params.headers {
            for (k, v) in headers {
                req = req.header(&k, &v);
            }
        }

        // Apply YouTube authentication cookies and SAPISIDHASH if auth is enabled
        if is_youtube && params.auth.unwrap_or(false)
            && let Some(auth_data) = load_yt_auth()
                && let Some(cookies) = auth_data.get("cookies").and_then(Value::as_str) {
                    req = req.header("Cookie", cookies);
                    let origin = "https://music.youtube.com";
                    if let Some(auth_header) = compute_youtube_authorization(cookies, origin) {
                        req = req.header("Authorization", auth_header);
                    }
                }

        // Apply body
        if let Some(body) = params.body {
            req = req.body(body);
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();

        let mut resp_headers = HashMap::new();
        for (k, v) in resp.headers() {
            if let Ok(val) = v.to_str() {
                resp_headers.insert(k.as_str().to_string(), val.to_string());
            }
        }

        let body = resp.text().await.unwrap_or_default();

        Ok(FetchResult {
            status,
            body,
            headers: resp_headers,
        })
    }

    /// Fetches the YouTube Music Home feed (`FEmusic_home`).
    pub async fn fetch_home(&self) -> anyhow::Result<Value> {
        let body = json!({
            "browseId": "FEmusic_home",
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": INNERTUBE_CLIENT_VERSION
                }
            }
        }).to_string();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("x-youtube-client-name".to_string(), INNERTUBE_CLIENT_NAME.to_string());
        headers.insert("x-youtube-client-version".to_string(), INNERTUBE_CLIENT_VERSION.to_string());

        let params = FetchParams {
            url: "https://www.youtube.com/youtubei/v1/browse?prettyPrint=false&alt=json".to_string(),
            method: Some("POST".to_string()),
            headers: Some(headers),
            body: Some(body),
            body_encoding: Some("text".to_string()),
            auth: Some(true),
        };

        let res = self.execute_request(params, true).await?;
        anyhow::ensure!(res.status == 200, "Home fetch failed with HTTP {}", res.status);
        let val: Value = serde_json::from_str(&res.body)?;
        Ok(val)
    }

    /// Searches YouTube Music for a given query string.
    pub async fn search(&self, query: &str) -> anyhow::Result<Value> {
        let body = json!({
            "query": query,
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": INNERTUBE_CLIENT_VERSION
                }
            }
        }).to_string();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("x-youtube-client-name".to_string(), INNERTUBE_CLIENT_NAME.to_string());
        headers.insert("x-youtube-client-version".to_string(), INNERTUBE_CLIENT_VERSION.to_string());

        let params = FetchParams {
            url: "https://www.youtube.com/youtubei/v1/search?prettyPrint=false&alt=json".to_string(),
            method: Some("POST".to_string()),
            headers: Some(headers),
            body: Some(body),
            body_encoding: Some("text".to_string()),
            auth: Some(true),
        };

        let res = self.execute_request(params, true).await?;
        anyhow::ensure!(res.status == 200, "Search fetch failed with HTTP {}", res.status);
        let val: Value = serde_json::from_str(&res.body)?;
        Ok(val)
    }

    /// Fetches live autocomplete search suggestions.
    pub async fn search_suggestions(&self, input: &str) -> anyhow::Result<Vec<String>> {
        let body = json!({
            "input": input,
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": INNERTUBE_CLIENT_VERSION
                }
            }
        }).to_string();

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("x-youtube-client-name".to_string(), INNERTUBE_CLIENT_NAME.to_string());
        headers.insert("x-youtube-client-version".to_string(), INNERTUBE_CLIENT_VERSION.to_string());

        let params = FetchParams {
            url: "https://www.youtube.com/youtubei/v1/music/get_search_suggestions?prettyPrint=false&alt=json".to_string(),
            method: Some("POST".to_string()),
            headers: Some(headers),
            body: Some(body),
            body_encoding: Some("text".to_string()),
            auth: Some(false),
        };

        let res = self.execute_request(params, true).await?;
        let mut suggestions = Vec::new();
        if let Ok(val) = serde_json::from_str::<Value>(&res.body)
            && let Some(contents) = val.pointer("/contents/0/searchSuggestionsSectionRenderer/contents").and_then(Value::as_array) {
                for item in contents {
                    if let Some(runs) = item.pointer("/searchSuggestionRenderer/navigationEndpoint/searchEndpoint/query").and_then(Value::as_str) {
                        suggestions.push(runs.to_string());
                    }
                }
            }
        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_youtubei_browse_does_not_return_400() {
        let client = YouTubeClient::new();
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("x-youtube-client-name".to_string(), "67".to_string());
        headers.insert("x-youtube-client-version".to_string(), "1.20250219.01.00".to_string());

        let body = json!({
            "browseId": "FEmusic_home",
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20250219.01.00"
                }
            }
        }).to_string();

        let params = FetchParams {
            url: "https://www.youtube.com/youtubei/v1/browse?prettyPrint=false&alt=json".to_string(),
            method: Some("POST".to_string()),
            headers: Some(headers),
            body: Some(body),
            body_encoding: Some("text".to_string()),
            auth: Some(false),
        };

        let result = client.execute_request(params, true).await.expect("Request should succeed");
        assert_eq!(result.status, 200, "Browse endpoint returned status {}", result.status);
        assert!(result.body.contains("responseContext"), "Response body should contain responseContext");
    }
}
