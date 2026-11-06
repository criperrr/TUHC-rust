# CURRENT.md - Project State & Navigation Index

Executive summary and direct file index for token-efficient agent navigation in TUHC-rust. Read this file first on every task.

---

## 1. Executive Summary

**Project Title:** TUHC-rust (The Unofficial Homestuck Collection - Tauri 2.0 / Rust Re-write)  
**Active Status:** Wayland DMA-BUF Hardware Acceleration & Scroll/Battery Remediation Complete on `perf/battery-scroll-optimization`.  
**Quality Gate Status:** All verification gates passing 100% (Compilation, Formatting, Clippy, 30 Rust Tests, 3 Adversarial Integration Tests, 68 Python E2E Contract Tests, Memory RSS < 150MB SLA) + Frontend Visual & Subsystem E2E passing 9/9 in 11.17s (11 visual screenshots captured).

**Key Achievements:**
- **WebKitGTK DMA-BUF Zero-Copy Presentation & Hardware Acceleration:** Removed legacy `WEBKIT_DISABLE_DMABUF_RENDERER=1` in `src-tauri/src/main.rs`. On Linux Wayland (AMDGPU/Mesa), this was forcing WebKitGTK 2.54 onto a CPU software shared-memory (shm) blitting path, saturating the GLib main loop (5,300+ `clock_gettime` calls/sec) and pegging CPU at ~40% (25% UIProcess + 18% WebProcess) with 35W APU power draw. Restored `HardwareAccelerationPolicy::Always` and `set_enable_smooth_scrolling(true)` in `src-tauri/src/lib.rs` for butter-smooth 60–144Hz scroll with near-zero CPU.
- **Desktop Window Container Architecture (Zero Root Scroll Invalidation):** Replaced document root scrolling (`html, body { overflow: hidden; height: 100%; }`, `#app { overflow: hidden; height: 100%; }`) with desktop-native flex container. Transformed `.appHeader` to `position: relative` (eradicating GPU layer transform recalculation per wheel delta), and assigned internal scrolling strictly to `.tabFrame { flex: 1 1 auto; min-height: 0; overflow-y: auto; overscroll-behavior-y: contain; contain: paint layout; }`. Added `min-height: 0; contain: paint layout;` to eliminate recursive flex recalculations on child elements.
- **Developer Experience & Cargo Run Convenience:** Added root `package.json` with scripts (`dev`, `build`, `test`), configured `default-run = "tuhc"` in `src-tauri/Cargo.toml` and `default-members = ["src-tauri"]` in root `Cargo.toml`. The user can now simply run `cargo run`, `npm run dev`, or `pnpm dev`.
- **Diagnostic HUD Loop Throttling & Filter Eradication:** Replaced heavy `backdrop-filter: blur(4px)` with high-contrast opaque background (`rgba(0, 15, 5, 0.95)`), and stopped the `requestAnimationFrame` 144Hz polling loop entirely whenever HUD is minimized or hidden.
- **Route Scroll Reset Hook:** Automatically resets `viewport.scrollTop = 0` on every route change in `app.js`.
- **Adversarial Audit & Attack Hardening of Frontend E2E Runner:** Implemented signal-safe child process cleanup on `SIGINT`/`SIGTERM` (eliminating orphaned server zombies), port validation, pre-flight target server probing, hermetic temporary `XDG_CONFIG_HOME` isolation, and stale report eviction.
- **PT-BR Translation Mod Alignment & Resilient Config Persistence:** Harmonized `save_config` IPC deserializer; synchronized `modListEnabled` and `enabledMods`; confirmed 2,401 pages translated in memory with Portuguese titles (`Insira o nome.`, `Tente novamente.`).
- **Deterministic Theme Engine Across All Views:** Unified theme attribution across `html`, `body`, `#app`, and `#page-container`; eliminated hardcoded `applyTheme('mspa')` on system views.

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
- `tests/frontend_e2e/run.py`: Master launcher for Frontend Visual & Subsystem E2E test suite (spawns Axum loopback server, executes Playwright runner).
- `tests/frontend_e2e/e2e_runner.js`: Playwright test runner verifying 9 core subsystems and capturing 11 screenshots.
- `tests/frontend_e2e/README.md`: Technical architectural comparison (@tauri-apps/wdio-tauri-service vs Playwright vs WebKitWebDriver).
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
