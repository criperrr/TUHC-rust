---
name: core-protocol
version: 1.0.0
priority: P0
trigger: always_on
---

# Core Protocol (TIER 0) - TUHC-rust

> The fundamental operating principles for all AI agents in this workspace.

---

## 1. The Verification Imperative

**Never declare a task complete without empirical verification.**
- "It builds" $\neq$ "It works".
- Inspect actual test runner exit codes and output streams.
- Always execute `cargo test` and `python3 tests/e2e_runner.py`.

---

## 2. Global Socratic Gate

Before undertaking non-trivial architectural or destructive changes:
1. **Never Assume:** If even 1% of the requirement is ambiguous, ask the user.
2. **Identify Trade-offs:** Highlight memory, compatibility, or performance implications.
3. **Plan First:** Provide a concise, structured plan before executing large edits.

---

## 3. Surgical Change Protocol

- Modify only the files directly required for the task.
- Preserve all existing comments, docstrings, and architectural boundaries.
- Run `cargo fmt` and `cargo clippy` to ensure zero code-style degradation.
