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

### 3.5. Global Discord Rich Presence (RPC)
A native, zero-dependency Discord Local IPC client is integrated into the backend runtime (`rpc::DiscordRpcManager`):
- **Cross-Platform Socket Discovery:** On Unix (`Linux`/`macOS`), candidate Unix domain sockets are queried in parallel across `$XDG_RUNTIME_DIR/discord-ipc-0..9`, Flatpak (`app/com.discordapp.Discord`), Snap (`snap.discord`), `/run/user/$UID/`, and `$TMPDIR`/`/tmp/discord-ipc-0..9` with a strict 150ms timeout (`\\.\pipe\discord-ipc-0` on Windows).
- **Binary Wire Protocol Framing:** Messages adhere to Discord's binary framing format `[opcode: u32 LE, length: u32 LE, json_bytes]`, supporting `Handshake` (`op=0`) and `SET_ACTIVITY` (`op=1`).
- **MSPA Chapter / Act Resolution:** A canonical chapter and icon key resolver (`resolve_story_chapter`) covers all MSPA pages (Jailbreak, Bard Quest, Problem Sleuth Chapters 1–22, Homestuck Beta, Acts 1–7, Openbound, Collide, and Ryanquest).
- **Dual Presence Modes:**
  - *Reading Presence:* Tracks active chapter, formatted page title (`"Homestuck (Page 001901)"` or localized `"Insira o nome. (Page 001902)"`), story icon, and elapsed reading session timestamp. Triggered automatically on story page retrieval when `enable_discord_rpc` is active.
  - *Music Presence:* Displays current track title, album, artist, and playback status (`play`/`pause` small icon).
- **Graceful Offline Fallback & Fault Tolerance:** If Discord is closed or unreachable, the RPC manager records the latest activity state in memory without blocking, timing out, or failing IPC/HTTP endpoints. Offline status is transparently reported via `get_discord_status`.
- **Privacy & Settings Control:** Configurable via `config.json` (`enable_discord_rpc: bool`), allowing users to toggle Rich Presence on/off in real-time.

---

## 4. Dual-Track Testing Architecture

Tests operate under a Dual-Track approach verified against reference oracles:
1. **Tier 1 (Core Features):** Validates happy-path contracts across VFS, store, config, IPC, and static assets.
2. **Tier 2 (Boundary & Security):** Verifies resilience against directory traversal (`../`), null-bytes, invalid ranges, and missing assets.
3. **Tier 3 (Combinations):** Validates pairwise interaction between mod overlays, translations, and persistent config states.
4. **Tier 4 (Real-World Journeys):** Simulates end-to-end reader journeys, multi-page sequential reading, audio sync delays, and tab restorations.
