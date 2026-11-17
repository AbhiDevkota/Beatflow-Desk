# BitChord Web

The web player for BeatFlow. The plan, and the decisions behind it, are in [`docs/web-app-build-prompt.md`](../docs/web-app-build-prompt.md).

The companion browser extension is gone: the desktop .exe (Tauri + Rust engine) is the only client with a bridge. The web build stays as the UI the .exe ships, and as a read-only preview on the web.

| Package | What it is |
|---|---|
| `app/` | The site: Vite, React 19, TypeScript, React Router, TanStack Query, Zustand, Tailwind CSS v4 |
| `shared/` | The page ↔ host bridge contract and client, used by the app |
| `docs/` | `apple-music-dom.md` (the design measurements), `kira-notes.md` (how the reference YouTube web client plays), `youtube-playback.md`, `youtube-signin.md`, `queue-and-player.md`, `lyrics.md`, `spotify-and-discord.md` |
| `fixtures/` | Real captured responses for parser tests (see its README) |
| `scripts/` | `capture-fixtures.mjs` |

## Setup

Node 22 or newer. pnpm comes through corepack, so nothing needs a global install:

```bash
corepack pnpm --dir web install
```

## Run the site

```bash
corepack pnpm --dir web/app dev
```

It serves on `http://localhost:5173`.

## Configuration

| Variable | Where | Default |
|---|---|---|
| `VITE_ANDROID_APP_URL` | app | The GitHub releases page |
| `VITE_PARTY_SERVER_URL` | app (`app/.env.local`, Pages env) | None. The Listen Together server; `vite build` fails without it |

## Deploy (Cloudflare Pages)

| Setting | Value |
| --- | --- |
| Root directory | `web` |
| Build command | `pnpm build` |
| Output directory | `app/dist` |
| Environment | `NODE_VERSION=22`, `VITE_PARTY_SERVER_URL` |

## Checks

```bash
corepack pnpm --dir web -r typecheck
corepack pnpm --dir web/app build
```