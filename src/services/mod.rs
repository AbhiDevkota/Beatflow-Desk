//! Third-party music providers, network, and integration services.

pub mod network;
pub mod spotify;
pub mod youtube;

pub use network::{client, get_loopback_port, start_loopback_server};
pub use spotify::SpotifyClient;
pub use youtube::YouTubeClient;
