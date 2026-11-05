---
name: quick-reference
version: 1.0.0
priority: P2
trigger: model_decision
description: Fast index of TUHC-rust specialized agents, skills, and verification scripts.
---

# Quick Reference - TUHC-rust

> Fast index of key agents, skills, and validation scripts for TUHC-rust.

## Primary Agents
- `orchestrator`: Project coordination and cross-domain task oversight.
- `backend-specialist`: Rust, Axum, Tokio, Tauri commands, VFS, and archive stores.
- `frontend-specialist`: Webview UI, Ruffle Flash player, audio sync, and theme CSS.
- `performance-optimizer`: Memory hygiene, RSS < 150MB verification, heap profiling.
- `qa-automation-engineer`: E2E test suite (Tier 1-4), mock server, contract validation.
- `devops-engineer`: CI/CD pipelines, Tauri bundling, GitHub Actions workflows.
- `security-auditor`: VFS path traversal defense, dependency audits (`cargo audit`).

## Key Commands & Verification Scripts
- Check & Format: `cargo fmt --check && cargo clippy -- -D warnings`
- Rust Tests: `cargo test`
- E2E Test Suite: `python3 tests/e2e_runner.py`
- Memory Verification: `python3 .agents/scripts/memory_check.py`
- Master Verification: `python3 .agents/scripts/verify_all.py`
