# Existing Rust Modules & Reuse Classification

This document categorizes all existing Rust code in `desktop/src-tauri/src` to ensure maximum reuse of proven production logic during migration to native Rust + Slint.

---

## 1. Module Inventory & Summary

| File | Lines | Size | Primary Responsibility |
| :--- | :--- | :--- | :--- |
| `src/main.rs` | 6 | 196 B | Application executable entry point |
| `src/lib.rs` | 4,033 | 158 KB | HTTP proxy, auth management, downloader, local HTTP streamer, protocol & IPC |
| `src/engine.rs` | 270 | 10.4 KB | `BeatflowEngine`: stream resolution, upgrade logic, Atmos toggles |
| `src/hires.rs` | 892 | 36.7 KB | Candidate scoring, title cleaning, duration matching, addon client |
| `src/dash.rs` | 168 | 7.0 KB | Dolby Atmos & DASH MPD manifest parser using `roxmltree` |

---

## 2. Reusability Classification

### Category A: Reusable Directly Without Changes

1. **`src/dash.rs` (100% Reusable)**
   - **Contents:** `DolbyPlan`, XML segment template resolver, duration parser.
   - **Dependencies:** `roxmltree`, `url`, `anyhow`.
   - **Notes:** Completely independent of Tauri. Can be copied or included directly into the core crate.

2. **`src/hires.rs` (100% Reusable)**
   - **Contents:**
     - Data models: `StreamInfo`, `Candidate`, `DownloadSource`, `InstalledAddon`, `Manifest`.
     - Logic: `clean_title()`, `is_artist_name()`, candidate ranking, duration tolerance checks.
     - HTTP Addon client: `AddonClient` (manifest parsing, search queries, stream probing).
     - Full test suite: 6 unit tests passing.
   - **Dependencies:** `reqwest`, `serde`, `serde_json`, `tokio`.
   - **Notes:** Completely decoupled from Tauri. Preserved entirely.

---

### Category B: Reusable With Minor Refactoring

1. **`src/engine.rs` (95% Reusable)**
   - **Contents:** `BeatflowEngine`, stream cache, auto-upgrade coordinator, Atmos preference state.
   - **Refactoring Required:**
     - Replace `crate::load_addons_from_disk()` with configuration service reference.
     - Provide clean method accessors for Slint state bridges.
   - **Logic Retained:** 100% of stream caching, candidate resolution, and quality upgrade orchestration.

2. **`src/lib.rs` - Downloader & Metadata Tagger (80% Reusable)**
   - **Contents:**
     - `downloads.downloadTrack` routine.
     - `mp4ameta` MP4/M4A tag embedding (cover art, artist, album, title, lyrics).
     - Directory resolution: `get_music_dir()`, `get_songs_dir()`, `get_lyrics_dir()`, `get_thumbnails_dir()`.
   - **Refactoring Required:** Extract from Tauri `bridge_call` into a dedicated `crate::core::downloads::DownloadManager`.

3. **`src/lib.rs` - Local Loopback Streamer & Range Server (90% Reusable)**
   - **Contents:**
     - Port `51189` loopback TCP server.
     - HTTP 206 Partial Content byte-range streaming for `/local/<file>`.
     - Static file server for `/thumbnail/<file>`.
     - Single-instance argument forwarder via `/forward-arg`.
   - **Refactoring Required:** Remove `tauri::AppHandle` parameter; accept a generic channel or event dispatcher for deep links.

---

### Category C: Tauri-Specific Code Requiring Adaptation

1. **`src/lib.rs` - IPC `bridge_call` Dispatcher**
   - **Current Implementation:** Receives JSON requests from web `window.__BEATFLOW_CALL__`.
   - **Target Implementation:** Converted to native Rust function calls and Slint property/callback bindings in `src/ui/bridge.rs`.

2. **`src/lib.rs` - Custom Protocol `beatflow://`**
   - **Current Implementation:** Registered with `tauri::Builder::register_asynchronous_uri_scheme_protocol`.
   - **Target Implementation:** Windows registry handler invokes BeatFlow executable with `beatflow://...` argument, which forwards to running instance or loads in-process.

3. **`src/lib.rs` - WebView2 Cookie Extraction & Login Window**
   - **Current Implementation:** Creates secondary Tauri WebviewWindow (`google-login`, `spotify-login`) and invokes `extract_webview2_cookies()`.
   - **Target Implementation:** Adapted using native browser capture pass (CDP DevTools Active Port) or a lightweight native webview dialog.

---

### Category D: Code Needing Replacement

1. **Tauri Runtime Shell:**
   - `tauri::Builder`, `tauri::generate_context!()`, `tauri.conf.json`.
   - Replaced by Slint application runtime (`slint::include_modules!()`, `MainWindow`).
