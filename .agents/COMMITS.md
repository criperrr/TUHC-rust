# COMMITS.md - TUHC-rust Commit Log History

Historical record of modifications and atomic commits.
Append new entries to the end of this file using the standard template:

```markdown
## `YYYY-MM-DD HH:MM` - `type(scope)`: Short title

- **Description:** Summary of changes...
- **Files Modified:**
  - `path/to/file`
- **Impact / Next Steps:** Outcome and next steps.
```

---

## `2026-11-05 18:15` - `feat(core): establish milestone 1 baseline, audit hardening and governance ci/cd`

- **Description:** Comprehensive forensic audit and baseline establishment for Milestone 1 (Backend Core, Asset Server, VFS & Test Infrastructure). Resolved concurrent config save race conditions with `tempfile::Builder` atomic persistence, eliminated legacy mobile and web debt (`ios.png`, Presco/Expo workflow text, INI3A-EQ3 plugin builder residue), hardened memory gatekeeper (< 150MB RSS), unified Clippy `--all-targets -D warnings` enforcement, enabled desktop packaging bundles in `tauri.conf.json`, rebuilt fast validation checklist, and established GitHub CI/CD workflows (`pr-gatekeeper.yml`, `staging.yml`, `release.yml`, `nightly-audit.yml`).
- **Files Modified:**
  - `GATE_STATUS.md`
  - `AGENTS.md`
  - `.gitignore`
  - `Cargo.toml`
  - `Cargo.lock`
  - `src/index.html`
  - `src-tauri/*`
  - `tests/*`
  - `.agents/*`
  - `.github/workflows/*`
- **Impact / Next Steps:** Milestone 1 Gate APPROVED with 100% test pass (68/68 E2E, 16 Rust tests, 57MB RSS). Repository baseline established on `main`, pushed to origin, and branched to `dev` for Milestone 2 development.

## `2026-11-05 18:34` - `fix(ci)`: Add test fixtures and resilient IPC query parameter coercion

- **Description:** Added self-contained test fixtures in `tests/fixtures/` with automatic fallback in `tests/common.py` when full asset pack is not mounted (enabling headless GitHub Actions CI execution). Hardened config persistence with `sync_all()` and added robust query parameter coercion (`parse_u64_arg`, `parse_bool_arg`, `parse_page_id_arg`) in Axum IPC dispatcher.
- **Files Modified:**
  - `src-tauri/src/config/mod.rs`
  - `src-tauri/src/server/mod.rs`
  - `tests/common.py`
  - `tests/fixtures/*`
  - `.agents/COMMITS.md`
- **Impact / Next Steps:** Ensures 100% green builds and test suites across both local developer environments and remote GitHub Actions CI runners.
