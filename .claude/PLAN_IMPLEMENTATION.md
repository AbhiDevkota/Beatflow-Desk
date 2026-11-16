# Native Slint Migration — Implementation Plan

Tracks the staged migration of BeatFlow to the standalone Slint crate in `native/`.
React app (`web/app`) stays the source of truth. Reference backend on branch `refrence/rust`.

## Status: [ ] not started  [~] in progress  [x] done

## Stage 0 — Shell skeleton [x]
Frameless window, Mica, sidebar, player pill, window controls, theme tokens,
animations (BeatLoader, scrubber, volume). Audio engine wired to Slint Player.

## Stage 1 — Search + Home with real data [~]
- [x] Integrate jsc (rquickjs) JS cipher module
- [x] Integrate webmusic (session/cookies)
- [x] Integrate ytmapi (Api, visitor data, call)
- [x] Integrate ytm (search, home, suggestions, album, artist, playlist)
- [x] Integrate backend types (Explore, SearchResults, Collection, ArtistPage, Library, Channels)
- [x] Add rquickjs dependency
- [x] Search page: query input + results list (ListView) wired to ytm::search_songs
- [ ] Home page: real shelves from ytm::home
- [ ] Wire search result click -> resolve stream -> play

## Stage 2 — Album / Artist / Playlist pages [ ]
## Stage 3 — Library + Downloads [ ]
## Stage 4 — Side panel (queue + lyrics) [ ]
## Stage 5 — Full-screen player [ ]
## Stage 6 — Settings + platform [ ]

## Notes
- Each stage committed separately with conventional commits.
- Visual verification against web app required before marking done.
- React app never deleted; desktop/ and web/ untouched.
