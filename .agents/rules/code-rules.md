---
name: code-rules
version: 1.0.0
priority: P0
trigger: model_decision
description: Apply when writing, modifying, or refactoring Rust, Tauri, or frontend code in TUHC-rust.
---

# Code Rules (TIER 1) - TUHC-rust

> Active whenever editing code, adding endpoints, or implementing subsystems.

---

## 1. Rust Best Practices (Tauri 2.0 & Axum)

1. **Error Propagation:**
   - Never use `.unwrap()` or `.expect()` in runtime HTTP handlers or IPC commands.
   - Use `?` operator or map errors into meaningful user-facing `String` or HTTP error statuses.
2. **Memory Safety & Allocations:**
   - Avoid unnecessary `.clone()` on large collections (`Arc<T>` should be shared instead).
   - Zero-copy streaming for binary assets (audio, SWF, images) via `axum::body::Body::from_stream`.
   - Never read entire multi-megabyte files into memory unless strictly required.
3. **Async / Tokio Concurrency:**
   - Do NOT run blocking CPU-heavy operations inside async Tokio threads; use `tokio::task::spawn_blocking` when reading large disk files.
   - Guard shared state using `tokio::sync::RwLock` for high read concurrency.
4. **Filesystem Security:**
   - All filesystem paths originating from URLs or IPC must pass through `Vfs::sanitize_path`.
   - Rejection criteria: relative path segments (`..`), null bytes (`\0`), URL-encoded traversal (`%2e%2e`).

---

## 2. Frontend / Webview Rules

1. **Ruffle Flash Integration:**
   - Ruffle WebAssembly player must be initialized with audio delay compensation from calibration tables.
   - Viewport dimensions must preserve classic 650x450 Flash aspect ratios without distortion.
2. **WebKitGTK Compatibility:**
   - Avoid unsupported modern browser experimental APIs.
   - Do not rely on persistent service workers or large indexedDB databases that could leak memory in WebKitGTK.
