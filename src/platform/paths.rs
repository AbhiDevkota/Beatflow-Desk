//! BeatFlow platform-specific directory and file path management.

use std::fs;
use std::path::PathBuf;

pub fn get_music_dir() -> PathBuf {
    let base_music = dirs::audio_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Music")))
        .unwrap_or_else(|| PathBuf::from("."));

    let subfolder = if cfg!(debug_assertions) {
        "BeatFlow Dev"
    } else {
        "BeatFlow"
    };

    let dir = base_music.join(subfolder);
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_config_dir() -> PathBuf {
    let dir = get_music_dir().join("config");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_songs_dir() -> PathBuf {
    let dir = get_music_dir().join("songs");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_lyrics_dir() -> PathBuf {
    let dir = get_music_dir().join("lyrics");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_thumbnails_dir() -> PathBuf {
    let dir = get_music_dir().join("thumbnails");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_thumbnails_image_dir() -> PathBuf {
    let dir = get_thumbnails_dir().join("image");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_thumbnails_art_dir() -> PathBuf {
    let dir = get_thumbnails_dir().join("art");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_connector_dir() -> PathBuf {
    let dir = get_music_dir().join("connector");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_cache_dir() -> PathBuf {
    let dir = get_music_dir().join("cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn get_addons_config_path() -> PathBuf {
    let new_path = get_config_dir().join("addons.json");
    let old_path = get_music_dir().join("addons.json");
    if old_path.exists() && !new_path.exists() {
        let _ = fs::rename(&old_path, &new_path);
    }
    new_path
}

pub fn get_yt_auth_path() -> PathBuf {
    get_config_dir().join("yt_auth.json")
}

pub fn get_spotify_auth_path() -> PathBuf {
    get_config_dir().join("spotify_auth.json")
}

pub fn get_settings_path() -> PathBuf {
    get_config_dir().join("settings.json")
}

pub fn get_library_path() -> PathBuf {
    get_config_dir().join("library.json")
}
