use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

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
    new_config: Option<AppConfig>,
    config: Option<AppConfig>,
    #[allow(non_snake_case)] newConfig: Option<AppConfig>,
) -> Result<(), String> {
    let mut new_config = new_config.or(config).or(newConfig).ok_or_else(|| {
        "Missing config payload (expected 'new_config', 'config', or 'newConfig')".to_string()
    })?;

    // Keep enabled_mods and settings.mod_list_enabled in sync
    if new_config.settings.mod_list_enabled.is_empty() {
        if let Some(ref em) = new_config.enabled_mods {
            if !em.is_empty() {
                new_config.settings.mod_list_enabled = em.clone();
            }
        }
    }
    if new_config.enabled_mods.is_none() {
        new_config.enabled_mods = Some(new_config.settings.mod_list_enabled.clone());
    }

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
        cfg.enabled_mods = Some(list.clone());
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugMetrics {
    pub rss_mb: f64,
    pub cpu_pct: f64,
    pub threads: usize,
    pub uptime_secs: u64,
}

static PROCESS_START_TIME: OnceLock<Instant> = OnceLock::new();

struct CpuTracker {
    last_ticks: u64,
    last_instant: Instant,
    last_pct: f64,
}

static CPU_TRACKER: Mutex<Option<CpuTracker>> = Mutex::new(None);

pub fn collect_debug_metrics() -> DebugMetrics {
    let start_instant = PROCESS_START_TIME.get_or_init(Instant::now);
    let uptime_secs = start_instant.elapsed().as_secs();

    let mut rss_mb = 0.0;
    let mut cpu_pct = 0.0;
    let mut threads = 1;

    #[cfg(target_os = "linux")]
    {
        // 1. Read Resident Set Size from /proc/self/statm
        if let Ok(statm_content) = std::fs::read_to_string("/proc/self/statm") {
            let mut parts = statm_content.split_whitespace();
            // parts: [size, resident, shared, text, lib, data, dt]
            if let Some(resident_str) = parts.nth(1) {
                if let Ok(resident_pages) = resident_str.parse::<u64>() {
                    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
                    let page_size_bytes = if page_size > 0 {
                        page_size as u64
                    } else {
                        4096
                    };
                    let bytes = resident_pages.saturating_mul(page_size_bytes);
                    let mb = (bytes as f64) / (1024.0 * 1024.0);
                    rss_mb = (mb * 10.0).round() / 10.0;
                }
            }
        }

        // 2. Read utime, stime, num_threads from /proc/self/stat
        if let Ok(stat_content) = std::fs::read_to_string("/proc/self/stat") {
            if let Some(idx) = stat_content.rfind(')') {
                let remainder = &stat_content[idx + 1..];
                let tokens: Vec<&str> = remainder.split_whitespace().collect();
                // tokens[0] is state (field 3)
                // tokens[11] is utime (field 14)
                // tokens[12] is stime (field 15)
                // tokens[17] is num_threads (field 20)
                if tokens.len() > 17 {
                    let utime = tokens[11].parse::<u64>().unwrap_or(0);
                    let stime = tokens[12].parse::<u64>().unwrap_or(0);
                    threads = tokens[17].parse::<usize>().unwrap_or(1);

                    let current_ticks = utime.saturating_add(stime);
                    let clk_tck_raw = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
                    let clk_tck = if clk_tck_raw > 0 {
                        clk_tck_raw as f64
                    } else {
                        100.0
                    };

                    let now = Instant::now();
                    if let Ok(mut lock) = CPU_TRACKER.lock() {
                        match *lock {
                            Some(ref mut tracker) => {
                                let elapsed =
                                    now.duration_since(tracker.last_instant).as_secs_f64();
                                if elapsed >= 0.15 {
                                    let ticks_diff =
                                        current_ticks.saturating_sub(tracker.last_ticks);
                                    let cpu_time_secs = (ticks_diff as f64) / clk_tck;
                                    let raw_pct = (cpu_time_secs / elapsed) * 100.0;
                                    tracker.last_ticks = current_ticks;
                                    tracker.last_instant = now;
                                    tracker.last_pct = (raw_pct * 10.0).round() / 10.0;
                                }
                                cpu_pct = tracker.last_pct;
                            }
                            None => {
                                *lock = Some(CpuTracker {
                                    last_ticks: current_ticks,
                                    last_instant: now,
                                    last_pct: 0.0,
                                });
                                cpu_pct = 0.0;
                            }
                        }
                    }
                }
            }
        }
    }

    DebugMetrics {
        rss_mb,
        cpu_pct,
        threads,
        uptime_secs,
    }
}

#[tauri::command]
pub fn get_debug_metrics() -> Result<DebugMetrics, String> {
    Ok(collect_debug_metrics())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_debug_metrics() {
        let metrics = collect_debug_metrics();
        assert!(metrics.threads >= 1);
        #[cfg(target_os = "linux")]
        {
            assert!(metrics.rss_mb >= 0.0);
            assert!(metrics.cpu_pct >= 0.0);
        }
    }
}
