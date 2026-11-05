use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use tuhc_lib::config::AppConfig;
use tuhc_lib::data::ArchiveStore;
use tuhc_lib::mods::ModRegistry;
use tuhc_lib::server::{AppStateInner, Vfs};

#[tokio::test]
async fn test_rapid_concurrent_mod_toggles_and_page_fetches() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let asset_dir = std::env::var("TUHC_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/criper/Downloads/Asset Pack V2"));
    if !asset_dir.exists() {
        eprintln!("Skipping test_rapid_concurrent_mod_toggles_and_page_fetches: asset directory {:?} does not exist", asset_dir);
        return;
    }
    let app_config = AppConfig {
        asset_dir: Some(asset_dir.to_string_lossy().to_string()),
        ..Default::default()
    };
    let _ = app_config.save();

    let store = Arc::new(ArchiveStore::load_from_dir(&asset_dir));
    let mod_registry =
        ModRegistry::load_from_dir(&asset_dir, &app_config.settings.mod_list_enabled);
    let vfs = Arc::new(Vfs::new(asset_dir.clone()));

    let state = AppStateInner {
        config: Arc::new(RwLock::new(app_config)),
        asset_port: 45100,
        asset_dir: asset_dir.clone(),
        store: store.clone(),
        mods: Arc::new(RwLock::new(mod_registry)),
        vfs: vfs.clone(),
    };

    let stop = Arc::new(AtomicBool::new(false));
    let fetch_count = Arc::new(AtomicUsize::new(0));
    let asset_count = Arc::new(AtomicUsize::new(0));
    let toggle_count = Arc::new(AtomicUsize::new(0));
    let inconsistency_count = Arc::new(AtomicUsize::new(0));

    // Spawn 4 background tasks rapidly reading pages and assets
    let mut reader_handles = Vec::new();
    for _reader_id in 0..4 {
        let state_clone = state.clone();
        let stop_clone = Arc::clone(&stop);
        let f_cnt = Arc::clone(&fetch_count);
        let a_cnt = Arc::clone(&asset_count);

        let handle = tokio::spawn(async move {
            while !stop_clone.load(Ordering::Relaxed) {
                // Fetch page 001901 (translated by PT-BR when active)
                if let Some(page) = state_clone.store.get_story_page("001901").await {
                    // Page title is either "Homestuck" or Portuguese translation "Homestuck PT-BR" / translated content
                    assert!(
                        page.title == "Homestuck" || page.title.contains("Homestuck"),
                        "Page title corrupted: {}",
                        page.title
                    );
                    f_cnt.fetch_add(1, Ordering::Relaxed);
                }

                // Resolve asset 00002.gif (overridden by PT-BR when active)
                if let Some(path) = state_clone.vfs.resolve("/storyfiles/hs2/00002.gif").await {
                    assert!(path.is_file(), "Resolved asset path must exist on disk");
                    a_cnt.fetch_add(1, Ordering::Relaxed);
                }

                tokio::task::yield_now().await;
            }
        });
        reader_handles.push(handle);
    }

    // Spawn 2 tasks rapidly toggling "Homestuck PT-BR" on and off
    let mut toggle_handles = Vec::new();
    for task_id in 0..2 {
        let state_clone = state.clone();
        let stop_clone = Arc::clone(&stop);
        let t_cnt = Arc::clone(&toggle_count);

        let handle = tokio::spawn(async move {
            let mut enabled = task_id % 2 == 0;
            for _ in 0..50 {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                enabled = !enabled;

                // Emulate toggle_mod IPC logic
                let (success, descriptors, patches) = {
                    let mut mods_lock = state_clone.mods.write().await;
                    match mods_lock.toggle_mod("Homestuck PT-BR", enabled) {
                        Ok(_) => {
                            let descriptors = mods_lock.get_active_descriptors();
                            let patches = mods_lock.load_active_patches();
                            (true, descriptors, patches)
                        }
                        Err(_) => (false, vec![], std::collections::HashMap::new()),
                    }
                };

                if success {
                    let _ = state_clone.vfs.rebuild_overlay(descriptors).await;
                    if patches.is_empty() {
                        state_clone.store.story.reset_to_base().await;
                    } else {
                        state_clone.store.story.apply_patches(&patches).await;
                    }

                    let mut cfg = state_clone.config.write().await;
                    let list = &mut cfg.settings.mod_list_enabled;
                    if enabled {
                        if !list.contains(&"Homestuck PT-BR".to_string()) {
                            list.push("Homestuck PT-BR".to_string());
                        }
                    } else {
                        list.retain(|id| id != "Homestuck PT-BR");
                    }
                    let _ = cfg.save();
                }

                t_cnt.fetch_add(1, Ordering::Relaxed);
                tokio::task::yield_now().await;
            }
        });
        toggle_handles.push(handle);
    }

    // Wait for toggle tasks to finish
    for handle in toggle_handles {
        handle.await.unwrap();
    }

    // Stop reader tasks
    stop.store(true, Ordering::Relaxed);
    for handle in reader_handles {
        handle.await.unwrap();
    }

    println!(
        "Concurrent mod stress test results: toggles={}, page_fetches={}, asset_resolutions={}",
        toggle_count.load(Ordering::Relaxed),
        fetch_count.load(Ordering::Relaxed),
        asset_count.load(Ordering::Relaxed)
    );

    // Now check if internal state is mutually consistent!
    let mod_enabled_in_registry = state.mods.read().await.is_enabled("Homestuck PT-BR");
    let mod_in_config = state
        .config
        .read()
        .await
        .settings
        .mod_list_enabled
        .contains(&"Homestuck PT-BR".to_string());
    let vfs_overlay_count = state.vfs.overlay_count().await;
    let page = state.store.get_story_page("001901").await.unwrap();
    let page_is_translated = page.content.contains("Um jovem rapaz");

    println!(
        "Final state consistency check:\n  registry_enabled: {}\n  config_enabled: {}\n  vfs_overlay_count: {}\n  page_is_translated: {}",
        mod_enabled_in_registry, mod_in_config, vfs_overlay_count, page_is_translated
    );

    if mod_enabled_in_registry != mod_in_config {
        eprintln!("STATE DESYNC: registry_enabled != config_enabled");
        inconsistency_count.fetch_add(1, Ordering::Relaxed);
    }

    if mod_enabled_in_registry && vfs_overlay_count == 0 {
        eprintln!("STATE DESYNC: registry says enabled but vfs overlay is empty");
        inconsistency_count.fetch_add(1, Ordering::Relaxed);
    }

    if !mod_enabled_in_registry && vfs_overlay_count > 0 {
        eprintln!("STATE DESYNC: registry says disabled but vfs overlay is active");
        inconsistency_count.fetch_add(1, Ordering::Relaxed);
    }

    if mod_enabled_in_registry != page_is_translated {
        eprintln!("STATE DESYNC: registry_enabled != page_is_translated");
        inconsistency_count.fetch_add(1, Ordering::Relaxed);
    }

    // Zero panic check: readers and togglers must have executed successfully without panicking
    assert!(fetch_count.load(Ordering::Relaxed) > 0);
    assert!(asset_count.load(Ordering::Relaxed) > 0);
    assert!(toggle_count.load(Ordering::Relaxed) > 0);

    // Fail if state desync occurred
    assert_eq!(
        inconsistency_count.load(Ordering::Relaxed),
        0,
        "Mod state desynchronization detected under concurrent toggling!"
    );
}
