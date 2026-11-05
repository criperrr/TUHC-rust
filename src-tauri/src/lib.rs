pub mod commands;
pub mod config;
pub mod data;
pub mod mods;
pub mod server;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

use config::AppConfig;
use data::ArchiveStore;
use mods::ModRegistry;
use server::{AppStateInner, Vfs};

pub struct AppState(pub Arc<AppStateInner>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_config = AppConfig::load();
            let asset_dir = app_config
                .asset_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/home/criper/Downloads/Asset Pack V2"));

            // Ephemeral loopback port channel
            let (port_tx, port_rx) = tokio::sync::oneshot::channel::<u16>();

            let store = Arc::new(ArchiveStore::load_from_dir(&asset_dir));
            let mod_registry =
                ModRegistry::load_from_dir(&asset_dir, &app_config.settings.mod_list_enabled);
            let vfs = Arc::new(Vfs::new(asset_dir.clone()));

            let descriptors = mod_registry.get_active_descriptors();
            let patches = mod_registry.load_active_patches();

            let vfs_init = Arc::clone(&vfs);
            let store_init = Arc::clone(&store);

            tauri::async_runtime::block_on(async {
                let _ = vfs_init.rebuild_overlay(descriptors).await;
                if !patches.is_empty() {
                    store_init.story.apply_patches(&patches).await;
                }
            });

            // Start Axum loopback server on 127.0.0.1:0
            let app_config_clone = app_config.clone();
            let asset_dir_clone = asset_dir.clone();
            let store_clone = Arc::clone(&store);
            let vfs_clone = Arc::clone(&vfs);
            let mod_registry_arc = Arc::new(RwLock::new(mod_registry));
            let mod_registry_clone = Arc::clone(&mod_registry_arc);

            tauri::async_runtime::spawn(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("Failed to bind ephemeral loopback socket on 127.0.0.1:0");

                let port = listener
                    .local_addr()
                    .expect("Failed to read local socket address")
                    .port();

                let inner = AppStateInner {
                    config: Arc::new(RwLock::new(app_config_clone)),
                    asset_port: port,
                    asset_dir: asset_dir_clone,
                    store: store_clone,
                    mods: mod_registry_clone,
                    vfs: vfs_clone,
                };

                let _ = port_tx.send(port);

                let app = axum::Router::new()
                    .route("/ipc", axum::routing::post(server::handle_ipc_post))
                    .route("/ipc/{cmd}", axum::routing::get(server::handle_ipc_get))
                    .route(
                        "/assets/{*path}",
                        axum::routing::any(server::fallback_router),
                    )
                    .fallback(server::fallback_router)
                    .layer(axum::middleware::from_fn(server::cors_middleware))
                    .with_state(inner);

                axum::serve(listener, app)
                    .await
                    .expect("Axum server crashed");
            });

            let asset_port = tauri::async_runtime::block_on(async {
                port_rx.await.expect("Failed to receive ephemeral port")
            });

            let inner = Arc::new(AppStateInner {
                config: Arc::new(RwLock::new(app_config)),
                asset_port,
                asset_dir,
                store,
                mods: mod_registry_arc,
                vfs,
            });

            app.manage(AppState(inner));

            // WebKitGTK memory optimization settings
            #[cfg(target_os = "linux")]
            {
                use webkit2gtk::{SettingsExt, WebContextExt, WebViewExt};
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.with_webview(|webview| {
                        let inner = webview.inner();
                        if let Some(context) = inner.context() {
                            context.set_cache_model(webkit2gtk::CacheModel::DocumentViewer);
                        }
                        if let Some(settings) = inner.settings() {
                            settings.set_enable_page_cache(false);
                            settings.set_enable_offline_web_application_cache(false);
                            settings.set_enable_smooth_scrolling(false);
                        }
                    });
                }
            }

            // Trim glibc allocator heap pages back to OS kernel
            #[cfg(target_os = "linux")]
            unsafe {
                libc::malloc_trim(0);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_story_page,
            commands::get_story_range,
            commands::get_music_data,
            commands::load_config,
            commands::save_config,
            commands::get_available_mods,
            commands::toggle_mod,
            commands::trim_memory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tuhc application");
}
