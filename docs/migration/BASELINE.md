# Baseline Architecture & Verification Audit

**Project:** BeatFlow Desktop  
**Target:** React + Tauri to Native Rust + Slint Migration  
**Date:** 2026-11-15  
**Auditor:** Senior Rust Desktop Engineer & Migration Lead  

---

## 1. System Environment

- **Host Operating System:** Windows 10/11 x86_64
- **Rust Toolchain:** `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1 (5f94df478 2026-08-27)`
- **Node.js Environment:** `node v24.18.0`, `npm 11.16.0`
- **Git Branch:** `feat/native-rust-slint-migration`
- **Configured Identity:** `Abhi Devkota <abhidevkota1234@gmail.com>`

---

## 2. Existing Architecture Overview

The pre-migration BeatFlow desktop application operates on a hybrid architecture:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        Tauri Application Shell                         │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    WebView2 Runtime (Chromium)                   │  │
│  │                                                                  │  │
│  │  - React 19.3.1 + TypeScript + Tailwind CSS v4                   │  │
│  │  - Zustand State Store (player.ts, ui.ts, video.ts)              │  │
│  │  - TanStack Query v5 (caching, infinite queries)                 │  │
│  │  - Shaka Player & Hls.js (HTML5 audio decoding)                  │  │
│  │  - YouTube.js / Innertube API client                             │  │
│  │  - @uimaxbai/am-lyrics component (TTML & synced LRC lyrics)      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                   │                                    │
│                 Tauri IPC Bridge / Loopback HTTP (51189)               │
│                                   ▼                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     Rust Backend Core Services                   │  │
│  │                                                                  │  │
│  │  - HTTP Reverse Proxy & Header Spoofing (reqwest)                │  │
│  │  - YouTube & Spotify Auth Management (SAPISID / sp_dc)           │  │
│  │  - Hi-Res & Lossless Stream Resolver (BeatflowEngine)            │  │
│  │  - Community Addon Engine (Tidal, Qobuz, Deezer candidate rank)  │  │
│  │  - Dolby Atmos DASH Manifest Parser (dash.rs, roxmltree)         │  │
│  │  - Metadata Tagger & Downloader (mp4ameta, dirs)                 │  │
│  │  - Local HTTP Server (Range seeking on localhost:51189)          │  │
│  │  - Custom Protocol (beatflow://) & Single Instance Guard        │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Baseline Verification Results

### Cargo Check
- **Command:** `cargo check --manifest-path desktop\src-tauri\Cargo.toml`
- **Result:** Succeeded (`Finished dev profile [unoptimized + debuginfo] in 45.15s`)
- **Errors:** 0

### Cargo Tests
- **Command:** `cargo test --manifest-path desktop\src-tauri\Cargo.toml`
- **Result:** Succeeded (`Finished test profile in 56.79s`)
- **Suite Results:**
  - `hires::tests::the_badge_says_what_the_stream_is` ... **OK**
  - `hires::tests::base_is_normalized_like_the_addon_protocol` ... **OK**
  - `hires::tests::a_copy_without_the_artist_is_not_the_song` ... **OK**
  - `hires::tests::atmos_rows_are_read_off_the_names_addons_use` ... **OK**
  - `hires::tests::a_row_matches_only_its_own_recording` ... **OK**
  - `hires::tests::quality_labels_are_read` ... **OK**
  - `http_tests::test_youtubei_browse_does_not_return_400` ... **OK**
  - `http_tests::test_youtubei_browse_authenticated_if_auth_present` ... **OK**
- **Summary:** 8 passed, 0 failed, 0 ignored.

### Cargo Clippy
- **Command:** `cargo clippy --manifest-path desktop\src-tauri\Cargo.toml`
- **Result:** Succeeded (0 compilation errors; 103 style/collapsible-if warnings).

---

## 4. Key Subsystems Identified

1. **Hi-Res Audio & Community Addons (`engine.rs`, `hires.rs`):**
   - Ranks available addon sources for Lossless, Hi-Res (24-bit/96kHz, 24-bit/192kHz), and Dolby Atmos (E-AC-3 JOC).
   - Normalizes titles, artists, and durations to prevent false candidate matches.
   - Caches upgraded streams in memory.

2. **DASH MPD Manifest Parser (`dash.rs`):**
   - Parses XML DASH manifests using `roxmltree`.
   - Resolves segment templates (`$RepresentationID$`, `$Bandwidth$`, `$Time$`, `$Number$`).
   - Extracts audio representations for Dolby Atmos and multi-channel streams.

3. **Desktop Local Loopback Server (`lib.rs`):**
   - Binds to `127.0.0.1:51189` (or dynamic port).
   - Serves local audio with HTTP 206 Partial Content byte-range seeking (`/local/<file>`).
   - Serves downloaded artwork thumbnails (`/thumbnail/<file>`).
   - Handles single-instance argument forwarding (`/forward-arg`).
   - Handles browser login redirection (`/auth-callback`, `/auth-callback-spotify`).

4. **Downloads Engine (`lib.rs`):**
   - Fetches audio bytes, writes to `Music/BeatFlow/songs/`.
   - Embeds MP4/M4A metadata, cover art, and synced lyrics using `mp4ameta`.
   - Organizes thumbnails into `image/` and `art/` subdirectories.

5. **Authentication Subsystem (`lib.rs`):**
   - Intercepts cookies via WebView2 or CDP (Chrome/Edge DevTools Active Port).
   - Computes SAPISIDHASH / SAPISID1PHASH / SAPISID3PHASH for YouTubei authentication.
   - Manages channel switching and active profile persistence.
