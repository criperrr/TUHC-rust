---
name: universal-rules
version: 1.0.0
priority: P0
trigger: always_on
---

# Universal Rules (TIER 0) - TUHC-rust

> Always-active rules that apply to every request in the TUHC-rust repository.

---

## 🌐 Language Handling

When user's prompt is NOT in English:
1. **Internally translate** for full technical precision and context comprehension.
2. **Respond in user's language** (e.g. Portuguese) matching their phrasing style.
3. **Code, variables, comments, and commit messages** must remain strictly in English.

---

## 🧹 Clean Code & Memory SLA (Global Mandatory)

**ALL code MUST follow clean, memory-efficient Rust and web patterns:**
- **Concise & Direct:** Avoid speculative over-engineering, dead code, and bloated abstractions.
- **Memory Ceiling:** Enforce total process Resident Set Size (RSS) strictly **< 150MB**.
- **Security First:** Never expose filesystem traversal vulnerabilities; always validate through `Vfs::sanitize_path`.
- **Testing:** Mandatory test verification before claiming completion. Run `cargo test` and `python3 tests/e2e_runner.py`.
