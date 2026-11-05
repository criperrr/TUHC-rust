use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use tuhc_lib::config::AppConfig;
use tuhc_lib::data::ArchiveStore;
use tuhc_lib::mods::ModRegistry;
use tuhc_lib::server::{AppStateInner, Vfs};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);

    let app_config = AppConfig::load();
    let asset_dir = app_config
        .asset_dir
        .as_ref()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(tuhc_lib::config::default_asset_dir()));

    let store = Arc::new(ArchiveStore::load_from_dir(&asset_dir));
    let mod_registry =
        ModRegistry::load_from_dir(&asset_dir, &app_config.settings.mod_list_enabled);
    let vfs = Arc::new(Vfs::new(asset_dir.clone()));

    let descriptors = mod_registry.get_active_descriptors();
    let patches = mod_registry.load_active_patches();

    let _ = vfs.rebuild_overlay(descriptors).await;
    if !patches.is_empty() {
        store.story.apply_patches(&patches).await;
    }

    let bind_addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind on {}: {}", bind_addr, e));

    let actual_port = listener.local_addr().unwrap().port();

    let inner = AppStateInner {
        config: Arc::new(RwLock::new(app_config)),
        asset_port: actual_port,
        asset_dir,
        store,
        mods: Arc::new(RwLock::new(mod_registry)),
        vfs,
    };

    println!(
        "[*] TUHC Backend Server listening on http://127.0.0.1:{}",
        actual_port
    );

    let app = axum::Router::new()
        .route(
            "/ipc",
            axum::routing::post(tuhc_lib::server::handle_ipc_post),
        )
        .route(
            "/ipc/{cmd}",
            axum::routing::get(tuhc_lib::server::handle_ipc_get),
        )
        .route(
            "/assets/{*path}",
            axum::routing::any(tuhc_lib::server::fallback_router),
        )
        .fallback(tuhc_lib::server::fallback_router)
        .layer(axum::middleware::from_fn(tuhc_lib::server::cors_middleware))
        .with_state(inner);

    axum::serve(listener, app).await.unwrap();
}
