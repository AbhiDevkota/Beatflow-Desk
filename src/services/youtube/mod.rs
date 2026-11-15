//! YouTube Music Innertube client and authentication services.

pub mod auth;
pub mod client;

pub use auth::{compute_youtube_authorization, load_yt_auth, save_yt_auth, sha1_hex};
pub use client::{FetchParams, FetchResult, YouTubeClient};
