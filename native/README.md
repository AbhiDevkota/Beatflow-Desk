# BeatFlow — native shell (Slint)

A standalone Slint port of the desktop app's UI, built one component at a time
per [`.claude/NATIVE_MIGRATION_BLUEPRINT.md`](../.claude/NATIVE_MIGRATION_BLUEPRINT.md).
The React + Tauri app (`desktop/`, `web/`) is untouched and remains the source
of truth: every metric, colour, type style and icon here is copied from it.

**Status: shell skeleton.** The frameless window, glass sidebar, player pill
(with transport, LCD, volume, BeatLoader), window controls, drag strip and the
design tokens are ported. Playback is mocked — no audio engine yet (that's the
blueprint's Stage 1), and the side panel drawer / full-screen player come
later.

## Run

```powershell
cd native
cargo run
```

Rust ≥ 1.90 (edition 2024), Windows 10/11. The window opens frameless with
Windows 11 Mica behind the stage and the DWM shadow/corners applied from
`src/main.rs` (`window-shadows` can't be used here: it targets
raw-window-handle 0.5, while Slint runs with 0.6).

## Layout

| Path | Role |
| --- | --- |
| `ui/theme.slint` | Every token from `web/app/src/styles/tokens.css`, verbatim, with dark + light palettes |
| `ui/state.slint` | The shell's globals, mirroring the Zustand stores (`state/player.ts`, `state/ui.ts`) |
| `ui/icons.slint` | `AppIcon`: monochrome SVG glyphs tinted via `Image.colorize` (`currentColor`) |
| `ui/sidebar.slint` | `Sidebar.tsx` (desktop layout, signed-out mock) |
| `ui/window-controls.slint` | `WindowControls.tsx`, the Discord-style trio |
| `ui/lcd.slint` | `Lcd.tsx`: artwork, title/artist, the scrub line and its useScrubber behaviour |
| `ui/player-bar.slint` | `PlayerBar.tsx` + `VolumeControl.tsx` + `BeatLoader.tsx` |
| `ui/app-window.slint` | The frameless window: fonts, stage, drag strip, controls |
| `fonts/` | DM Sans Regular/Medium/SemiBold/Bold, bundled at compile time via `import "*.ttf"` |
| `icons/` | 33 SVGs generated verbatim from the React sources — **never retyped by hand** |
| `tools/gen-icons.mjs` | The icon generator (rerun: `node native/tools/gen-icons.mjs [lucide-icons-dir]`) |
| `src/main.rs` | Mock playback state, timers, window callbacks, Mica + DWM chrome |

## What's mocked

- **Playback**: one "Demo Track" / "Demo Artist", 3:42 (222s), driven by a
  100ms timer. Play toggles paused → buffering → playing (a 600ms single-shot
  stands in for the loader's latency); next/previous restart the track;
  repeat cycles off → all → one (all/one wrap the playhead); the scrubber
  seeks on release, exactly like `useScrubber.ts`.
- **Signed out**: no FavoriteButton (it returns `null` when signed out), the
  Google pill in the sidebar footer, no Library sections or user playlists.
- **Volume**: the slider, mute and the wheel (one notch = 0.05, rounded to
  whole percents) all work against the mock store.

## Deviations from the web app (Slint limits, all deliberate)

- **No live backdrop-filter.** The web blurs what's behind the pill and
  sidebar (`saturate(2.4) blur(28px)`); the native window trades the opaque
  page background for Windows 11 Mica behind a slightly translucent stage
  (`#1f1f1fed`), so the wallpaper blurs instead of the page content.
- **Pill shadows**: two box-shadows become two elements (the pill itself plus
  a 1/255-alpha twin), and the CSS `-4px`/`-2px` spreads can't be expressed.
- **Pill inner stroke**: the `::after` inset stroke is a uniform 1px border at
  the top edge's effective alpha (white 0.22 × 0.25 dark / 0.05 light); Slint
  borders can't vary per side.
- **BeatLoader**: each bar's `cubic-bezier(0.45,0.05,0.55,0.95)` alternate
  keyframes become a triangle wave over the same 1.9s full cycle, staggered by
  0.22s/0.95s of it; bar heights/opacities still use each keyframe's own
  0%/50%/100% range.
- **Press scale**: `active:scale-[0.98]` animates width/height instead —
  Slint has no transform scale, so buttons shrink toward their top-left
  rather than their centre.
- **Scrubber dim**: `Lcd`'s names soften to opacity 0.3 on hover, but the
  web's `blur(3px)` on them has no per-element equivalent.
- **LCD artwork**: the `joe`-colour placeholder tile; real artwork arrives
  with the queue (Stage 3).
