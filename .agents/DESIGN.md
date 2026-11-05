# DESIGN.md - Technical Design Decisions for TUHC-rust

This document records the foundational architectural decisions, trade-offs, and design rationale behind TUHC-rust.

---

## 1. Decision Log

### ADR-001: Architecture Migration from Electron to Tauri 2.0 (Rust)
- **Context:** The legacy Unofficial Homestuck Collection runs on Electron 11–18, requiring Chromium and Node.js runtimes. It frequently consumes 800MB to 1.4GB of RAM and exhibits significant startup latency.
- **Decision:** Re-architect the application in Rust using Tauri 2.0 and native Webview (WebKitGTK on Linux, WebView2 on Windows).
- **Consequences:**
  - Memory consumption drops from ~900MB to **~65MB RSS** (over 90% reduction).
  - Startup time reduced from 4.5s to < 300ms.
  - Native binary distribution without bundled browser runtimes.

---

### ADR-002: Ephemeral Loopback Axum Server vs. Custom Webview Protocol
- **Context:** Tauri allows registering custom protocols (e.g. `tauri://` or `assets://`). However, media streaming (MP3 background music, Flash SWF files, animations) requires strict `Range: bytes=start-end` handling and HTTP 206 Partial Content negotiation. WebKitGTK on Linux has documented shortcomings and buffering bugs when streaming partial content through custom URI scheme callbacks.
- **Decision:** Spawn an embedded, high-performance Axum HTTP server on an ephemeral loopback port (`127.0.0.1:0`). The webview connects to this local HTTP server for assets and audio streams.
- **Consequences:**
  - 100% standard HTTP 1.1 compliance for Range requests, MIME headers, and CORS.
  - Flawless compatibility with the Ruffle WebAssembly Flash emulator.
  - No port collision issues due to OS-assigned ephemeral ports (`port 0`).

---

### ADR-003: Virtual File System (VFS) with Multi-Tier Mod Overlay
- **Context:** Community mods (such as `Homestuck PT-BR`) override story images, dialogues, and interface graphics without altering the canonical Asset Pack V2 files.
- **Decision:** Implement a layered Virtual File System in `src-tauri/src/server/vfs.rs`. Active mods register virtual mount trees (e.g. `./Assets/` -> `assets://`). Incoming asset requests check active mod overlays in priority order before falling back to the base Asset Pack.
- **Consequences:**
  - Non-destructive modding: Asset Pack V2 remains pristine.
  - Dynamic mod toggling without restarting the desktop application.
  - Path traversal protection enforced centrally at the VFS ingress point.

---

### ADR-004: In-Memory Monolithic Archive Indexing
- **Context:** Homestuck comprises 9,991 story pages with complex next/previous links, retcons, and media references. Querying SQLite or parsing JSON per page introduces disk I/O overhead.
- **Decision:** Parse `mspa.json` and `music.json` once at startup into memory structures (`StoryStore` and `MusicStore`) backed by `HashMap<String, StoryPage>`.
- **Consequences:**
  - Microsecond page lookups ($O(1)$).
  - In-memory translation patches (e.g. `translation2.json`) can be applied cleanly over memory records.
  - Memory footprint of the entire story catalog in memory is less than 35MB.

---

### ADR-005: Sandboxed Mod Parser vs Full JS Runtime
- **Context:** Legacy mods include a `mod.js` file defining metadata, trees, and routes. Running a full Node/V8 runtime solely to parse this configuration is heavy and introduces security risks (arbitrary code execution).
- **Decision:** Implement a deterministic regex-based tokenizer and parser in Rust (`src-tauri/src/mods/parser.rs`) capable of parsing declarative `mod.js` configurations safely.
- **Consequences:**
  - Zero arbitrary code execution vulnerability from community mods.
  - High parsing speed without spinning up a JavaScript engine in the backend.

---

### ADR-006: Memory Hygiene Budget and Glibc Trimming
- **Context:** Desktop applications running on Linux often retain freed allocator heap pages in glibc, leading to artificial RSS bloat.
- **Decision:**
  - Set WebKitGTK cache model to `DocumentViewer`.
  - Disable webview offline cache and page cache.
  - Call `libc::malloc_trim(0)` during idle moments and tab closures.
  - Enforce a strict CI Gatekeeper ceiling of **< 150MB RSS**.
- **Consequences:**
  - Application maintains a lean, consistent memory footprint across long reading sessions.
