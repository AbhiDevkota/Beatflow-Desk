# Complete Animation & Interaction Inventory

This document catalogues all animation behaviors, transition curves, timings, and micro-interactions in BeatFlow. Replicating these in Slint is critical to preserving the tactile feel and visual polish of the application.

---

## 1. Mini-Player to Fullscreen Player FLIP Transformation

| Attribute | Specification |
| :--- | :--- |
| **Origin Element** | Mini-player pill artwork thumbnail (`#miniplayer-artwork`, `34x34px` at bottom center) |
| **Destination Element**| Fullscreen player large artwork (`[--art:min(38vw,560px)]` centered or left-aligned) |
| **Transition Type** | FLIP (First-Last-Invert-Play) geometry morph |
| **Duration** | `340ms` (expansion), `280ms` (collapse) |
| **Easing Curve** | Custom cubic bezier: `cubic-bezier(0.2, 0.0, 0.0, 1.0)` (iOS / macOS modal curve) |
| **Associated Elements**| Scrubber line, transport controls, and track title morph outward simultaneously |
| **Backdrop Expansion** | Dark mesh backdrop fades from `0` to `1` opacity concurrently (`duration: 300ms`) |
| **Reverse Collapse** | Triggered by minimize button (`startCollapse`); reverses geometry back into pill coordinates |

---

## 2. Shared Artwork & Ambient Mesh Backdrop

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Ambient Mesh Crossfade** | `700ms` | `ease-in-out` | Previous track blurred backdrop fades out while new track blurred backdrop (`blur: 84px`, `saturate: 1.65`) fades in |
| **Artwork Change Crossfade**| `300ms` | `ease-in` | Album cover smoothly crossfades between tracks to avoid visual flashing |
| **Thumbnail Hover Bloom** | `200ms` | `ease-out` | Mini-player artwork dims slightly to reveal the centered `Maximize2` expand icon |
| **Card Artwork Hover Scale**| `180ms` | `ease-out` | Album/artist card covers scale up slightly (`scale(1.03)`) on pointer hover |

---

## 3. Side Panel Navigation & Slide Transitions

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Panel Slide In** | `300ms` | `cubic-bezier(0.215, 0.61, 0.355, 1)` | Side panel slides from right (`translateX(100%)` to `translateX(0)`) |
| **Panel Slide Out** | `250ms` | `ease-in` | Side panel slides off-screen right |
| **Player Pill Shift** | `200ms` | `ease-out` | Mini-player pill re-centers smoothly when side panel opens (`margin-right: 320px`) |
| **Tab Switch Crossfade** | `150ms` | `ease-in-out` | Content crossfades when switching between Lyrics and Up Next tabs |

---

## 4. Scrubber & Progress Bar Micro-Interactions

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Scrubber Track Expansion**| `100ms` | `ease-in` | Scrubber line height increases from `2px` to `4px` when hovered or dragged |
| **Timestamp Reveal** | `120ms` | `ease-in` | Elapsed and remaining timestamps fade in above the scrubber line ends |
| **Progress Fill Update** | Real-time | Linear | Fill width scales continuously in sync with audio engine playhead position |
| **Drag Snap** | Instant | - | Seeking while dragging updates visual playhead immediately without waiting for audio engine seek |

---

## 5. Volume Control Animation

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Volume Slider Expansion**| `200ms` | `ease-out` | Slider capsule unfolds smoothly on pointer enter over volume icon |
| **Action Icons Fade** | `150ms` | `ease-in` | Sibling buttons (Favorite, Menu, Lyrics) fade to `0.1` opacity when volume is expanded |
| **Slider Fill Animation** | `300ms` | `ease-out` | Volume bar level animates smoothly on scroll-wheel or click adjustments |

---

## 6. Real-Time Lyrics Synchronized Animations

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Active Line Magnification**| `200ms` | `ease-out` | Current lyric line scales to full white (`1.0` opacity) and larger type size |
| **Neighbor Line Fade** | `200ms` | `ease-out` | Adjacent lines dim progressively (`0.7`, `0.58`, `0.46` opacity) |
| **Smooth Lyric Scroll** | Continuous | `cubic-bezier(0.25, 0.1, 0.25, 1.0)` | Lyric list scrolls continuously to keep current line at 30% viewport height |
| **Syllable Wipe (TTML)** | Duration of word | Linear | Gradient text fill wipes horizontally across individual syllables in sync with audio |

---

## 7. Transport Button Micro-Interactions

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Button Press Depression** | `100ms` | `ease-in` | Transport buttons depress slightly (`scale(0.92)`) when clicked/pressed |
| **Equalizer Loading Bars** | Loop | `ease-in-out` | `BeatLoader` 3-bar jumping equalizer while audio stream is buffering |
| **Favorite Star Pop** | `180ms` | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` | Star icon scales up (`scale(1.2)`) and pops into filled red/gold state |

---

## 8. Dynamic Background Tinting

| Animation | Duration | Easing | Description |
| :--- | :--- | :--- | :--- |
| **Page Tint Transition** | `300ms` | `ease-in` | Background and accent colors transition smoothly when navigating to an album with extracted dominant artwork palette |
| **Skeleton Shimmer** | `1500ms` loop | Linear | Continuous light sweep across placeholder blocks during data fetches |
