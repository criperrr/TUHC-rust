# E2E Test Infrastructure: TUHC Tauri 2.0 Re-implementation

## 1. Overview & Objectives

This document specifies the architecture, methodology, and execution contracts of the End-to-End (E2E) Test Suite for the TUHC Tauri 2.0 re-implementation (`tuhc_tauri`).

The test suite operates under a **Dual-Track Testing Architecture**:
- **Requirement-Driven & Opaque-Box**: Tests validate externally observable behaviors across HTTP endpoints, IPC protocols, virtual file system (VFS) overlays, data serialization, and configuration persistence without binding to internal implementation details.
- **Progressive Testability & Independence**: Tests are fully self-contained, isolated, order-independent, and derive ground-truth expectations directly from verified reference oracles.
- **Dual Execution Engine**: Supports both direct contract/oracle execution (with an integrated Axum-compatible contract server backed by Asset Pack V2) and live-attach execution against the running Tauri 2.0 application backend.

---

## 2. Authoritative Reference Oracles

Every test assertion is anchored to authoritative reference data available in the environment:

| Oracle Source | Path | Verification Role |
|---------------|------|-------------------|
| **Asset Pack V2** | `/home/criper/Downloads/Asset Pack V2` | File existence, exact byte sizes, MD5/SHA256 digests, MIME typing, and Range response verification for ~19,000 files. |
| **Monolithic Story Archive** | `.../archive/data/mspa.json` | 9,991 canonical story pages, media mappings, story titles, page IDs (`000001` - `010029`), next/previous links, retcon flags. |
| **Monolithic Music Archive** | `.../archive/data/music.json` | 37 albums, 797 musical tracks, artists, durations, and flash sync references. |
| **Legacy Mod Pack (PT-BR)** | `.../mods/Homestuck PT-BR` | `mod.js` syntax & trees mapping (`./Assets/` -> `assets://`), `translation2.json` (2,401 translated pages), and 1,143 localized replacement assets. |
| **Flash Calibration Tables** | `/tmp/tuhc-repo/src/js/flashProps.js` | Millisecond audio delay calibrations (`audioDelay`), negative seek offsets, Cascade multi-segment timings, and viewport dimensions (`cropHeight`, `indexedFlashProps`). |
| **Reference Configuration** | `/home/criper/.config/unofficial-homestuck-collection/config.json` | Schema, migration keys (`__internal__`), active tabs, history arrays, reader progress (`newReader`), theme overrides, and `modListEnabled` format. |

---

## 3. Four-Tier Test Taxonomy

### Tier 1: Feature Coverage (Core Requirements)
Covers baseline happy paths for all foundational features. Requires $\ge 5$ distinct test cases per core feature:
1. **Asset Server (F03)**: Loopback HTTP requests, status 200 OK, MIME headers (GIF, PNG, MP3, SWF, WASM), Range headers (`bytes=0-1023`), Content-Length accuracy.
2. **In-Memory Store (F04)**: Story page querying (`001901`, `001902`), story ranges (`001901` count 10), music archive retrieval, album listing.
3. **Config & Save Persistence (F05)**: Config read, config mutation & write-back, schema integrity, tab data persistence, settings preservation.
4. **Tauri Core IPC APIs (F06)**: IPC command handling for `get_app_info`, `get_story_page`, `get_story_range`, `get_music_data`, `load_config`, `save_config`, `get_available_mods`, `toggle_mod`.
5. **Webview Asset Loading (F07/F09)**: Static web assets, fonts, stylesheets, theme definitions, Ruffle emulator assets.

### Tier 2: Boundary & Corner Cases (Robustness & Error Handling)
Verifies boundary conditions, error handling, and security resilience. Requires $\ge 5$ distinct test cases per boundary domain:
1. **Missing Assets (404 Handling)**: Non-existent paths, traversal attacks (`../../etc/passwd`), null-byte injections, empty path queries.
2. **Out-of-Bounds Story Queries**: Page ID `000000`, negative page numbers, pages $> 010029$, invalid alphanumeric non-page IDs, empty string IDs.
3. **Malformed & Invalid Mods**: Directory without `mod.js`, invalid JavaScript syntax, missing `translation2.json`, missing `Assets` folder, cyclic tree mappings.
4. **Malformed Range Requests**: Syntactically invalid ranges (`bytes=xyz`), open-ended ranges exceeding file length, inverted ranges (`bytes=100-50`), empty range strings.
5. **Malformed IPC / Config Payloads**: Empty request bodies, invalid JSON syntax, missing required fields, type mismatches (string instead of boolean).

### Tier 3: Cross-Feature Combinations (Pairwise Interaction)
Validates interactions between disparate subsystems operating simultaneously:
1. **Mod PT-BR Overlay + Story Fetch**: Translation text merged into story payload while localized media (`00002.gif`) is simultaneously served via the VFS.
2. **Mod Toggle + Config Persistence**: Toggling a mod updates in-memory registry, flushes `modListEnabled` to `config.json`, and reflects immediately on next asset query.
3. **Audio Delay Calculation + Flash Asset**: Fetching a Flash SWF combined with lookup in `flashProps.audioDelay` (positive delays, negative delays, Cascade multi-segment splits).
4. **Theme Override + Story Page Layout**: Setting custom theme in config affects story page metadata and style resolution.
5. **Range Streaming on Localized Asset vs Base Asset**: Range requests on base asset vs localized asset return respective byte slices without corruption.

### Tier 4: Real-World Scenarios (End-to-End User Workflows)
Simulates end-to-end user journeys mirroring actual reader sessions:
1. **Sequential Story Reader Walkthrough**: Traversing consecutive pages (`001901` $\to$ `001902` $\to$ `001903` $\to$ `001904`), validating navigation chains, title updates, and history tracking.
2. **Flash Audio Sync Scenario**: Loading `[S] WV?: Rise up.` (`003701`) and `[S] Cascade.` (`006009`), verifying `.swf` asset streaming, `#audioStart` Frag IPC dispatch, and delay calibration.
3. **Full Homestuck PT-BR Localized Reading Session**: Enabling Portuguese mod, verifying translated titles, body text, pesterlog `vueHooks` button text (`Abrir pesterlog` / `Fechar pesterlog`), and localized graphics.
4. **Music Player Sound Test**: Querying album discography, inspecting track metadata, verifying audio streaming endpoints.
5. **Tab Session Save & Restore**: Creating multi-tab sessions with history stacks, saving state, restarting session, and validating full state restoration.

---

## 4. Test Runner Architecture (`tests/e2e_runner.py`)

The test harness is implemented in pure Python 3 (standard library only) to ensure zero setup friction, fast startup, and universal CI/CD compatibility.

```
teamwork_projects/tuhc_tauri/tests/
├── e2e_runner.py                       # CLI test runner & orchestrator
├── common.py                           # Test harness base, assertion helpers, HTTP client, oracles
├── mock_server.py                      # Reference contract backend (Axum/Tauri contract simulator)
├── tier1_features/
│   ├── test_asset_server.py            # F03 Asset server & range streaming
│   ├── test_monolithic_store.py        # F04 In-memory JSON store
│   ├── test_config_persistence.py      # F05 Config & save manager
│   ├── test_ipc_contracts.py           # F06 Tauri IPC APIs
│   └── test_webview_assets.py          # F07/F09 Webview assets & styling
├── tier2_boundaries/
│   ├── test_missing_assets.py          # 404s & path traversal protection
│   ├── test_out_of_bounds_pages.py     # Out-of-bounds page IDs
│   ├── test_invalid_mods.py            # Corrupt/missing mod structures
│   ├── test_malformed_ranges.py        # Malformed HTTP Range requests
│   └── test_malformed_payloads.py      # Malformed IPC/Config bodies
├── tier3_combinations/
│   ├── test_mod_overlay_story.py       # PT-BR translation + VFS overlay
│   ├── test_mod_toggle_persistence.py  # Runtime toggle & config persistence
│   ├── test_audio_sync_flash.py        # Flash props + audio delay offsets
│   ├── test_theme_story_nav.py         # Config theme override + story reader
│   └── test_range_streaming_overlay.py # Range requests on VFS overlay assets
└── tier4_realworld/
    ├── test_sequential_reading.py      # Pages 001901..001904 walkthrough
    ├── test_flash_audio_sync.py        # Cascade / Rise Up flash audio sync
    ├── test_ptbr_reading_session.py    # Complete PT-BR reader workflow
    ├── test_music_player_session.py    # Sound test & track player
    └── test_tab_session_restore.py     # Tab state preservation & restore
```

### Command-Line Interface

```bash
# Run all test tiers (auto-spawns reference contract server on ephemeral port)
python3 tests/e2e_runner.py

# Run a specific tier
python3 tests/e2e_runner.py --tier 1
python3 tests/e2e_runner.py --tier 2
python3 tests/e2e_runner.py --tier 3
python3 tests/e2e_runner.py --tier 4

# Run against a live running Tauri 2.0 backend
python3 tests/e2e_runner.py --live --url http://127.0.0.1:45100

# Filter tests by substring pattern
python3 tests/e2e_runner.py -k ptbr
python3 tests/e2e_runner.py -k range

# Generate JSON report for CI/CD
python3 tests/e2e_runner.py --json-report test_report.json
```

---

## 5. Verification & Acceptance Criteria

1. **Deterministic Oracles**: Every test compares against observable physical files in Asset Pack V2, `mspa.json`, `music.json`, `flashProps.js`, and `config.json`.
2. **Zero Facade Testing**: No mock stubs that bypass validation; all HTTP queries send real socket requests, verify HTTP headers, and read exact binary byte payloads.
3. **Coverage Compliance**:
   - Tier 1: $\ge 25$ tests (5 core features $\times \ge 5$ tests).
   - Tier 2: $\ge 25$ tests (5 boundary domains $\times \ge 5$ tests).
   - Tier 3: $\ge 5$ tests across pairwise feature combinations.
   - Tier 4: $\ge 5$ real-world end-to-end user journeys.
4. **Execution Speed**: Full suite runs in $< 5$ seconds.
