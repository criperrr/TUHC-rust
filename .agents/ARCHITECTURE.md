# TUHC-rust Architecture & Subsystem Specification

Technical architecture specification for TUHC-rust: the ultra-lightweight Tauri 2.0 re-implementation of The Unofficial Homestuck Collection.

---

## 1. Executive System Overview

TUHC-rust replaces the monolithic, memory-heavy Electron runtime with a high-performance native desktop architecture:
- **Core Engine:** Tauri 2.0 with Rust 2021 backend.
- **IPC & Loopback Architecture:** Dual interface featuring native Tauri invoke commands for desktop integration and an Axum HTTP loopback server (`127.0.0.1:0`) for zero-latency asset streaming, range-request video/audio playback, and Webview compliance.
- **Virtual File System (VFS):** Overlay filesystem resolving canonical assets from `Asset Pack V2` with layered community mods (e.g. `Homestuck PT-BR`).
- **In-Memory Story & Music Archive:** Instant zero-allocation querying for 9,991 story pages (`mspa.json`) and 797 music tracks (`music.json`).
- **Memory Footprint:** Resident Set Size (RSS) under 100MB in standard operation, strictly gated below 150MB.

---

## 2. Layered Component Architecture

```
┌──────────────────────────────────────────────────────────┐
│              Webview Layer (WebKitGTK / WebView2)         │
│  - Vue / Vanilla JS Shell (src/index.html)               │
│  - Ruffle Flash Player (WebAssembly)                      │
│  - Custom Audio Sync Engine & Calibration Tables         │
└──────────────┬─────────────────────────────┬─────────────┘
               │ HTTP (Range/Assets)         │ Tauri IPC
               ▼                             ▼
┌──────────────────────────────┐ ┌─────────────────────────┐
│     Axum Loopback Server     │ │    Tauri Commands       │
│  - Ephemeral port (127.0.0.1)│ │  - Window & Tab control │
│  - Range request streaming   │ │  - App metadata         │
│  - Permissive CORS layer     │ │  - Mod toggles & Config │
└──────────────┬───────────────┘ └───────────┬─────────────┘
               │                             │
               ▼                             ▼
┌──────────────────────────────────────────────────────────┐
│              Rust Core Engine (src-tauri)                │
│                                                          │
│  ┌────────────────────┐         ┌─────────────────────┐  │
│  │     VFS Engine     │         │   Archive Store     │  │
│  │ - Asset Pack V2    │         │ - mspa.json (9,991) │  │
│  │ - Community Mods   │         │ - music.json (797)  │  │
│  │ - Tree Overlays    │         │ - In-memory Patches │  │
│  └────────────────────┘         └─────────────────────┘  │
│                                                          │
│  ┌────────────────────┐         ┌─────────────────────┐  │
│  │    Mod Registry    │         │   Config Manager    │  │
│  │ - mod.js tokenizer │         │ - config.json       │  │
│  │ - translation.json │         │ - Tab persistence   │  │
│  └────────────────────┘         └─────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## 3. Core Subsystems

### 3.1. Virtual File System (VFS)
The VFS abstracts physical disk assets into a unified logical namespace:
- **Base Layer:** Asset Pack V2 root (containing `/storyfiles/`, `/archive/`, `/assets/`, etc.).
- **Mod Overlay Layer:** Active mods declare virtual mapping trees in `mod.js`:
  ```javascript
  trees: {
    "./Assets/": "assets://",
    "./storyfiles/": "assets://storyfiles/"
  }
  ```
- **Resolution Strategy:**
  1. Incoming URI `/assets/storyfiles/hs2/00002.gif` is sanitized.
  2. The VFS checks all active mod descriptors in reverse registration order.
  3. If an overlay file exists, it is returned immediately.
  4. If not found in any mod, the VFS falls back to the canonical Asset Pack V2 root.
  5. If absent from base, returns HTTP 404 Not Found.

### 3.2. Axum Streaming & Range Requests
Media files (audio tracks `.mp3`, flash archives `.swf`, videos) require byte-range streaming:
- Supports `Range: bytes=start-end` headers.
- Emits standard `206 Partial Content` with `Content-Range`, `Content-Length`, and `Accept-Ranges: bytes`.
- Uses streaming chunk buffers to prevent loading 50MB+ SWF files entirely into RAM.

### 3.3. In-Memory Store & Dynamic Translation Patching
- `StoryStore` loads `mspa.json` on startup.
- When community translation mods are enabled (e.g. `translation2.json`), translation diffs are applied directly to in-memory story page models (`StoryPage.title`, `StoryPage.content`).
- Disk assets remain untouched and pure.

### 3.4. Memory Management & Webview Optimization
- Linux WebKitGTK settings use `webkit2gtk::CacheModel::DocumentViewer` to prevent aggressive disk and memory caching of visited story pages.
- Memory trim calls via `libc::malloc_trim(0)` return freed heap chunks to the OS kernel.

### 3.5. Discord Rich Presence (RPC) Subsystem
- **Cross-Platform IPC:** Connects asynchronously to Discord's local IPC socket (`/tmp/discord-ipc-0` or `$XDG_RUNTIME_DIR/discord-ipc-0` on Linux/macOS; `\\.\pipe\discord-ipc-0` on Windows).
- **Graceful Fault Tolerance:** Operates in a dedicated non-blocking background task. If Discord is closed or unreachable, it silently enters exponential backoff retry without blocking UI rendering or Axum server streaming.
- **Reading Presence:** Displays current Act/Sub-act, page number (`Page 003890 of 008128`), page title (`Dave: Ascend`), story icon asset, and elapsed reading time.
- **Music Presence:** Displays active music album, track title, and playback state when using the in-app sound test / music player.
- **Privacy & Settings Control:** Configurable via `settings.json` (`enableDiscordRpc: bool`), allowing users to toggle Rich Presence on/off in real-time.

---

## 4. Dual-Track Testing Architecture

Tests operate under a Dual-Track approach verified against reference oracles:
1. **Tier 1 (Core Features):** Validates happy-path contracts across VFS, store, config, IPC, and static assets.
2. **Tier 2 (Boundary & Security):** Verifies resilience against directory traversal (`../`), null-bytes, invalid ranges, and missing assets.
3. **Tier 3 (Combinations):** Validates pairwise interaction between mod overlays, translations, and persistent config states.
4. **Tier 4 (Real-World Journeys):** Simulates end-to-end reader journeys, multi-page sequential reading, audio sync delays, and tab restorations.
