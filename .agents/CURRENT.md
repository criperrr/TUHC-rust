# CURRENT.md - Project State & Navigation Index

Executive summary and direct file index for token-efficient agent navigation in TUHC-rust. Read this file first on every task.

---

## 1. Executive Summary

**Project Title:** TUHC-rust (The Unofficial Homestuck Collection - Tauri 2.0 / Rust Re-write)  
**Active Status:** Adversarial Audit & Hardening Complete on `test/harden-frontend-e2e`.  
**Quality Gate Status:** All 6 verification gates passing 100% (Compilation, Formatting, Clippy, 30 Rust Tests, 68 Python E2E, Memory 54MB < 150MB SLA) + Frontend Visual & Subsystem E2E passing 9/9 in 9.71s (11 visual screenshots captured).

**Key Achievements:**
- **Adversarial Audit & Attack Hardening of Frontend E2E Runner:** Executed 5 attack vectors (CLI boundary fuzzing, environment variable sabotage, concurrency collisions, mid-flight process disruption, adversarial network payloads). Implemented signal-safe child process cleanup on `SIGINT`/`SIGTERM` (eliminating orphaned server zombies), port validation, pre-flight target server probing, and stale report eviction.
- **Hermetic Environment Isolation for E2E Tests:** Sandboxed the embedded Axum backend during test execution in a clean temporary `XDG_CONFIG_HOME`, isolating developer configuration (`config.json`), preventing active mod bleed (e.g. PT-BR translation), and eliminating user tab data pollution.
- **Defensive Subsystem Assertion Resilience:** Hardened `e2e_runner.js` with robust executable verification (`isExecutableFile`) for `CHROMIUM_PATH`, resilient story arrow / pesterlog assertions across languages, and optimized Sound Test album link interaction (dropping duration from 6001ms to 1549ms).
- **Scroll Performance & GPU Compositing Acceleration:** Enabled WebKitGTK smooth scrolling and hardware acceleration (`HardwareAccelerationPolicy::Always`), isolated `.appHeader` with GPU transform layer (`will-change: transform; transform: translateZ(0);`), eliminating repaints and high CPU lag during scroll.
- **Window Decorator Duplicity Resolution & Compact 2-Row Chrome:** Removed redundant HTML `#titleBar` element to eliminate dual minimize/maximize/close controls with native OS window manager. Restored approved 2-row layout: Row 1 Tab Bar with tabs and `＋`, Row 2 Address Bar with `< > ⟳ ⌂`, input, GO, `★` favorites, and `⚙` settings button.
- **PT-BR Translation Mod Alignment & Resilient Config Persistence:** Harmonized `save_config` IPC deserializer to accept `new_config`, `config`, and `newConfig`; synchronized `modListEnabled` and `enabledMods` between backend Rust and frontend JavaScript; implemented case/underscore-tolerant mod lookup; confirmed 2,401 pages translated in memory with Portuguese titles (`Insira o nome.`, `Tente novamente.`).
- **Deterministic Theme Engine Across All Views:** Unified theme attribution across `html`, `body`, `#app`, and `#page-container`; eliminated hardcoded `applyTheme('mspa')` on system views so that user settings (`themeOverride` and `forceThemeOverride`) consistently take priority.

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
