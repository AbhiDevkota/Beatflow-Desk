# BeatFlow Web Frontend

The React frontend for BeatFlow Desktop, stored inside `desktop/src-tauri/web/`.

| Package / Folder | What it is |
|---|---|
| `app/` | The UI: Vite, React 19, TypeScript, React Router, TanStack Query, Zustand, Tailwind CSS v4 |
| `shared/` | Shared bridge contract and client used by the app |
| `docs/` | `apple-music-dom.md` (design measurements), `kira-notes.md`, `youtube-playback.md`, `youtube-signin.md`, `queue-and-player.md`, `lyrics.md`, `spotify-and-discord.md` |
| `fixtures/` | Captured responses for parser tests (see its README) |
| `scripts/` | `capture-fixtures.mjs` |

## Setup

Node 22 or newer. pnpm comes through corepack, so nothing needs a global install:

```bash
corepack pnpm --config.pm-on-fail=ignore --dir desktop/src-tauri/web install
```

## Run the frontend standalone

```bash
corepack pnpm --config.pm-on-fail=ignore --dir desktop/src-tauri/web/app dev
```

It serves on `http://localhost:5173`, matching `devUrl` in `desktop/src-tauri/tauri.conf.json`.

## Checks

```bash
corepack pnpm --config.pm-on-fail=ignore --dir desktop/src-tauri/web -r typecheck
corepack pnpm --config.pm-on-fail=ignore --dir desktop/src-tauri/web -r test
corepack pnpm --config.pm-on-fail=ignore --dir desktop/src-tauri/web/app build
```
