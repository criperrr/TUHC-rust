# CURRENT.md - Project State & Navigation Index

Executive summary and direct file index for token-efficient agent navigation in TUHC-rust. Read this file first on every task.

---

## 1. Executive Summary

**Project Title:** TUHC-rust (The Unofficial Homestuck Collection - Tauri 2.0 / Rust Re-write)  
**Active Status:** Milestone 1 Gate APPROVED — Baseline Established on `main` & `dev`.  
**Quality Gate Status:** All 8 verification gates passing 100% (`GATE_STATUS.md`).

**Key Achievements:**
- **Core Rust Implementation:** Complete Tauri 2.0 re-implementation with in-memory story and music catalog indexing, Axum loopback asset server, streaming byte-range support, and community mod VFS overlay (`src-tauri/src/`).
- **Atomic Config Persistence:** Hardened `AppConfig::save()` with `tempfile::Builder` preventing race conditions and raw disk corruption under multi-threaded concurrency.
- **Dual-Track E2E Test Suite:** 68/68 automated tests passing across 4 tiers (Feature Coverage, Boundaries & Security, Combinations, Real-World User Journeys).
- **Ultra-lightweight Memory Footprint:** Server RSS measured at 57.34MB (headroom of 92.66MB under 150MB SLA ceiling).
- **Git & Remote Setup:** Repository initialized on branch `main`, remote origin configured to `https://github.com/criperrr/TUHC-rust.git`, and development baseline established on `dev`.
- **Governance Ecosystem:** Comprehensive governance with branch axioms, conventional commit enforcement, strict quality gatekeepers, and 11 specialized desktop agents.

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
- `bin/server.rs`: Standalone contract server binary for CI and test harnesses.

### 2.2. Frontend Shell (`src/`)
- `index.html`: Shell loading Webview UI, Ruffle Flash player WASM, and loopback communication.

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
