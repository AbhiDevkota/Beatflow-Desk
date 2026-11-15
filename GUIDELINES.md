# BeatFlow Native Migration Guidelines & Engineering Contract

This document constitutes the permanent, non-negotiable engineering contract for migrating BeatFlow from its React + TypeScript + Tauri + Rust architecture to a fully native Rust + Slint desktop application.

---

## A. Migration Philosophy

1. **Preserve the Existing Application:** The existing BeatFlow implementation (frontend and backend) is the single authoritative source of truth. We are upgrading the underlying engine and GUI framework, not inventing a new music player.
2. **Zero Redesign:** The user experience, visual layouts, colors, iconography, and interactions must match the existing production application. Do not alter design elements for personal preference.
3. **Zero Feature Reduction:** Every feature discovered in the codebase—whether prominent, secondary, experimental, or hidden—must be preserved in the native application.
4. **No Placeholder Implementations:** Completed phases must never contain mock data, stubbed actions, or `TODO` shortcuts in user-facing paths.
5. **No Unnecessary Rewrites:** Existing Rust code in `desktop/src-tauri/src` represents battle-tested production logic. Maximize reuse and refactor only when necessary to decouple from the Tauri runtime.
6. **Zero Visual Regressions:** All components, paddings, margins, border radii, shadows, typography, and states (hover, active, focus, disabled, loading, empty, error) must achieve parity with the web baseline.

---

## B. UI Preservation Rules

1. **Component Dimensions & Positioning:** Maintain exact dimensions:
   - Sidebar width: `260px` (or `33.88vw` on smaller desktop views).
   - Player bar (pill): height `56px`, bottom offset `20px`, max width `668px`, 3-column layout (`184px`, `minmax(180px, 427px)`, `minmax(57px, auto)`).
   - Track rows: `55px` row height, artwork `40px` (or `60px` for video tracks).
   - Side panel: width `300px` on the right side.
   - Body gutter: `25px` (<1000px) / `40px` (≥1000px).
2. **Typography & Styling:**
   - Font family: San Francisco Pro Display / DM Sans fallback.
   - Headings, body styles, callouts, captions, line heights, and weights defined in `tokens.css`.
   - Exact color variables: Light theme (`--pageBG: #ffffff`, `--systemPrimary: #000000e0`, `--keyColor: #d60017`), Dark theme (`--pageBG: #1f1f1f`, `--systemPrimary: #ffffffeb`, `--keyColor: #fa586a`), artwork dynamic tinting (`data-tint`).
3. **Asset & Icon Geometry:**
   - Use identical SVG vector paths from `NavIcons.tsx`, `Transport.tsx`, and `LogoMark.tsx`.
   - Use high-resolution branding assets (`BeatFlow-Banner.png`, `Logo.png`, app icons).
4. **Visual States & Parity:**
   - Compare old and new interfaces at identical window sizes (e.g., 1280x800, 1440x900, 1920x1080).
   - Verify every state: normal, hover, pressed, active, focus, loading (`BeatLoader`), buffering, error, empty.

---

## C. Rust Engineering Rules

1. **Modular Architecture:** Organize into clear, cohesive subsystems:
   - `core`: State management, audio playback engine, queue, and library services.
   - `services`: Music providers (YouTube Music Innertube, Addons, Spotify, Lyrics).
   - `audio`: Native audio output and decoding pipeline (FLAC, AAC, MP3, OPUS, WAV, Atmos).
   - `platform`: Windows OS integrations (SMTC, media keys, system tray, single instance, deep links).
   - `ui`: Slint window adapter, models, event dispatchers, and state synchronizers.
2. **Separation of Concerns:** Keep UI definitions (`.slint`) declarative. Never embed complex business logic or heavy calculations into Slint markup expressions.
3. **Thread Safety & Async Runtime:**
   - Use Tokio for async network requests, disk I/O, and background tasks.
   - Never block the UI thread. Expensive operations run asynchronously and dispatch updates to the UI via `slint::invoke_from_event_loop`.
4. **Memory & Performance:**
   - Avoid unnecessary allocations and cloning.
   - Use explicit error handling with `anyhow::Result` or typed domain error enums.
   - Forbid `unwrap()` and `expect()` in production paths.
5. **No Global Mutable State:** Use structured dependency injection, message channels, or synchronized types (`Arc<RwLock<T>>`, `Arc<Mutex<T>>`).

---

## D. Slint Implementation Rules

1. **Component Modularity:** Decompose interfaces into clean, reusable Slint components under `ui/components/`, `ui/pages/`, and `ui/player/`.
2. **Typed Rust-to-Slint Interfaces:**
   - Expose clean structs and models (`slint::ModelRc`, `slint::SharedString`, `slint::Image`).
   - Define explicit callbacks for UI actions (`play_clicked()`, `seek_to(f32)`, `search(string)`).
3. **Animation Parity:**
   - Preserve original timing curves and durations (e.g. cubic-bezier `(0.215, 0.61, 0.355, 1)` with `300ms`).
   - Use Slint `animate <property> { duration: ...; easing: ...; }` and state transitions.
   - Recreate the mini-player pill to fullscreen modal transformation.
4. **Rendering Efficiency:**
   - Avoid redundant UI re-renders and excessive model rebuilds.
   - Cache decoded image textures and thumbnails appropriately.

---

## E. Asset Preservation Rules

1. **Zero Asset Loss:** Do not delete or degrade original graphics, icons, or audio assets.
2. **Reuse Original Assets:** Carry over `web/app/src/assets/fonts/`, `web/app/src/assets/logo-mark.svg`, `Logo.png`, `BeatFlow-Banner.png`, and icons from `desktop/src-tauri/icons/`.
3. **Pixel-Perfect Vector Graphics:** Replicate exact SVG paths and viewBox dimensions for all navigation and transport icons.

---

## F. Testing & Verification Rules

1. **Compilation Gate:** Every phase must compile cleanly without regressions (`cargo check`).
2. **Clippy Quality:** Code must pass `cargo clippy` without critical issues.
3. **Automated Unit Tests:** Port and preserve existing test suites (`hires::tests`, `http_tests`, queue tests).
4. **Visual Verification:** Compare visual layout against the baseline application across all viewports and themes.
5. **Functional Verification:** Verify real audio playback, scrubbing, volume adjustments, addon resolution, search queries, lyrics synchronization, downloads, and disk persistence.

---

## G. Git & Workflow Rules

1. **Branch Isolation:** All migration work occurs on `feat/native-rust-slint-migration`.
2. **Local Identity:** Commits must use:
   - `user.name`: `Abhi Devkota`
   - `user.email`: `abhidevkota1234@gmail.com`
3. **Conventional Commits:** Every commit must follow Conventional Commits format (e.g. `feat(...)`, `refactor(...)`, `fix(...)`, `chore(...)`, `test(...)`).
4. **No Destructive Operations:** Never force-push, rewrite history, or discard user work.
5. **No AI Trailers:** Forbid automated co-author or AI attribution trailers.

---

## H. Documentation Rules

1. Maintain updated progress records in `docs/migration/PROGRESS.md`.
2. Document all architectural decisions and module mappings.
3. Record visual and functional parity audits in `docs/migration/VERIFICATION_REPORT.md`.
