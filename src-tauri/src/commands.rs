use crate::config::AppConfig;
use crate::data::story::{StoryPage, StoryPageSummary};
use crate::mods::ModMetadata;
use crate::rpc::DiscordStatus;
use crate::AppState;

#[tauri::command]
pub fn get_app_info(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "version": "2.8.1",
        "asset_dir": state.0.asset_dir.to_string_lossy(),
        "asset_port": state.0.asset_port,
        "overlay_generation": state.0.vfs.generation(),
        "overlayGeneration": state.0.vfs.generation(),
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

    let page = state
        .0
        .store
        .get_story_page(trimmed)
        .await
        .ok_or_else(|| format!("Page '{}' not found", trimmed))?;

    if state.0.config.read().await.settings.enable_discord_rpc {
        let _ = state
            .0
            .discord_rpc
            .update_reading_presence(&page.page_id, &page.title)
            .await;
    }

    Ok(page)
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

    if state.0.discord_rpc.get_status().await.enabled != new_config.settings.enable_discord_rpc {
        let _ = state
            .0
            .discord_rpc
            .set_enabled(new_config.settings.enable_discord_rpc)
            .await;
    }

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
        "enabled": enabled,
        "overlay_generation": state.0.vfs.generation(),
        "overlayGeneration": state.0.vfs.generation(),
    }))
}

#[tauri::command]
pub async fn get_mod_hooks(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mods_lock = state.0.mods.read().await;
    Ok(serde_json::json!({
        "vueHooks": mods_lock.get_active_vue_hooks(),
        "overlayGeneration": state.0.vfs.generation(),
        "overlay_generation": state.0.vfs.generation(),
    }))
}

#[tauri::command]
pub async fn evaluate_log_button_text(
    state: tauri::State<'_, AppState>,
    content: String,
    log_hidden: Option<bool>,
) -> Result<String, String> {
    let mods_lock = state.0.mods.read().await;
    Ok(mods_lock.evaluate_log_button_text(&content, log_hidden.unwrap_or(true)))
}

#[tauri::command]
pub async fn get_discord_status(
    state: tauri::State<'_, AppState>,
) -> Result<DiscordStatus, String> {
    Ok(state.0.discord_rpc.get_status().await)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn set_discord_presence(
    state: tauri::State<'_, AppState>,
    mode: Option<String>,
    page_id: Option<String>,
    page_title: Option<String>,
    track_id: Option<String>,
    track_title: Option<String>,
    album_name: Option<String>,
    artist: Option<String>,
    playing: Option<bool>,
) -> Result<DiscordStatus, String> {
    let mode_str = mode.as_deref().unwrap_or("reading");
    match mode_str {
        "reading" => {
            let pid = page_id.unwrap_or_else(|| "001901".to_string());
            let title = match page_title {
                Some(t) => t,
                None => state
                    .0
                    .store
                    .get_story_page(&pid)
                    .await
                    .map(|p| p.title)
                    .unwrap_or_else(|| "Homestuck".to_string()),
            };
            Ok(state
                .0
                .discord_rpc
                .update_reading_presence(&pid, &title)
                .await)
        }
        "music" => {
            let title = track_title.unwrap_or_else(|| "Homestuck Music".to_string());
            let album = album_name.unwrap_or_default();
            let is_playing = playing.unwrap_or(true);
            Ok(state
                .0
                .discord_rpc
                .update_music_presence(&title, &album, artist.as_deref(), track_id, is_playing)
                .await)
        }
        "clear" => Ok(state.0.discord_rpc.clear_presence().await),
        _ => Ok(state.0.discord_rpc.get_status().await),
    }
}

#[tauri::command]
pub async fn toggle_discord_rpc(
    state: tauri::State<'_, AppState>,
    enabled: Option<bool>,
) -> Result<DiscordStatus, String> {
    let current = state.0.discord_rpc.get_status().await.enabled;
    let target = enabled.unwrap_or(!current);
    let status = state.0.discord_rpc.set_enabled(target).await;
    {
        let mut cfg = state.0.config.write().await;
        cfg.settings.enable_discord_rpc = target;
        let _ = cfg.save();
    }
    Ok(status)
}

#[tauri::command]
pub fn trim_memory() {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::malloc_trim(0);
    }
}

#[tauri::command]
pub fn client_log(level: String, message: String) {
    eprintln!("[WEBVIEW-{}] {}", level.to_uppercase(), message);
}

#[tauri::command]
pub fn win_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn win_maximize(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub fn win_close(window: tauri::Window) {
    let _ = window.close();
}
