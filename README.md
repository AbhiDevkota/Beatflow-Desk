<div align="center">

<img src="BeatFlow-Banner.png" alt="BeatFlow banner" width="100%" />

<br/>

# BeatFlow

### A modern YouTube Music client

<br/>

[![License](https://img.shields.io/github/license/AbhiDevkota/BeatFlow?style=for-the-badge&labelColor=0d1117)](LICENSE)

<br/>

[**Download**](#download) · [**Features**](#features) · [**About**](#about) · [**Disclaimer**](#disclaimer)

</div>

> [!IMPORTANT]
> BeatFlow is not affiliated with, endorsed by, or connected to YouTube or Google. Use it at your own discretion.

---

<div align="center">

<h1><a id="features"></a>Features</h1>

<table>
  <tr>
    <td width="50%" valign="top">

#### Playback & Hi-Res
- Search, browse, and play music from YouTube Music
- Hi-Res Lossless (24-bit/192kHz) and Dolby Atmos stream auto-upgrades
- Gapless playback and low-latency audio engine via Rodio
- Time-synchronized lyrics with active line tracking (LRC & TTML)
- Local music library integration & offline downloads with embedded ID3/MP4 metadata

#### Connectivity
- Direct YouTube Music Innertube API integration
- Deep-linking with custom URI scheme (`beatflow://`)
- Native Windows media keys (Play/Pause, Next, Prev, Stop)
- Single-instance activation and IPC routing over loopback TCP

    </td>
    <td width="50%" valign="top">

#### Experience & Native Architecture
- **100% Native Rust + Slint GUI**: Zero Electron, Zero Tauri, Zero Chromium WebView2
- Instant cold start (~115 ms) and minimal memory footprint (~24 MB RAM)
- Apple Music web player design language and frameless glassmorphic UI
- Smooth animations: 420ms spring fullscreen FLIP expansion, ambient glow backdrop, and animated scrubber
- Up Next queue timeline with Shuffle and Repeat modes
- 8 structured preference sections for audio quality, network, and storage

#### Desktop Integrations
- Frameless window titlebar with native DWM window dragging
- Native minimize, maximize/restore, and close controls
- Global keyboard and multimedia key shortcuts

    </td>
  </tr>
</table>

</div>

---

<div align="center">

<h1><a id="building"></a>Building & Running</h1>

</div>

### Prerequisites
- [Rust](https://rustup.rs/) (version 1.90.0 or later recommended)
- Windows 10/11 (or Linux/macOS with standard audio libraries)
- Git

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/AbhiDevkota/BeatFlow.git
cd BeatFlow

# Run the native desktop application in development mode
cargo run -p beatflow

# Build an optimized, standalone native release binary
cargo build --release -p beatflow

# Run all test suites
cargo test -p beatflow
```

The resulting standalone executable will be located at:
```
target/release/beatflow.exe
```

---

<div align="center">

<h1><a id="architecture"></a>Architecture</h1>

</div>

BeatFlow is engineered with a modular Rust + Slint desktop architecture:

- **`ui/`**: Slint declarative design definitions (`.slint`), containing design tokens, components (player bar, titlebar, LCD display, fullscreen player, side panel), and views (home, search, downloads, settings).
- **`src/ui/`**: Slint component compilation, thread-safe UI bridge (`UiBridge`), and Slint model adapters.
- **`src/core/`**: Core music application logic, including:
  - `playback/`: Native Rodio audio output engine, DASH stream parsing, and Hi-Res Lossless / Dolby Atmos matching algorithms.
  - `queue/`: Queue timeline state machine with repeat, shuffle, and index jumping.
  - `lyrics/`: Multi-provider lyrics aggregator, LRC / TTML parsing, and playhead synchronization clock.
  - `downloads/`: Background downloader, progress tracking, and ID3v2 / MP4 audio tagging.
  - `app_coordinator.rs`: Event coordinator linking UI events with asynchronous Tokio background workers.
- **`src/platform/`**: OS integrations including Windows global media hotkeys, `beatflow://` registry URI scheme handlers, and single-instance IPC loopback.
- **`src/services/`**: Network communication, YouTube Music (Innertube) API client, and authentication management.

---

---

<div align="center">

<h1><a id="download"></a>Download</h1>

Get the latest available build from the [Releases](https://github.com/AbhiDevkota/BeatFlow/releases) page.

</div>

---

<div align="center">

<h1><a id="about"></a>About</h1>

BeatFlow is a forked adaptation of the original BitChord project. Thank you to the BitChord team for creating the framework and foundation this project builds on.

</div>

---

<div align="center">

<h1><a id="disclaimer"></a>Disclaimer</h1>

BeatFlow is an independent third-party client. YouTube and Google are trademarks of their respective owners. Use of this software must comply with applicable terms and laws.

</div>

---

<div align="center">

<h1><a id="license"></a>License</h1>

This project is licensed under the GNU General Public License v3.0 (GPLv3). See the [LICENSE](LICENSE) file for details.

</div>
