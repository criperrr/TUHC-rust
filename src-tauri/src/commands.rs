use crate::config::AppConfig;
use crate::data::story::{StoryPage, StoryPageSummary};
use crate::mods::ModMetadata;
use crate::AppState;

#[tauri::command]
pub fn get_app_info(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "version": "2.8.1",
        "asset_dir": state.0.asset_dir.to_string_lossy(),
        "asset_port": state.0.asset_port,
    }))
}

#[tauri::command]
pub async fn get_story_page(
    state: tauri::State<'_, AppState>,
    page_id: String,
) -> Result<StoryPage, String> {
    let trimmed = page_id.trim();
    if trimmed.is_empty() || trimmed == "000000" || trimmed.starts_with('-') {
        return Err("Page not found".to_string());
    }

    state
        .0
        .store
        .get_story_page(trimmed)
        .await
        .ok_or_else(|| format!("Page '{}' not found", trimmed))
}

#[tauri::command]
pub async fn get_story_range(
    state: tauri::State<'_, AppState>,
    start: usize,
    count: usize,
) -> Result<Vec<StoryPageSummary>, String> {
    Ok(state.0.store.get_story_range(start, count).await)
}

#[tauri::command]
pub fn get_music_data(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(state.0.store.get_music_raw())
}

#[tauri::command]
pub async fn load_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = state.0.config.read().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn save_config(
    state: tauri::State<'_, AppState>,
    new_config: AppConfig,
) -> Result<(), String> {
    new_config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;

    {
        let mut cfg = state.0.config.write().await;
        *cfg = new_config.clone();
    }

    // Sync active mods and VFS overlay
    {
        let mut mods_lock = state.0.mods.write().await;
        mods_lock.rescan(&new_config.settings.mod_list_enabled);
        let descriptors = mods_lock.get_active_descriptors();
        let _ = state.0.vfs.rebuild_overlay(descriptors).await;
        let patches = mods_lock.load_active_patches();
        if patches.is_empty() {
            state.0.store.story.reset_to_base().await;
        } else {
            state.0.store.story.apply_patches(&patches).await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_available_mods(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ModMetadata>, String> {
    Ok(state.0.mods.read().await.get_available_mods())
}

#[tauri::command]
pub async fn toggle_mod(
    state: tauri::State<'_, AppState>,
    mod_id: String,
    enabled: bool,
) -> Result<serde_json::Value, String> {
    let (success, descriptors, patches) = {
        let mut mods_lock = state.0.mods.write().await;
        mods_lock.toggle_mod(&mod_id, enabled).map(|_| {
            let descriptors = mods_lock.get_active_descriptors();
            let patches = mods_lock.load_active_patches();
            (true, descriptors, patches)
        })?
    };

    if success {
        let _ = state.0.vfs.rebuild_overlay(descriptors).await;
        if patches.is_empty() {
            state.0.store.story.reset_to_base().await;
        } else {
            state.0.store.story.apply_patches(&patches).await;
        }

        // Keep config in sync
        let mut cfg = state.0.config.write().await;
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

    Ok(serde_json::json!({
        "status": "ok",
        "mod_id": mod_id,
        "enabled": enabled
    }))
}

#[tauri::command]
pub fn trim_memory() {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::malloc_trim(0);
    }
}
