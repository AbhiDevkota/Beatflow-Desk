//! Network services: HTTP client, loopback streaming server, and CORS proxy.

pub mod http;
pub mod loopback;

pub use http::client;
pub use loopback::{get_loopback_port, start_loopback_server};
