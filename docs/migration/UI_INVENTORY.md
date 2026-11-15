# Comprehensive UI Component & View Inventory

This document provides a detailed breakdown of all user interface components, views, layouts, and visual states that must be preserved in Slint.

---

## 1. Application Shell & Window Chrome

### 1.1 WindowControls (`WindowControls.tsx`)
- **Location:** Top-right corner of frameless application window.
- **Controls:**
  - **Minimize:** Horizontal line glyph (`—`).
  - **Maximize / Restore:** Square glyph (`□` or overlapping squares).
  - **Close:** Cross glyph (`✕`) with red hover background (`#e81123`).
- **Drag Region:** Non-button area of top bar supports window drag (`-webkit-app-region: drag` equivalent).

### 1.2 Layout Grid (`AppShell.tsx`)
- **Structure:** Two-tier flex/grid layout:
  1. Top: Full-viewport workspace with fixed sidebar, scrolling main content, floating player pill, and slide-in side panel.
  2. Bottom: Optional contextual banner (e.g. extension or app notice).
- **Dimensions:**
  - Sidebar: Width `260px` (or responsive `33.88vw` under `768px`).
  - Main scroll area: Padded `pl-[260px] pb-[116px]`.
  - Body gutter: `25px` on standard displays; `40px` on wide screens (`≥1000px`).
  - Side panel: Width `300px` fixed on right edge.

---

## 2. Navigation Sidebar (`Sidebar.tsx`)

### 2.1 Brand Header
- **Logo:** `LogoMark` SVG glyph (`22px` height).
- **Title:** "BeatFlow" in bold typography (`700 19px/24px`, tracking `-0.01em`).

### 2.2 Navigation Items
- **Primary Group:**
  - `Search` (Icon: `SearchIcon`, path: `/search`)
  - `Home` (Icon: `HomeIcon`, path: `/home`)
  - `New` (Icon: `NewIcon`, path: `/new`)
  - `Library` (Icon: `LibraryIcon`, path: `/library`)
  - `Downloads` (Icon: `DownloadsIcon`, path: `/downloads`)
  - `Listen Together` (Icon: `Users`, path: `/settings/listen-together`)
- **Library Sub-Group (when signed in):**
  - Section Header: "Library" (`callout-emphasized`, secondary color)
  - `Recently Added` (`RecentlyAddedIcon`, path: `/library/recent`)
  - `Artists` (`ArtistsIcon`, path: `/library/artists`)
  - `Albums` (`AlbumsIcon`, path: `/library/albums`)
  - `Songs` (`SongsIcon`, path: `/library/songs`)
  - `Made for You` (`MadeForYouIcon`, path: `/library/made-for-you`)
- **Playlists Sub-Group:**
  - Section Header: "Playlists" with '+' New Playlist button.
  - `All Playlists` (`AllPlaylistsIcon`, path: `/library/playlists`)
  - User's saved/created playlist list items with `ListMusic` icon.
- **Spotify Sub-Group (when Spotify account connected):**
  - Section Header: "Spotify"
  - `All Spotify Playlists`
  - User's imported Spotify playlist list items.
- **Footer Actions:**
  - Account Profile button (`AccountButton.tsx`): Avatar thumbnail, name/email, channel switch popover.
  - Settings button: Glass pill icon button (`Settings` gear icon).

---

## 3. Mini-Player Bar (`PlayerBar.tsx`, `Lcd.tsx`)

### 3.1 Dimensions & Styling
- **Height:** `56px` (`--player-pill-height`).
- **Offset:** `20px` from bottom edge, centered within the available page area.
- **Max Width:** `668px`.
- **Border Radius:** Pill shape (`999px` / `var(--radius-pill)`).
- **Background:** Glassmorphic translucent material with blur and inner stroke.

### 3.2 Left Column: Transport Controls (`184px`)
- **Shuffle Toggle:** `ShuffleIcon`, colored with `--keyColor` when enabled.
- **Previous Button:** `BackwardIcon` (mirrored forward icon).
- **Play / Pause / Loading:**
  - Normal Play: `PlayIcon` (`28px`).
  - Active Pause: `PauseIcon` (`28px`).
  - Buffering / Loading: `BeatLoader` animated equalizer bars.
- **Next Button:** `ForwardIcon` (`26px`).
- **Repeat Toggle:** Tri-state (`RepeatIcon` for repeat-all, `RepeatOneIcon` for repeat-one, dimmed for off).

### 3.3 Center Column: LCD Display (`minmax(180px, 427px)`)
- **Artwork Thumbnail:**
  - Dimensions: `34x34px` (square) or `60x34px` (16:9 music video).
  - Hover Effect: Overlay with `Maximize2` expand glyph.
  - Interaction: Clicking opens the Fullscreen Player.
- **Track Information:**
  - Title: Truncated bold text; links to Album view; carries `ExplicitBadge` if explicit.
  - Audio Quality Badge: `PlaybackQualityLabel` ("Lossless", "Hi-Res", "Dolby Atmos").
  - Artist Line: Truncated secondary text with clickable artist credit links.
- **Scrubber Bar (`Progress`):**
  - Thin 2px line along bottom of LCD; expands to 4px on hover.
  - Hover timestamps: Elapsed time on left, remaining time on right.
  - Interactive seeking: Click or drag to seek playhead.

### 3.4 Right Column: Action Buttons (`minmax(57px, auto)`)
- **Favorite Button:** `StarIcon` (unfilled) or `StarFilledIcon` (gold/red filled).
- **Track Menu (`...`):** Context menu for Up Next, Add to Playlist, Download, etc.
- **Lyrics Button:** `LyricsIcon` / `LyricsActiveIcon`; toggles side panel lyrics.
- **Up Next Button:** `UpNextIcon`; toggles side panel queue.
- **Volume Control:** Speaker icon (`VolumeIcon` with dynamic wave arcs); popup slider on hover.

---

## 4. Fullscreen Player (`FullScreenPlayer.tsx`)

### 4.1 Visual Architecture
- **Ambient Backdrop (`MeshBackdrop.tsx`):**
  - Deep dark background (`#0c0c0e`).
  - Blurred dual-layer artwork crossfade with Gaussian blur (`84px`) and saturation boost.
  - Subtle top, bottom, and radial dark vignettes.
- **Header:**
  - Minimize button (`ChevronDown`).
  - Right tools: Audio quality selector, lyrics toggle, up next toggle.
- **Center Artwork:**
  - Large square artwork (`[--art:min(38vw,calc(100dvh-240px),560px)]`).
  - Rounded corners (`18px` to `22px`).
  - Multi-layer 3D shadow: `shadow-[0_28px_70px_-12px_rgba(0,0,0,0.7),0_12px_28px_-8px_rgba(0,0,0,0.45)]`.
- **Metadata & Scrubber:**
  - Large track title (`title-1-emphasized`).
  - Artist credits with links.
  - High-precision scrubber line with elapsed and remaining timestamps.
  - Audio tier badge (`SourceQualityBadge`).
- **Transport Controls:**
  - Prominent Play/Pause button (`46px` glyph in `56px` circular button).
  - Skip forward/backward (`38px` glyphs).
  - Shuffle and repeat toggles.
- **Volume Slider:**
  - Horizontal volume track with speaker min/max icons.
  - Smooth width animations and drag interactions.
- **Dual-Pane Mode:**
  - Displays Lyrics panel or Up Next queue side-by-side with artwork on desktop displays.

---

## 5. Side Panel (`SidePanel.tsx`, `LyricsPanel.tsx`, `UpNext.tsx`)

- **Placement:** Anchored to right viewport edge, width `300px`.
- **Tabs:** Segmented control to switch between **Lyrics** and **Up Next**.
- **Lyrics View:**
  - Time-synced scrolling lyric lines with active line magnification.
  - Unsynced fallback display with manual scroll.
  - Bottom controls: Translation and Romanization pills.
- **Up Next View:**
  - Drag-and-drop track rows.
  - Active track indicator.
  - AutoPlay toggle with infinity glyph.
  - Clear queue action button.

---

## 6. Page Templates & Views

1. **Home Page (`HomePage.tsx`):**
   - Top Picks 3:4 portrait carousel cards.
   - Recently Played 4-row track scroller.
   - Dynamic recommendation shelves (albums, artists, playlists).
   - Find Your Mood category canvas.
2. **Search Page (`SearchPage.tsx`):**
   - Search input bar with search glyph and clear button.
   - Suggestions dropdown.
   - Top Results cards grid (118px cards with 90px artwork).
   - Category result shelves (Songs, Albums, Artists, Playlists, Videos).
   - Category tiles grid for explore mode.
3. **Album Page (`AlbumPage.tsx`):**
   - Album header: Large square cover, title, artist link, year, track count, duration.
   - Play and Shuffle action buttons.
   - Numbered track list table (track #, title, explicit badge, duration, plays count, menu).
   - "More by [Artist]" shelf.
4. **Artist Page (`ArtistPage.tsx`):**
   - Wide hero header image with artist name and monthly listeners.
   - "Top Songs" numbered track list.
   - Discography shelves (Albums, Singles, EPs, Featured).
5. **Playlist Page (`PlaylistPage.tsx`):**
   - Header with cover art, playlist title, creator, total tracks.
   - Track list with draggable order, date added, duration.
6. **Downloads Page (`DownloadsPage.tsx`):**
   - Header with search filter, sort dropdown, and "Open Folder" action.
   - Active downloads section with progress bars, ETA, and download speeds.
   - Downloaded songs table with play, delete, and audio format badges.
7. **Settings Page (`SettingsPage.tsx`):**
   - 8 structured categories: Playback, Content Preferences, Audio Quality, Sources & Addons, Appearance, Lyrics, Storage, About.
   - Custom toggle switches, dropdown selects, and folder pickers.
