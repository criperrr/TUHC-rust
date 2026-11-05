# ISSUES.md - Known Issues, Backlog & Technical Debt

---

## 1. Active Tasks & Backlog
- [ ] Implement live frontend Vue/Webview UI shell loading inside `src/index.html`.
- [ ] Add cross-platform Windows (WebView2) CI build matrix in `release.yml`.
- [ ] Support multi-language asset pack directory auto-discovery across common user download paths.

## 2. Technical Debt & Mitigations
- **Issue:** Legacy `mod.js` syntax variations across third-party mods.
  - *Status:* Regex tokenizer covers common patterns (`trees`, `routes`); more complex conditional JS logic may require edge-case parsing extensions.
- **Issue:** WebKitGTK smooth scrolling memory overhead.
  - *Status:* Mitigated via `settings.set_enable_smooth_scrolling(false)` in `src-tauri/src/lib.rs`.
