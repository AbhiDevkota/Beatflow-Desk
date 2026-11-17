# Native Slint Migration — Implementation Plan

Tracks the staged migration of BeatFlow to the standalone Slint crate in `native/`.
React app (`web/app`) stays the source of truth. Reference backend on branch `refrence/rust`.

## Status: [ ] not started  [~] in progress  [x] done

## Stage 0 — Shell skeleton [x]
Frameless window, Mica, sidebar, player pill, window controls, theme tokens,
animations (BeatLoader, scrubber, volume). Audio engine wired to Slint Player.

## Stage 1 — Search + Home + Explore with real data [x]
- [x] Integrate jsc (rquickjs) JS cipher module
- [x] Integrate webmusic (session/cookies)
- [x] Integrate ytmapi (Api, visitor data, call)
- [x] Integrate ytm (search, home, suggestions, album, artist, playlist)
- [x] Integrate backend types (Explore, SearchResults, Collection, ArtistPage, Library, Channels)
- [x] Add rquickjs dependency
- [x] Search page: 1:1 pill search field + 12 Apple Music CategoryTiles + Top Results cards + Songs list wired to ytm::search_songs
- [x] Wire search result click -> resolve stream -> play
- [x] Home page: 1:1 Top Picks, Quick Picks, and New Releases shelves wired to ytm::home
- [x] New / Explore page: 1:1 New Albums & Singles, Top Charts, and Featured Playlists wired to ytm::new_albums and ytm::charts

## Stage 2 — Album / Artist / Playlist pages [ ]
## Stage 3 — Library + Downloads [ ]
## Stage 4 — Side panel (queue + lyrics) [ ]
## Stage 5 — Full-screen player [ ]
## Stage 6 — Settings + platform [ ]

## Notes
- Each stage committed separately with conventional commits.
- Visual verification against web app required before marking done.
- React app never deleted; desktop/ and web/ untouched.
