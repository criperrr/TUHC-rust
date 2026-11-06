---
name: request-routing
version: 1.0.0
priority: P0
trigger: always_on
---

# Request Routing - TUHC-rust

> Routing table for mapping user requests to specialized agent personas and toolsets.

---

## Agent Routing Table

| Task Domain | Primary Agent | Specialized Tooling & Directives |
|---|---|---|
| **Rust Backend & Axum Server** | `backend-specialist` | Tokio async runtime, Axum routing, Tower HTTP, Range headers |
| **Virtual File System & Mods** | `backend-specialist` | `server/vfs.rs`, `mods/parser.rs`, legacy `mod.js` syntax |
| **Webview & Ruffle Frontend** | `frontend-specialist` | WebKitGTK / WebView2, Vue/HTML5 shell, Ruffle WASM player |
| **Memory Footprint & Leaks** | `performance-optimizer`| Memory RSS < 150MB SLA, `malloc_trim`, zero-copy streaming |
| **E2E & Contract Testing** | `qa-automation-engineer` | Dual-track Python tests, mock server, regression verification |
| **Frontend Visual & E2E Testing** | `frontend-e2e-specialist` | Playwright, screenshots, console logs, UI subsystem audits |
| **Security & Sandbox Audit** | `security-auditor` | VFS path traversal defense, `cargo audit`, input validation |
| **CI/CD, Release & Packaging** | `devops-engineer` | GitHub Actions, Tauri bundler, Debian `.deb` / `.AppImage` |
| **Legacy Architecture / Archive** | `code-archaeologist` | `mspa.json`, `music.json`, FlashProps calibration tables |
| **High-Level Coordination** | `orchestrator` | Multi-phase planning, cross-agent task handoff |
