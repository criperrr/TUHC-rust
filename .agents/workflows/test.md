---
name: test
description: Test execution and verification workflow for TUHC-rust.
version: 1.0.0
requires_agents: qa-automation-engineer
requires_skills: testing-patterns, verify-changes
artifact_outputs: test-report
---

# /test - Test Execution Workflow

$ARGUMENTS

---

## Purpose
Execute Rust unit tests, dual-track Python E2E contracts, and memory checks.

---

## Sub-commands
- `/test`: Run all unit and E2E tests (`cargo test && python3 tests/e2e_runner.py`).
- `/test unit`: Run Rust unit tests (`cargo test`).
- `/test e2e`: Run Python E2E tests (`python3 tests/e2e_runner.py`).
- `/test memory`: Run memory benchmark (`python3 .agents/scripts/memory_check.py`).
