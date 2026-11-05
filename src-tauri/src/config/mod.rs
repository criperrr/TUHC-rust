use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub key: String,
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "has_embed")]
    pub has_embed: bool,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub future: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosedTab {
    pub key: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabData {
    #[serde(alias = "active_tab_key")]
    pub active_tab_key: String,
    pub tabs: HashMap<String, Tab>,
    #[serde(alias = "tab_list")]
    pub tab_list: Vec<String>,
    #[serde(default, alias = "sorted_tab_list")]
    pub sorted_tab_list: Vec<String>,
    #[serde(default, alias = "closed_tab_list")]
    pub closed_tab_list: Vec<ClosedTab>,
}

impl Default for TabData {
    fn default() -> Self {
        let mut tabs = HashMap::new();
        tabs.insert(
            "000".to_string(),
            Tab {
                key: "000".to_string(),
                url: "/".to_string(),
                title: "The Unofficial Homestuck Collection".to_string(),
                has_embed: false,
                history: Vec::new(),
                future: Vec::new(),
            },
        );
        Self {
            active_tab_key: "000".to_string(),
            tabs,
            tab_list: vec!["000".to_string()],
            sorted_tab_list: vec!["000".to_string()],
            closed_tab_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub key: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SaveData {
    pub saves: HashMap<String, Bookmark>,
    #[serde(default, alias = "save_list")]
    pub save_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReaderSettings {
    #[serde(default = "default_reader_current")]
    pub current: String,
    #[serde(default = "default_reader_limit")]
    pub limit: String,
}

fn default_reader_current() -> String {
    "001901".to_string()
}
fn default_reader_limit() -> String {
    "001902".to_string()
}

impl Default for NewReaderSettings {
    fn default() -> Self {
        Self {
            current: default_reader_current(),
            limit: default_reader_limit(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextOverrideSettings {
    #[serde(default, alias = "font_family")]
    pub font_family: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default = "default_font_size", alias = "font_size")]
    pub font_size: f64,
    #[serde(default = "default_font_size", alias = "line_height")]
    pub line_height: f64,
    #[serde(default, alias = "paragraph_spacing")]
    pub paragraph_spacing: bool,
    #[serde(default, alias = "high_contrast")]
    pub high_contrast: bool,
}

fn default_font_size() -> f64 {
    1.0
}

impl Default for TextOverrideSettings {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            bold: false,
            font_size: 1.0,
            line_height: 1.0,
            paragraph_spacing: false,
            high_contrast: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default, alias = "new_reader")]
    pub new_reader: NewReaderSettings,
    #[serde(default = "default_true")]
    pub notifications: bool,
    #[serde(default, alias = "sub_notifications")]
    pub sub_notifications: bool,
    #[serde(default = "default_true", alias = "show_address_bar")]
    pub show_address_bar: bool,
    #[serde(default = "default_true", alias = "url_tooltip")]
    pub url_tooltip: bool,
    #[serde(default, alias = "switch_to_new_tabs")]
    pub switch_to_new_tabs: bool,
    #[serde(default = "default_true", alias = "force_scroll_bar")]
    pub force_scroll_bar: bool,
    #[serde(default, alias = "hide_fullscreen_header")]
    pub hide_fullscreen_header: bool,
    #[serde(default = "default_true", alias = "smooth_scrolling")]
    pub smooth_scrolling: bool,
    #[serde(default = "default_true", alias = "pixel_scaling")]
    pub pixel_scaling: bool,
    #[serde(default, alias = "mspa_mode")]
    pub mspa_mode: bool,
    #[serde(default = "default_true", alias = "bandcamp_embed")]
    pub bandcamp_embed: bool,
    #[serde(default = "default_true", alias = "allow_sys_update_notifs")]
    pub allow_sys_update_notifs: bool,
    #[serde(default, alias = "last_checked_update")]
    pub last_checked_update: String,
    #[serde(default, alias = "dev_mode")]
    pub dev_mode: bool,
    #[serde(default, alias = "enable_hardware_acceleration")]
    pub enable_hardware_acceleration: bool,
    #[serde(default, alias = "use_system_window_decorations")]
    pub use_system_window_decorations: bool,
    #[serde(default = "default_true", alias = "use_tabbed_browsing")]
    pub use_tabbed_browsing: bool,
    #[serde(default = "default_theme", alias = "theme_override")]
    pub theme_override: String,
    #[serde(
        default = "default_theme",
        rename = "themeOverrideUI",
        alias = "themeOverrideUi",
        alias = "theme_override_ui"
    )]
    pub theme_override_ui: String,
    #[serde(default, alias = "force_theme_override")]
    pub force_theme_override: bool,
    #[serde(
        default,
        rename = "forceThemeOverrideUI",
        alias = "forceThemeOverrideUi",
        alias = "force_theme_override_ui"
    )]
    pub force_theme_override_ui: bool,
    #[serde(default, alias = "text_override")]
    pub text_override: TextOverrideSettings,
    #[serde(default = "default_true", alias = "arrow_nav")]
    pub arrow_nav: bool,
    #[serde(default, alias = "open_logs")]
    pub open_logs: bool,
    #[serde(default = "default_true", alias = "hq_audio")]
    pub hq_audio: bool,
    #[serde(default = "default_true", alias = "js_flashes")]
    pub js_flashes: bool,
    #[serde(default, alias = "reduced_motion")]
    pub reduced_motion: bool,
    #[serde(default = "default_true")]
    pub credits: bool,
    #[serde(default, alias = "fast_forward")]
    pub fast_forward: bool,
    #[serde(default = "default_true")]
    pub retcon1: bool,
    #[serde(default = "default_true")]
    pub retcon2: bool,
    #[serde(default = "default_true")]
    pub retcon3: bool,
    #[serde(default = "default_true")]
    pub retcon4: bool,
    #[serde(default = "default_true")]
    pub retcon5: bool,
    #[serde(default = "default_true")]
    pub retcon6: bool,
    #[serde(default)]
    pub bolin: bool,
    #[serde(default)]
    pub soluslunes: bool,
    #[serde(default)]
    pub unpeachy: bool,
    #[serde(default, rename = "pxsTavros", alias = "pxs_tavros")]
    pub pxs_tavros: bool,
    #[serde(default, alias = "cursed_history")]
    pub cursed_history: bool,
    #[serde(default, alias = "ruffle_fallback")]
    pub ruffle_fallback: bool,
    #[serde(default, alias = "mod_list_enabled")]
    pub mod_list_enabled: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}
fn default_theme() -> String {
    "default".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            new_reader: NewReaderSettings::default(),
            notifications: true,
            sub_notifications: false,
            show_address_bar: true,
            url_tooltip: true,
            switch_to_new_tabs: false,
            force_scroll_bar: true,
            hide_fullscreen_header: false,
            smooth_scrolling: true,
            pixel_scaling: true,
            mspa_mode: false,
            bandcamp_embed: true,
            allow_sys_update_notifs: true,
            last_checked_update: String::new(),
            dev_mode: false,
            enable_hardware_acceleration: false,
            use_system_window_decorations: false,
            use_tabbed_browsing: true,
            theme_override: default_theme(),
            theme_override_ui: default_theme(),
            force_theme_override: false,
            force_theme_override_ui: false,
            text_override: TextOverrideSettings::default(),
            arrow_nav: true,
            open_logs: false,
            hq_audio: true,
            js_flashes: true,
            reduced_motion: false,
            credits: true,
            fast_forward: false,
            retcon1: true,
            retcon2: true,
            retcon3: true,
            retcon4: true,
            retcon5: true,
            retcon6: true,
            bolin: false,
            soluslunes: false,
            unpeachy: false,
            pxs_tavros: false,
            cursed_history: false,
            ruffle_fallback: false,
            mod_list_enabled: Vec::new(),
            extra: serde_json::Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default, rename = "__internal__")]
    pub internal: serde_json::Value,
    #[serde(default, alias = "last_migrated_version")]
    pub last_migrated_version: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
    #[serde(default, alias = "asset_dir")]
    pub asset_dir: Option<String>,
    #[serde(default, alias = "tab_data")]
    pub tab_data: TabData,
    #[serde(default, alias = "save_data")]
    pub save_data: SaveData,
    #[serde(default)]
    pub settings: AppSettings,
    #[serde(default = "default_app_version", alias = "app_version")]
    pub app_version: String,
}

fn default_app_version() -> String {
    "2.8.1".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            internal: serde_json::json!({ "migrations": { "version": "2.8.1" } }),
            last_migrated_version: Some("2.8.1".to_string()),
            timestamp: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            ),
            asset_dir: Some("/home/criper/Downloads/Asset Pack V2".to_string()),
            tab_data: TabData::default(),
            save_data: SaveData::default(),
            settings: AppSettings::default(),
            app_version: default_app_version(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("unofficial-homestuck-collection")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            let default_config = Self::default();
            let _ = default_config.save();
            return default_config;
        }

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    log::error!("Failed to parse config.json, using defaults: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                log::error!("Failed to read config.json: {}", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        use std::io::Write;
        let path = Self::config_path();
        let parent = path.parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Missing parent directory")
        })?;
        fs::create_dir_all(parent)?;

        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut temp_file = tempfile::Builder::new()
            .prefix("config-")
            .suffix(".tmp")
            .tempfile_in(parent)?;

        temp_file.write_all(json_str.as_bytes())?;
        temp_file.flush()?;
        temp_file.as_file().sync_all()?;
        temp_file.persist(&path).map_err(|e| e.error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_and_roundtrip() {
        let mut cfg = AppConfig::default();
        cfg.settings
            .mod_list_enabled
            .push("Homestuck PT-BR".to_string());
        cfg.tab_data.active_tab_key = "001".to_string();

        let json = serde_json::to_string(&cfg).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.tab_data.active_tab_key, "001");
        assert_eq!(loaded.settings.mod_list_enabled, vec!["Homestuck PT-BR"]);
        assert!(loaded.settings.notifications);
        assert_eq!(loaded.app_version, "2.8.1");
    }
}
