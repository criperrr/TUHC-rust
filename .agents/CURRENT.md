# CURRENT.md - Project State & Navigation Index

Executive summary and direct file index for token-efficient agent navigation in TUHC-rust. Read this file first on every task.

---

## 1. Executive Summary

**Project Title:** TUHC-rust (The Unofficial Homestuck Collection - Tauri 2.0 / Rust Re-write)  
**Active Status:** Aligned story commands and footer typography with authentic MSPA, verified logo transparency and settings controls on `fix/title-transparency-typography-and-settings`.
**Quality Gate Status:** All verification gates passing 100% (Compilation, Formatting, Clippy, 31 Rust Tests, 3 Adversarial Integration Tests, 68 Python E2E Contract Tests, Memory RSS below 150MB SLA).

**Key Achievements:**
- **Universal Asset Resolution for Act Banners & Graphics:** Fixed `resolveAssetUrl` to automatically route all paths (including `/maps/`, `/images/`, `/storyfiles/`, `/archive/`, `/sweetbroandhellajeff/`, `/advimgs/`, `/scraps/`) through `${state.assetBase}/assets${url}` via Axum VFS, completely eliminating broken image `[?]` icons on `/map` and displaying all 26 authentic animated GIF act banners.
- **Top Homepage 3D Animated Logo Fix:** Replaced `<video>` element (which rendered an empty white box in Linux WebKitGTK due to GStreamer codec gaps) with native 227-frame 800x300 animated GIF (`collection_logo.gif`) and static fallback (`logo_v2_static.png`) when `reducedMotion` is enabled.
- **Problem Sleuth Map Graphic Alignment:** Replaced generic placeholder banner with authentic `/maps/map_04.gif` on `/map/4`.
- **Complete Elimination of Arial Across UI:** Replaced all system sans-serif/Arial typography across `.tabTitle`, `.tab`, `.urlTooltip`, form elements, and system pages with authentic `courierstuck` and `Courier Prime` font stack.
- **WOFF WebFont Bundling & Path Sanitization:** Converted `courierstuck` and `Courier Prime` to lightweight `.woff` format and created non-space URL-safe copies with CSS `@font-face` fallbacks (relative `../fonts/` and absolute `/fonts/`). Added `<link rel="preload">` in `index.html`.
- **Discord Browsing Rich Presence:** Added browsing and menu presence handler in `commands.rs` and `rpc/mod.rs` with automatic activity payload serialization and empty field stripping.
- **Authentic TUHC Official Homepage Recreation:** Faithfully reconstructed the official Homepage matching `Homepage.vue`, `HomeRowItem.vue`, and official TUHC production CSS. Features Top Logo Card with video/image fallback, Homestuck Card with dynamic "Begin Homestuck" / "Resume Homestuck at page [X]" button and canonical page numbering, 10 authentic HomeRowItems (Homestuck BETA, Music, SBAHJ, Dave's blog, Squiddles, Namco High, Paradox Space, Snapchat, Skaianet, Epilogues), MSPA Classic Card (Jailbreak, Bard Quest, Problem Sleuth, Unlocks, Science FAQ), Old Comics TSO Card, News Card (MSPA Newsposts, Blogspot, Formspring, Tumblr), and Unsorted Card (Wallpapers, The Baby is You, Vigil Prince, Ryanquest).
- **Navigation Bar & Jumpbox Border Sanitization:** Eliminated double borders, inner box outlines, and focus box-shadows from `.jumpBoxInput` and `.jumpBox`, perfectly aligning visual styling with original `AddressBar.vue`.
- **Footer MSPA Logo Resolution:** Replaced broken static `/assets/images/mspalogo_mspa.png` path with theme-aware `updateFooterLogos(theme)` dynamically resolving through `resolveAssetUrl(...)` across themes (`mspa`, `scratch`, `sbahj`, `trickster`, `A6A6`, `pxs`) with automatic error fallback (`onerror="this.style.display='none'"`).
- **MSPA Typography Refinement:** Removed generic `Arial, Helvetica, sans-serif` from `.customNavBanner` and replaced with authentic font stack (`"Courier Prime", "Courier New", Verdana, Geneva, monospace, sans-serif`).
- **Reading Progress Auto-Save & Resume:** Implemented automatic tracking of read pages within the Homestuck story range (`001901` to `010030`), automatically updating `settings.newReader.current` and `limit` in `config.json` via debounced persistence, and dynamically surfacing `"Resume Homestuck at page [X]"` on the Homepage.
- **Frontend E2E Test Suite 100% Pass:** Resolved asynchronous command container race condition in `renderStoryPage` through synchronous DOM scaffolding and fast in-memory page cache, achieving 9/9 passing tests in Playwright suite.
- **Visual Readability and Logo Transparency:** Converted the homepage animated logo to transparent frames and optimized it from 13.9MB to 5.8MB; increased settings labels, controls, dates, navigation, and story text sizes; set the UI font stack to Courier Prime to remove remaining Arial use.
- **Theme Surface Contrast:** Added explicit foreground colors for log and card surfaces, corrected scratch/cascade/collide log backgrounds to match their light text, made Trickster body text black against its bright pink surfaces, and brightened footer metadata on dark themes.
- **Settings Selector and Archive Alignment:** Fixed the theme selector foreground/background and native color scheme so the selected value remains readable in dark themes; centered archive thumbnails over their date/page metadata across all entries.
- **Archive Entry Column Alignment:** Set a shared responsive archive icon/metadata column (up to 200px) so descriptions in Jailbreak, Bard Quest, Problem Sleuth, and similar entries share one horizontal start position.
- **Automatic Progress Saves & Cartridge Shelf:** Story pages update a protected autosave cartridge in `saveData`; the SBURB Save Deck displays autosaves and manual cartridges with page, title, and time plus load/delete actions. Manual cartridges retain their own Flash storage snapshots.
- **Ruffle SharedObject Save Bridge:** Wrapped the embedded player’s local storage methods and persist each Flash game’s SharedObject data through `saveData.flashStorage`, so selecting a cartridge seeds that player with its saved data. Ruffle has no public general-purpose frame savestate API, so games that do not persist their own state cannot be restored to arbitrary in-memory frames.

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
- `config/mod.rs`: `AppConfig` persistence with atomic tempfile replacement, `config.json` serialization, tab management, flexible save cartridge metadata, and Flash storage snapshots.
- `mods/mod.rs`: `ModRegistry` managing installed and enabled community mods.
- `mods/parser.rs`: Regex-based declarative parser for `mod.js` and patch loader for `translation2.json`.
- `rpc/mod.rs`: Native asynchronous Discord Local IPC manager, MSPA chapter resolution, Reading/Music presence, and Unix domain socket client.
- `bin/server.rs`: Standalone contract server binary for CI and test harnesses.

### 2.2. Frontend Shell (`src/`)
- `index.html`: Shell loading Webview UI, Ruffle Flash player WASM, and loopback communication.
- `styles/main.css`: 13 authentic MSPA themes, Courier Prime/Courierstuck typography, readable settings controls, pesterlog quirks, and 950px frame.
- `js/app.js`: Tabbed browser navigation, URL router, progress autosaves, save cartridge shelf, bookmark persistence, settings, and sound test.
- `js/flashProps.js`: Millisecond audio delay tables, negative seek offsets, crop heights, and transitions.
- `js/mediaEngine.js`: Ruffle WASM iframe loader, per-game SharedObject save bridge, Frag IPC bridge, HTML5 audio/video, and Sburb canvas.
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
