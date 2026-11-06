pub mod handler;
pub mod vfs;

use axum::{
    body::Body,
    extract::{Path as AxumPath, Query, Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Router,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::data::ArchiveStore;
use crate::mods::ModRegistry;
use crate::rpc::DiscordRpcManager;
pub use vfs::{ActiveModDescriptor, Vfs};

#[derive(Clone)]
pub struct AppStateInner {
    pub config: Arc<RwLock<AppConfig>>,
    pub asset_port: u16,
    pub asset_dir: PathBuf,
    pub store: Arc<ArchiveStore>,
    pub mods: Arc<RwLock<ModRegistry>>,
    pub vfs: Arc<Vfs>,
    pub discord_rpc: Arc<DiscordRpcManager>,
}

pub struct AssetServerHandle {
    pub port: u16,
    pub state: AppStateInner,
}

fn json_response(status: StatusCode, val: serde_json::Value) -> Response {
    let body = match serde_json::to_string(&val) {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let mut res = (status, body).into_response();
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    handler::add_cors_headers(headers);
    res
}

fn get_frontend_src_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("TUHC_FRONTEND_SRC") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }
    let manifest_parent = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("src"));
    if let Some(p) = manifest_parent {
        if p.is_dir() {
            return p;
        }
    }
    let cur_src = PathBuf::from("src");
    if cur_src.is_dir() {
        return cur_src;
    }
    let parent_src = PathBuf::from("../src");
    if parent_src.is_dir() {
        return parent_src;
    }
    PathBuf::from("src")
}

async fn serve_index_html() -> Response {
    let src_dir = get_frontend_src_dir();
    let index_file = src_dir.join("index.html");
    if let Ok(bytes) = std::fs::read(&index_file) {
        if let Ok(s) = std::str::from_utf8(&bytes) {
            if s.contains("The Unofficial Homestuck Collection") {
                let mut res = (StatusCode::OK, bytes).into_response();
                let headers = res.headers_mut();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                );
                headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
                handler::add_cors_headers(headers);
                return res;
            }
        }
    }

    // Clean built-in fallback shell
    let content = b"<!DOCTYPE html><html><head><meta charset='utf-8'><title>The Unofficial Homestuck Collection</title><link rel='stylesheet' href='/styles/main.css'></head><body><div id='app'>The Unofficial Homestuck Collection Webview Shell</div></body></html>";
    let mut res = (StatusCode::OK, content.as_slice()).into_response();
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&content.len().to_string()).unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    handler::add_cors_headers(headers);
    res
}

async fn serve_style_file(style_path: &str) -> Response {
    let clean_rel = style_path.trim_start_matches('/');
    let src_dir = get_frontend_src_dir();
    let file = src_dir.join("styles").join(clean_rel);

    if file.is_file() {
        if let Ok(bytes) = std::fs::read(&file) {
            let mut res = (StatusCode::OK, bytes).into_response();
            let headers = res.headers_mut();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/css; charset=utf-8"),
            );
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            handler::add_cors_headers(headers);
            return res;
        }
    }

    // Built-in fallback
    let css_data =
        b"body { font-family: 'Courier New', monospace; background: #000; color: #fff; }";
    let mut res = (StatusCode::OK, css_data.as_slice()).into_response();
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&css_data.len().to_string()).unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    handler::add_cors_headers(headers);
    res
}

async fn serve_js_file(js_path: &str) -> Response {
    let clean_rel = js_path.trim_start_matches('/');
    let src_dir = get_frontend_src_dir();
    let file = src_dir.join("js").join(clean_rel);

    if file.is_file() {
        if let Ok(bytes) = std::fs::read(&file) {
            let mut res = (StatusCode::OK, bytes).into_response();
            let headers = res.headers_mut();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript; charset=utf-8"),
            );
            headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            handler::add_cors_headers(headers);
            return res;
        }
    }

    // Fallback JS
    let js_data = format!(
        "/* fallback {} */\nconsole.log('TUHC dynamic js loaded: {}');",
        clean_rel, clean_rel
    );
    let mut res = (StatusCode::OK, js_data.into_bytes()).into_response();
    let headers = res.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    handler::add_cors_headers(headers);
    res
}

/// Coerces a JSON value (number, positive integer, or string) to u64, or default
fn parse_u64_arg(val: Option<&serde_json::Value>, default: u64) -> usize {
    val.and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_i64().and_then(|n| u64::try_from(n).ok()))
            .or_else(|| v.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
    })
    .unwrap_or(default) as usize
}

/// Coerces a JSON value (bool, 0/1 integer, or boolean string) to bool, or default
fn parse_bool_arg(val: Option<&serde_json::Value>, default: bool) -> bool {
    val.and_then(|v| {
        v.as_bool()
            .or_else(|| match v.as_i64() {
                Some(1) => Some(true),
                Some(0) => Some(false),
                _ => None,
            })
            .or_else(|| match v.as_str() {
                Some(s) if s.eq_ignore_ascii_case("true") || s == "1" => Some(true),
                Some(s) if s.eq_ignore_ascii_case("false") || s == "0" => Some(false),
                _ => None,
            })
    })
    .unwrap_or(default)
}

/// Extracts page_id supporting "page_id", "pageId", or "id" as string or positive integer
fn parse_page_id_arg(args: &serde_json::Value) -> Option<String> {
    let raw = args
        .get("page_id")
        .or_else(|| args.get("pageId"))
        .or_else(|| args.get("id"))?;
    if let Some(s) = raw.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed.starts_with('-') || trimmed == "000000" {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else if let Some(n) = raw.as_i64() {
        if n <= 0 {
            None
        } else {
            Some(format!("{:06}", n))
        }
    } else {
        None
    }
}

pub async fn dispatch_ipc(
    state: &AppStateInner,
    command: &str,
    args: &serde_json::Value,
) -> Response {
    match command {
        "get_app_info" => {
            let info = serde_json::json!({
                "version": "2.8.1",
                "asset_dir": state.asset_dir.to_string_lossy(),
                "asset_port": state.asset_port,
                "overlay_generation": state.vfs.generation(),
                "overlayGeneration": state.vfs.generation(),
            });
            json_response(StatusCode::OK, info)
        }
        "get_story_page" => {
            let page_id = match parse_page_id_arg(args) {
                Some(id) => id,
                None => {
                    return json_response(
                        StatusCode::NOT_FOUND,
                        serde_json::json!({"error": "Page not found"}),
                    );
                }
            };

            match state.store.get_story_page(&page_id).await {
                Some(page) => {
                    if state.config.read().await.settings.enable_discord_rpc {
                        let _ = state
                            .discord_rpc
                            .update_reading_presence(&page.page_id, &page.title)
                            .await;
                    }
                    match serde_json::to_value(&page) {
                        Ok(v) => json_response(StatusCode::OK, v),
                        Err(e) => json_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            serde_json::json!({"error": e.to_string()}),
                        ),
                    }
                }
                None => json_response(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": format!("Page '{}' not found", page_id)}),
                ),
            }
        }
        "get_story_range" => {
            let start = parse_u64_arg(args.get("start"), 1901);
            let count = parse_u64_arg(args.get("count"), 10);

            let summaries = state.store.get_story_range(start, count).await;
            match serde_json::to_value(&summaries) {
                Ok(v) => json_response(StatusCode::OK, v),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({"error": e.to_string()}),
                ),
            }
        }
        "get_music_data" => {
            let raw = state.store.get_music_raw();
            json_response(StatusCode::OK, raw)
        }
        "load_config" => {
            let cfg = state.config.read().await;
            match serde_json::to_value(&*cfg) {
                Ok(v) => json_response(StatusCode::OK, v),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({"error": e.to_string()}),
                ),
            }
        }
        "save_config" => {
            let config_val = if let Some(c) = args.get("config") {
                c
            } else if let Some(c) = args.get("new_config") {
                c
            } else if let Some(c) = args.get("newConfig") {
                c
            } else if args.is_object() && args.get("settings").is_some() {
                args
            } else {
                return json_response(
                    StatusCode::BAD_REQUEST,
                    serde_json::json!({"error": "Invalid config schema"}),
                );
            };

            let new_config: AppConfig = match serde_json::from_value(config_val.clone()) {
                Ok(c) => c,
                Err(e) => {
                    return json_response(
                        StatusCode::BAD_REQUEST,
                        serde_json::json!({"error": format!("Invalid config schema: {}", e)}),
                    )
                }
            };

            if let Err(e) = new_config.save() {
                return json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({"error": format!("Failed to save config: {}", e)}),
                );
            }

            // Sync discord RPC setting if changed
            if state.discord_rpc.get_status().await.enabled
                != new_config.settings.enable_discord_rpc
            {
                let _ = state
                    .discord_rpc
                    .set_enabled(new_config.settings.enable_discord_rpc)
                    .await;
            }

            {
                let mut cfg = state.config.write().await;
                *cfg = new_config.clone();
            }

            // Sync modListEnabled with active mods
            {
                let mut mods_lock = state.mods.write().await;
                mods_lock.rescan(&new_config.settings.mod_list_enabled);
                let descriptors = mods_lock.get_active_descriptors();
                let _ = state.vfs.rebuild_overlay(descriptors).await;
                let patches = mods_lock.load_active_patches();
                if patches.is_empty() {
                    state.store.story.reset_to_base().await;
                } else {
                    state.store.story.apply_patches(&patches).await;
                }
            }

            json_response(
                StatusCode::OK,
                serde_json::json!({"status": "ok", "saved": true}),
            )
        }
        "get_available_mods" => {
            let mods_lock = state.mods.read().await;
            let list = mods_lock.get_available_mods();
            match serde_json::to_value(&list) {
                Ok(v) => json_response(StatusCode::OK, v),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({"error": e.to_string()}),
                ),
            }
        }
        "toggle_mod" => {
            let mod_id = if let Some(m) = args.get("mod_id").and_then(|v| v.as_str()) {
                m.to_string()
            } else if let Some(m) = args.get("modId").and_then(|v| v.as_str()) {
                m.to_string()
            } else if let Some(m) = args.get("id").and_then(|v| v.as_str()) {
                m.to_string()
            } else {
                return json_response(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": "Missing or non-string mod_id"}),
                );
            };

            let enabled = parse_bool_arg(args.get("enabled"), false);

            let (success, descriptors, patches) = {
                let mut mods_lock = state.mods.write().await;
                match mods_lock.toggle_mod(&mod_id, enabled) {
                    Ok(_) => {
                        let descriptors = mods_lock.get_active_descriptors();
                        let patches = mods_lock.load_active_patches();
                        (true, descriptors, patches)
                    }
                    Err(e) => {
                        return json_response(
                            StatusCode::NOT_FOUND,
                            serde_json::json!({"error": e}),
                        )
                    }
                }
            };

            if success {
                let _ = state.vfs.rebuild_overlay(descriptors).await;
                if patches.is_empty() {
                    state.store.story.reset_to_base().await;
                } else {
                    state.store.story.apply_patches(&patches).await;
                }

                // Update config
                let mut cfg = state.config.write().await;
                let list = &mut cfg.settings.mod_list_enabled;
                if enabled {
                    if !list.contains(&mod_id) {
                        list.push(mod_id.clone());
                    }
                } else {
                    list.retain(|id| id != &mod_id);
                }
                let _ = cfg.save();
            }

            json_response(
                StatusCode::OK,
                serde_json::json!({
                    "status": "ok",
                    "mod_id": mod_id,
                    "enabled": enabled,
                    "overlay_generation": state.vfs.generation(),
                    "overlayGeneration": state.vfs.generation(),
                }),
            )
        }
        "get_mod_hooks" => {
            let mods_lock = state.mods.read().await;
            let hooks = mods_lock.get_active_vue_hooks();
            json_response(
                StatusCode::OK,
                serde_json::json!({
                    "vueHooks": hooks,
                    "overlayGeneration": state.vfs.generation(),
                    "overlay_generation": state.vfs.generation(),
                }),
            )
        }
        "evaluate_log_button_text" => {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .or_else(|| args.get("raw_content").and_then(|v| v.as_str()))
                .unwrap_or("");
            let log_hidden = parse_bool_arg(
                args.get("log_hidden").or_else(|| args.get("logHidden")),
                true,
            );
            let mods_lock = state.mods.read().await;
            let text = mods_lock.evaluate_log_button_text(content, log_hidden);
            json_response(
                StatusCode::OK,
                serde_json::json!({
                    "text": text,
                    "log_hidden": log_hidden,
                    "logHidden": log_hidden,
                }),
            )
        }
        "get_discord_status" => {
            let status = state.discord_rpc.get_status().await;
            match serde_json::to_value(&status) {
                Ok(v) => json_response(StatusCode::OK, v),
                Err(e) => json_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({"error": e.to_string()}),
                ),
            }
        }
        "set_discord_presence" => {
            let mode = args
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("reading");

            match mode {
                "reading" => {
                    let page_id = parse_page_id_arg(args).unwrap_or_else(|| "001901".to_string());
                    let page_title = if let Some(t) = args
                        .get("page_title")
                        .or_else(|| args.get("pageTitle"))
                        .and_then(|v| v.as_str())
                    {
                        t.to_string()
                    } else {
                        state
                            .store
                            .get_story_page(&page_id)
                            .await
                            .map(|p| p.title)
                            .unwrap_or_else(|| "Homestuck".to_string())
                    };

                    let status = state
                        .discord_rpc
                        .update_reading_presence(&page_id, &page_title)
                        .await;
                    json_response(
                        StatusCode::OK,
                        serde_json::json!({"status": "ok", "presence": status}),
                    )
                }
                "music" => {
                    let track_title = args
                        .get("track_title")
                        .or_else(|| args.get("trackTitle"))
                        .or_else(|| args.get("track_name"))
                        .or_else(|| args.get("trackName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Homestuck Music");
                    let album_name = args
                        .get("album_name")
                        .or_else(|| args.get("albumName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let artist = args.get("artist").and_then(|v| v.as_str());
                    let track_id = args
                        .get("track_id")
                        .or_else(|| args.get("trackId"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let playing = parse_bool_arg(args.get("playing"), true);

                    let status = state
                        .discord_rpc
                        .update_music_presence(track_title, album_name, artist, track_id, playing)
                        .await;
                    json_response(
                        StatusCode::OK,
                        serde_json::json!({"status": "ok", "presence": status}),
                    )
                }
                "clear" => {
                    let status = state.discord_rpc.clear_presence().await;
                    json_response(
                        StatusCode::OK,
                        serde_json::json!({"status": "ok", "presence": status}),
                    )
                }
                _ => {
                    let status = state.discord_rpc.get_status().await;
                    json_response(
                        StatusCode::OK,
                        serde_json::json!({"status": "ok", "presence": status}),
                    )
                }
            }
        }
        "toggle_discord_rpc" => {
            let current = state.discord_rpc.get_status().await.enabled;
            let enabled = parse_bool_arg(args.get("enabled"), !current);
            let status = state.discord_rpc.set_enabled(enabled).await;
            {
                let mut cfg = state.config.write().await;
                cfg.settings.enable_discord_rpc = enabled;
                let _ = cfg.save();
            }
            json_response(
                StatusCode::OK,
                serde_json::json!({"status": "ok", "enabled": enabled, "connected": status.connected}),
            )
        }
        "trim_memory" => {
            crate::commands::trim_memory();
            json_response(StatusCode::OK, serde_json::json!({"status": "ok"}))
        }
        _ => json_response(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": format!("Unknown IPC command '{}'", command)}),
        ),
    }
}

pub async fn handle_ipc_post(State(state): State<AppStateInner>, req: Request<Body>) -> Response {
    let bytes = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return json_response(
                StatusCode::BAD_REQUEST,
                serde_json::json!({"error": "Failed to read request body"}),
            )
        }
    };

    if bytes.is_empty() {
        return json_response(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Empty request body"}),
        );
    }

    let val: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => {
            return json_response(
                StatusCode::BAD_REQUEST,
                serde_json::json!({"error": "Invalid JSON payload"}),
            )
        }
    };

    let cmd = if let Some(c) = val.get("cmd").and_then(|v| v.as_str()) {
        c.to_string()
    } else if let Some(c) = val.get("command").and_then(|v| v.as_str()) {
        c.to_string()
    } else {
        return json_response(
            StatusCode::BAD_REQUEST,
            serde_json::json!({"error": "Missing IPC command"}),
        );
    };

    let empty_args = serde_json::json!({});
    let args = if let Some(a) = val.get("args") {
        a
    } else if let Some(a) = val.get("payload") {
        a
    } else {
        &empty_args
    };

    dispatch_ipc(&state, &cmd, args).await
}

pub async fn handle_ipc_get(
    State(state): State<AppStateInner>,
    AxumPath(cmd): AxumPath<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let args = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
    dispatch_ipc(&state, &cmd, &args).await
}

pub async fn fallback_router(State(state): State<AppStateInner>, req: Request<Body>) -> Response {
    let path = req.uri().path();
    if path == "/" || path == "/index.html" {
        return serve_index_html().await;
    }
    if let Some(style_rel) = path.strip_prefix("/styles/") {
        return serve_style_file(style_rel).await;
    }
    if let Some(js_rel) = path.strip_prefix("/js/") {
        return serve_js_file(js_rel).await;
    }
    if path.starts_with("/assets/") {
        return handler::handle_asset_request(&state.vfs, req).await;
    }
    // Single-Page Application (SPA) routing fallback:
    // Any non-asset route without a dot extension (e.g. /mspa/6, /homestuck/1, /settings, /music)
    // serves index.html so the client-side router handles navigation seamlessly.
    if !path.contains('.') {
        return serve_index_html().await;
    }
    handler::handle_asset_request(&state.vfs, req).await
}

pub async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    if req.method() == Method::OPTIONS {
        let mut res = StatusCode::NO_CONTENT.into_response();
        handler::add_cors_headers(res.headers_mut());
        return res;
    }

    let mut res = next.run(req).await;
    handler::add_cors_headers(res.headers_mut());
    res
}

pub async fn start_server(state: AppStateInner, port_tx: tokio::sync::oneshot::Sender<u16>) {
    let app = Router::new()
        .route("/ipc", post(handle_ipc_post))
        .route("/ipc/{cmd}", get(handle_ipc_get))
        .route("/", get(fallback_router))
        .route("/index.html", get(fallback_router))
        .route("/styles/{*path}", get(fallback_router))
        .route("/js/{*path}", get(fallback_router))
        .route("/assets/{*path}", any(fallback_router))
        .fallback(fallback_router)
        .layer(middleware::from_fn(cors_middleware))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind ephemeral loopback socket on 127.0.0.1:0");

    let port = listener
        .local_addr()
        .expect("Failed to read local socket address")
        .port();

    let _ = port_tx.send(port);
    log::info!("Axum asset server started on http://127.0.0.1:{}/", port);

    axum::serve(listener, app)
        .await
        .expect("Axum server crashed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u64_arg() {
        assert_eq!(parse_u64_arg(Some(&serde_json::json!(0)), 1901), 0);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!("0")), 1901), 0);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!(1901)), 10), 1901);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!("1901")), 10), 1901);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!(" 42 ")), 10), 42);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!(-5)), 1901), 1901);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!("-5")), 1901), 1901);
        assert_eq!(parse_u64_arg(Some(&serde_json::json!("abc")), 1901), 1901);
        assert_eq!(parse_u64_arg(None, 1901), 1901);
    }

    #[test]
    fn test_parse_bool_arg() {
        assert!(parse_bool_arg(Some(&serde_json::json!(true)), false));
        assert!(!parse_bool_arg(Some(&serde_json::json!(false)), true));
        assert!(parse_bool_arg(Some(&serde_json::json!("true")), false));
        assert!(!parse_bool_arg(Some(&serde_json::json!("False")), true));
        assert!(parse_bool_arg(Some(&serde_json::json!(1)), false));
        assert!(!parse_bool_arg(Some(&serde_json::json!(0)), true));
        assert!(parse_bool_arg(Some(&serde_json::json!("1")), false));
        assert!(!parse_bool_arg(Some(&serde_json::json!("0")), true));
        assert!(!parse_bool_arg(Some(&serde_json::json!("invalid")), false));
        assert!(!parse_bool_arg(None, false));
    }

    #[test]
    fn test_parse_page_id_arg() {
        let m1 = serde_json::json!({"page_id": "001901"});
        assert_eq!(parse_page_id_arg(&m1), Some("001901".to_string()));

        let m1_camel = serde_json::json!({"pageId": "001901"});
        assert_eq!(parse_page_id_arg(&m1_camel), Some("001901".to_string()));

        let m2 = serde_json::json!({"id": 1901});
        assert_eq!(parse_page_id_arg(&m2), Some("001901".to_string()));

        let m3 = serde_json::json!({"id": "pony"});
        assert_eq!(parse_page_id_arg(&m3), Some("pony".to_string()));

        let m4 = serde_json::json!({"page_id": -1});
        assert_eq!(parse_page_id_arg(&m4), None);

        let m5 = serde_json::json!({"page_id": "000000"});
        assert_eq!(parse_page_id_arg(&m5), None);

        let m6 = serde_json::json!({});
        assert_eq!(parse_page_id_arg(&m6), None);
    }
}
