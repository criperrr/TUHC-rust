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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::data::ArchiveStore;
use crate::mods::ModRegistry;
pub use vfs::{ActiveModDescriptor, Vfs};

#[derive(Clone)]
pub struct AppStateInner {
    pub config: Arc<RwLock<AppConfig>>,
    pub asset_port: u16,
    pub asset_dir: PathBuf,
    pub store: Arc<ArchiveStore>,
    pub mods: Arc<RwLock<ModRegistry>>,
    pub vfs: Arc<Vfs>,
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
    handler::add_cors_headers(headers);
    res
}

async fn serve_html_shell() -> Response {
    let content = b"<!DOCTYPE html><html><head><title>The Unofficial Homestuck Collection</title></head><body><div id='app'>TUHC Webview Shell</div></body></html>";
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
    handler::add_cors_headers(headers);
    res
}

async fn serve_main_css() -> Response {
    let css_data =
        b"body { font-family: 'Courier New', monospace; background: #000; color: #fff; }";
    let mut res = (StatusCode::OK, css_data.as_slice()).into_response();
    let headers = res.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/css"));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&css_data.len().to_string()).unwrap(),
    );
    handler::add_cors_headers(headers);
    res
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
            });
            json_response(StatusCode::OK, info)
        }
        "get_story_page" => {
            let page_id = if let Some(s) = args.get("page_id").and_then(|v| v.as_str()) {
                s.to_string()
            } else if let Some(s) = args.get("id").and_then(|v| v.as_str()) {
                s.to_string()
            } else if let Some(n) = args.get("page_id").and_then(|v| v.as_i64()) {
                if n < 0 {
                    return json_response(
                        StatusCode::NOT_FOUND,
                        serde_json::json!({"error": "Page not found"}),
                    );
                }
                format!("{:06}", n)
            } else {
                return json_response(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": "Missing page_id"}),
                );
            };

            let page_id_trimmed = page_id.trim();
            if page_id_trimmed.is_empty()
                || page_id_trimmed == "000000"
                || page_id_trimmed.starts_with('-')
            {
                return json_response(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": "Page not found"}),
                );
            }

            match state.store.get_story_page(page_id_trimmed).await {
                Some(page) => match serde_json::to_value(&page) {
                    Ok(v) => json_response(StatusCode::OK, v),
                    Err(e) => json_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        serde_json::json!({"error": e.to_string()}),
                    ),
                },
                None => json_response(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": format!("Page '{}' not found", page_id_trimmed)}),
                ),
            }
        }
        "get_story_range" => {
            let start = args.get("start").and_then(|v| v.as_u64()).unwrap_or(1901) as usize;
            let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

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
            } else if let Some(m) = args.get("id").and_then(|v| v.as_str()) {
                m.to_string()
            } else {
                return json_response(
                    StatusCode::NOT_FOUND,
                    serde_json::json!({"error": "Missing or non-string mod_id"}),
                );
            };

            let enabled = args
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

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
                serde_json::json!({"status": "ok", "mod_id": mod_id, "enabled": enabled}),
            )
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
        return serve_html_shell().await;
    }
    if path == "/styles/main.css" {
        return serve_main_css().await;
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
