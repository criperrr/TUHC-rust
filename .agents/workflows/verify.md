---
name: verify
description: Verify code changes work by running full compilation, clippy, unit tests, E2E contracts, and memory checks.
version: 1.0.0
requires_agents: qa-automation-engineer
requires_skills: verify-changes
artifact_outputs: verification-report
---

# /verify - Full System Verification

$ARGUMENTS

---

## Verification Pipeline
1. `cargo check`
2. `cargo fmt --check`
3. `cargo clippy -- -D warnings`
4. `cargo test`
5. `python3 tests/e2e_runner.py`
6. `python3 .agents/scripts/memory_check.py`
