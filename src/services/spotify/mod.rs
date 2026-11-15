//! Spotify account integration, playlists, and canvas services.

pub mod auth;
pub mod client;

pub use auth::{load_spotify_auth, save_spotify_auth};
pub use client::SpotifyClient;
