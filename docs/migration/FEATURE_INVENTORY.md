# Complete Feature Inventory

This inventory documents all features discovered in the BeatFlow codebase that must be preserved during the migration to native Rust + Slint.

---

## 1. Music Discovery & Browsing

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Top Picks Shelf** | 3:4 portrait cards based on user's recent listening and profile | `HomePage.tsx`, `TopPicksShelf.tsx`, `homeLayout.ts` | Required |
| **Recently Played Shelf** | Compact 4-row horizontal track scroller | `HomePage.tsx`, `TrackShelf.tsx` | Required |
| **Dynamic Shelves** | Personalized shelves with localized titles, subtitles, and "See All" | `HomePage.tsx`, `ShelfView.tsx` | Required |
| **Find Your Mood** | Mood categories canvas with gradient badges | `FindYourMoodShelf.tsx`, `MoodCanvas.tsx` | Required |
| **Moods & Genres Page** | Grid of genre and mood category cards with custom stripe colors | `MoodsPage.tsx`, `CategoryTiles.tsx` | Required |
| **New Releases Page** | New albums, singles, and editorial releases | `NewPage.tsx` | Required |
| **Infinite Scroll Pagination** | Dynamic preloading of next shelf batches via continuation tokens | `HomePage.tsx`, `queries.ts` | Required |

---

## 2. Search & Recommendations

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Instant Search** | Search input with debounce and query cache | `SearchPage.tsx`, `SearchField.tsx` | Required |
| **Search Suggestions** | Live autocomplete suggestions as user types | `SearchField.tsx`, `innertube.ts` | Required |
| **Top Results Cards** | Multi-column rich cards (artwork, kind label, artist, context menu) | `SearchPage.tsx` | Required |
| **Categorized Search** | Results sectioned by Artists, Albums, Songs, Playlists, Videos | `SearchPage.tsx` | Required |
| **Explore Category Tiles** | Grid of browse tiles shown when search input is empty | `SearchPage.tsx`, `CategoryTiles.tsx` | Required |

---

## 3. Playback & Audio Controls

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Core Transport** | Play, Pause, Next Track, Previous Track (with 3-second restart rule) | `player.ts`, `PlayerBar.tsx`, `FullScreenPlayer.tsx` | Required |
| **Progress & Seeking** | Real-time scrubber with elapsed & remaining time indicators | `useScrubber.ts`, `Lcd.tsx`, `FullScreenPlayer.tsx` | Required |
| **Volume & Mute** | Linear and log volume control, scroll-wheel adjustment, mute toggle | `VolumeControl.tsx`, `wheelVolume.ts` | Required |
| **Shuffle Mode** | Fisher-Yates shuffled order with active track preservation | `timeline.ts`, `player.ts` | Required |
| **Repeat Modes** | Tri-state repeat: Off, Repeat All, Repeat One | `player.ts`, `Transport.tsx` | Required |
| **AutoPlay / Radio** | Automatically extends queue with related music when queue ends | `builder.ts`, `timeline.ts` | Required |
| **Crossfade** | Smooth volume ramp between overlapping tracks (0, 3, 6, 9, 12s) | `player.ts` | Required |
| **AutoMix DJ Engine** | Beat-matched, phrase-aligned transitions between tracks | `automix/`, `timeline.ts` | Required |
| **Stats for Nerds** | Technical overlay (codec, bitrate, sample rate, buffer state) | `NerdStats.tsx`, `engine.ts` | Required |

---

## 4. Audio Quality & Hi-Res Engine

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Standard Quality** | Opus 160kbps / AAC 128kbps via YouTube Music | `engine.rs`, `resolver.ts` | Required |
| **Lossless FLAC** | 16-bit / 44.1kHz CD quality via community addons | `hires.rs`, `engine.rs` | Required |
| **Hi-Res Lossless** | 24-bit / 96kHz and 24-bit / 192kHz master quality streams | `hires.rs`, `engine.rs` | Required |
| **Dolby Atmos** | Multi-channel E-AC-3 JOC playback from DASH manifests | `dash.rs`, `engine.rs` | Required |
| **Quality Upgrade Engine**| Automatic or manual search for higher-tier streams | `engine.rs`, `qualityUpgrade.ts` | Required |
| **Revert to Original** | Downgrades upgraded track back to YouTube Music source | `engine.rs`, `player.ts` | Required |
| **Quality Badges** | Visual tier pill ("Lossless", "Hi-Res Lossless", "Dolby Atmos") | `SourceQualityBadge.tsx`, `hires.rs` | Required |

---

## 5. Queue & Playlist Management

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Context Queue** | Album or playlist context preserved in sequence | `timeline.ts` | Required |
| **Up Next Queue** | User-queued tracks placed immediately after active track | `timeline.ts`, `UpNext.tsx` | Required |
| **Drag & Drop Reordering** | Interactive reordering of upcoming queue entries | `UpNext.tsx` | Required |
| **Queue Operations** | Play Next, Add to Queue, Remove, Clear Up Next | `timeline.ts`, `UpNext.tsx` | Required |
| **Jump to Track** | Instant transition to any entry in the playback timeline | `timeline.ts`, `player.ts` | Required |

---

## 6. Real-Time Lyrics System

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Time-Synced Lyrics** | Line-by-line scrolling synchronized to playhead clock | `clock.ts`, `LyricsPanel.tsx` | Required |
| **Word-by-Word Sync** | Syllable / word-level highlight wipes from TTML documents | `ttml.ts`, `model.ts` | Required |
| **Multi-Provider Aggregation**| Fallback cascade across LRCLib, LrcRed, LyricsPlus, Kugou, Genius, YouTube | `providers.ts`, `repository.ts` | Required |
| **Pronunciation / Romanization** | Pinyin, Romaji, and phonetic transcription support | `parse.ts`, `LyricsPanel.tsx` | Required |
| **Lyrics Translation** | Integrated translations for non-native language lyrics | `LYRICS_TRANSLATION.md`, `providers.ts` | Required |
| **Manual Time Offset** | Adjust sync offset (+/- milliseconds) for timing discrepancies | `clock.ts`, `LyricsSettings.tsx` | Required |

---

## 7. Library & User Content

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Liked Songs / Favorites** | Toggle favorite status with animated star icon | `FavoriteButton.tsx`, `library.ts` | Required |
| **User Playlists** | List, view, create, edit, and delete user playlists | `LibraryPage.tsx`, `NewPlaylistModal.tsx` | Required |
| **Library Sections** | Recently Added, Artists, Albums, Songs, Made For You | `LibraryPage.tsx`, `Sidebar.tsx` | Required |
| **Detail Views** | Header with artwork, metadata, play, shuffle, and add-to-library | `DetailHeader.tsx`, `AlbumPage.tsx`, `PlaylistPage.tsx` | Required |
| **Artist Pages** | Hero image, monthly audience, top songs, discography shelves | `ArtistPage.tsx` | Required |

---

## 8. Downloads & Offline Storage

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Track Downloader** | Background download with progress, speed, and size tracking | `desktopDownloads.ts`, `lib.rs` | Required |
| **Metadata Tagging** | Embeds artist, title, album, year, artwork, and synced lyrics (ID3/MP4) | `lib.rs` (`mp4ameta`) | Required |
| **Offline Playback** | Seamless playback of downloaded local audio files | `DownloadsPage.tsx`, `lib.rs` | Required |
| **File Management** | Delete download, open in Windows Explorer, view file path | `DownloadsPage.tsx`, `lib.rs` | Required |
| **Sorting & Filtering** | Sort by Recently Added, Title, Artist, or File Size | `DownloadsPage.tsx` | Required |

---

## 9. Spotify & Third-Party Integrations

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Spotify Account Link** | Connect via `sp_dc` cookie; view user playlists and saved tracks | `SpotifyPage.tsx`, `lib.rs` | Required |
| **Spotify Playlist Import** | Match Spotify track metadata to YouTube Music equivalents | `SpotifyPlaylistPage.tsx`, `matching.ts` | Required |
| **Discord Rich Presence** | Live "Listening to..." presence with artwork and elapsed time | `discordPresence.ts`, `lib.rs` | Required |
| **Listen Together / Party** | Synchronized multi-user listening sessions via WebSocket | `listenTogether.ts`, `partySync.ts` | Required |

---

## 10. Platform & Desktop Integrations

| Feature | Description | Source File(s) | Status |
| :--- | :--- | :--- | :--- |
| **Window Controls** | Custom frameless minimize, maximize, restore, close buttons | `WindowControls.tsx`, `lib.rs` | Required |
| **Single Instance Locking**| Forward arguments from secondary launches to main instance | `lib.rs` (`127.0.0.1:51189`) | Required |
| **Custom URI Protocol** | Handle `beatflow://` deep links from external apps/browser | `lib.rs`, `deepLink.ts` | Required |
| **System Media Controls** | Windows SMTC / media keys (Play, Pause, Next, Prev) | `desktop/src-tauri` | Required |
| **System Tray Icon** | Background tray icon with quick playback controls and restore | Native OS API | Required |
