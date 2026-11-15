//! Shared HTTP network services and client builder.

use std::sync::OnceLock;

static SHARED_HTTP: OnceLock<reqwest::Client> = OnceLock::new();

pub fn client() -> &'static reqwest::Client {
    SHARED_HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("BeatFlow/1.9.2")
            .build()
            .unwrap_or_default()
    })
}
