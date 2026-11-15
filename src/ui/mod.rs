//! Slint UI Bindings and Event Loop Bridge.

slint::include_modules!();

pub mod bridge;

pub use bridge::UiBridge;
