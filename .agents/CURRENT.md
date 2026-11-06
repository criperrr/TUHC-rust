# CURRENT.md - Project State & Navigation Index

Executive summary and direct file index for token-efficient agent navigation in TUHC-rust. Read this file first on every task.

---

## 1. Executive Summary

**Project Title:** TUHC-rust (The Unofficial Homestuck Collection - Tauri 2.0 / Rust Re-write)  
**Active Status:** Milestones 2–4 Complete & Gate APPROVED — Implementation on `feat/m2-m4-implementation`.  
**Quality Gate Status:** All 5 verification gates passing 100% (Reviewers, Challengers, Forensic Auditor).

**Key Achievements:**
- **Frontend Webview & Visual Parity (M2):** Authentic MSPA chrome with tabs, retro address bar, bookmarks persistence, sound test bar, 13 canonical themes, Courier typography, character quirk colors, and StoryPage rendering (`src/index.html`, `src/styles/main.css`, `src/js/app.js`).
- **Media Engine & Ruffle Flash Sync (M3):** Self-hosted Ruffle WASM embedding inside srcdoc iframe, Frag IPC ActionScript bridge, millisecond audio calibration tables (`src/js/flashProps.js`), Cascade sub-segment offsets, Collide dynamic video curtain, and Sburb HTML5 Canvas support (`src/js/mediaEngine.js`).
- **Community Mod Engine & Discord RPC (M4):** Legacy `mod.js` parser for single-file and directory mods, O(1) VFS overlay with generation tracking, native translation patch merging, VueHooks integration (`src/js/modHooks.js`), and cross-platform Discord Local IPC Rich Presence (`src-tauri/src/rpc/mod.rs`).
- **Integration Seam Hardening:** Resolved ActionScript Frag IPC `#audioSeek&<ms>` parameter parsing, `renderMedia` boolean contract, direct settings coercion, and iframe DOM detachment for memory hygiene.
- **Ultra-lightweight Memory Footprint:** Server RSS measured at ~62MB (headroom of >87MB under 150MB SLA ceiling).
- **Dual-Track & Checklist Verification:** 6/6 checklist checks passed, 31/31 Rust cargo tests passed, 68/68 Python E2E contracts passed, 17/17 adversarial media tests passed.

---

## 2. Navigation Index

### 2.1. Core Engine (`src-tauri/src/`)
- `lib.rs`: Tauri builder, WebKitGTK DocumentViewer memory configuration, IPC command registrations.
- `main.rs`: Desktop runtime entrypoint.
- `commands.rs`: Tauri IPC invoke commands (`get_app_info`, `get_story_page`, `load_config`, `save_config`, `get_available_mods`, `toggle_mod`, `trim_memory`).
- `server/mod.rs`: Axum loopback server initialization, REST IPC mirror routes (`/ipc`, `/ipc/{cmd}`).
- `server/vfs.rs`: Virtual File System engine, path traversal sanitizer, mod tree overlay resolver.
- `server/handler.rs`: Asset streaming handler, MIME detection, HTTP 206 Partial Content Range parser.
- `data/mod.rs`: `ArchiveStore` combining StoryStore and MusicStore.
- `data/story.rs`: `StoryStore` loading and querying `mspa.json` (9,991 pages) with dynamic patch applicator.
- `data/music.rs`: `MusicStore` loading and querying `music.json` (797 tracks, 37 albums).
- `config/mod.rs`: `AppConfig` persistence with atomic tempfile replacement, `config.json` serialization, tab management.
- `mods/mod.rs`: `ModRegistry` managing installed and enabled community mods.
- `mods/parser.rs`: Regex-based declarative parser for `mod.js` and patch loader for `translation2.json`.
- `rpc/mod.rs`: Native asynchronous Discord Local IPC manager, MSPA chapter resolution, Reading/Music presence, and Unix domain socket client.
- `bin/server.rs`: Standalone contract server binary for CI and test harnesses.

### 2.2. Frontend Shell (`src/`)
- `index.html`: Shell loading Webview UI, Ruffle Flash player WASM, and loopback communication.
- `styles/main.css`: 13 authentic MSPA themes, Courier typography, pesterlog quirks, and 950px frame.
- `js/app.js`: Tabbed browser navigation, URL router, bookmark persistence, settings, and sound test.
- `js/flashProps.js`: Millisecond audio delay tables, negative seek offsets, crop heights, and transitions.
- `js/mediaEngine.js`: Ruffle WASM iframe loader, Frag IPC bridge, HTML5 audio/video, and Sburb canvas.
- `js/modHooks.js`: VueHooks evaluator, rate-limited console sandbox, English & PT-BR button text.

### 2.3. Test Infrastructure (`tests/` & `src-tauri/tests/`)
- `tests/e2e_runner.py`: Unified test execution engine.
- `tests/mock_server.py`: Reference contract server oracle.
- `tests/common.py`: Shared test client, assertion helpers, and asset validation.
- `tests/tier1_features/`: Happy-path feature contract tests.
- `tests/tier2_boundaries/`: Error handling, 404s, malformed ranges, path traversal security.
- `tests/tier3_combinations/`: Cross-subsystem interaction tests (mods + story + persistence).
- `tests/tier4_realworld/`: End-to-end user journeys (sequential reading, Flash audio delay sync).
- `src-tauri/tests/adversarial_config_concurrency.rs`: High-concurrency atomic save stress test.
- `src-tauri/tests/adversarial_mod_toggle_stress.rs`: Rapid concurrent mod toggle and VFS test.
- `src-tauri/tests/adversarial_store_keys.rs`: Non-numeric and boundary key lookups.

### 2.4. Governance & CI/CD
- `GATE_STATUS.md`: Quality gate verification matrix and audit sign-off.
- `.agents/AGENTS.md`: Master governance rules, branch axiom, commit policies, Rust conventions.
- `.agents/ARCHITECTURE.md`: Technical architecture specification.
- `.agents/DESIGN.md`: Architecture Decision Records (ADRs).
- `.agents/rules/`: Granular behavioral rules for agents.
- `.agents/agent/`: 11 specialized agent personas for desktop/Rust tasks.
- `.agents/scripts/checklist.py`: Fast 6-step validation checklist.
- `.agents/scripts/memory_check.py`: Hardened memory footprint gatekeeper (< 150MB RSS).
- `.agents/scripts/session_manager.py`: Stack detector and module analyzer.
- `.agents/scripts/verify_all.py`: Full master verification pipeline.
- `.github/workflows/pr-gatekeeper.yml`: Strict PR checks (branch naming, commit syntax, clippy, test, E2E, memory).
- `.github/workflows/staging.yml`: Validation and build on `dev` branch.
- `.github/workflows/release.yml`: Release build and packaging for Linux desktop.
- `.github/workflows/nightly-audit.yml`: Nightly dependency vulnerability and memory audit.
