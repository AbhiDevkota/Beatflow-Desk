//! Library management, favorites, and user playlist persistence.

pub mod store;

pub use store::{HistoryEntry, LibraryData, LibraryStore, UserPlaylist};
