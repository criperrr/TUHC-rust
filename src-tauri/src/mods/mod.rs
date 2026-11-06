pub mod parser;
pub mod patcher;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::data::story::StoryPagePatch;
use crate::server::vfs::ActiveModDescriptor;
pub use parser::{evaluate_log_button_text as evaluate_log_button_text_fn, VueHookDescriptor};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModMetadata {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: f64,
    pub description: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub has_trees: bool,
    #[serde(default)]
    pub has_edit: bool,
    #[serde(default)]
    pub has_hooks: bool,
    #[serde(default)]
    pub vue_hooks: Vec<VueHookDescriptor>,
}

#[derive(Clone)]
struct ModEntry {
    meta: ModMetadata,
    root_dir: PathBuf,
    trees: HashMap<String, String>,
    routes: HashMap<String, String>,
    translation_files: Vec<String>,
}

pub struct ModRegistry {
    mods_dir: PathBuf,
    mods: Vec<ModEntry>,
    enabled_order: Vec<String>,
}

impl ModRegistry {
    pub fn load_from_dir(asset_dir: &Path, enabled_list: &[String]) -> Self {
        let mods_dir = asset_dir.join("mods");
        let mut registry = Self {
            mods_dir,
            mods: Vec::new(),
            enabled_order: enabled_list.to_vec(),
        };
        registry.rescan(enabled_list);
        registry
    }

    pub fn rescan(&mut self, enabled_list: &[String]) {
        self.enabled_order = enabled_list.to_vec();
        self.mods.clear();
        if !self.mods_dir.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&self.mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let mod_js = path.join("mod.js");
                    if mod_js.is_file() {
                        let id = entry.file_name().to_string_lossy().to_string();
                        if let Some(parsed) = parser::parse_mod_js(&mod_js) {
                            self.register_parsed_mod(id, path, parsed, enabled_list);
                        }
                    }
                } else if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("js")
                {
                    let id = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if !id.is_empty() {
                        if let Some(parsed) = parser::parse_mod_js(&path) {
                            let root = path
                                .parent()
                                .map(PathBuf::from)
                                .unwrap_or_else(|| self.mods_dir.clone());
                            self.register_parsed_mod(id, root, parsed, enabled_list);
                        }
                    }
                }
            }
        }
    }

    fn register_parsed_mod(
        &mut self,
        id: String,
        root_dir: PathBuf,
        parsed: parser::ParsedModInfo,
        enabled_list: &[String],
    ) {
        let enabled = enabled_list.contains(&id);
        let has_trees = !parsed.trees.is_empty();
        let has_hooks = !parsed.vue_hooks.is_empty();
        let has_edit = parsed.edit
            || !parsed.translation_files.is_empty()
            || root_dir.join("translation2.json").is_file();
        let desc = parsed
            .description
            .clone()
            .or_else(|| parsed.summary.clone())
            .unwrap_or_else(|| format!("{} mod for Homestuck", parsed.title));

        let meta = ModMetadata {
            id: id.clone(),
            title: parsed.title,
            author: parsed
                .author
                .unwrap_or_else(|| "Unknown Author".to_string()),
            version: parsed.version,
            description: desc,
            summary: parsed.summary,
            enabled,
            has_trees,
            has_edit,
            has_hooks,
            vue_hooks: parsed.vue_hooks,
        };

        self.mods.push(ModEntry {
            meta,
            root_dir,
            trees: parsed.trees,
            routes: parsed.routes,
            translation_files: parsed.translation_files,
        });
    }

    pub fn get_available_mods(&self) -> Vec<ModMetadata> {
        self.mods.iter().map(|m| m.meta.clone()).collect()
    }

    pub fn is_enabled(&self, mod_id: &str) -> bool {
        self.mods
            .iter()
            .any(|m| m.meta.id == mod_id && m.meta.enabled)
    }

    pub fn toggle_mod(&mut self, mod_id: &str, enabled: bool) -> Result<bool, String> {
        if let Some(m) = self.mods.iter_mut().find(|m| m.meta.id == mod_id) {
            m.meta.enabled = enabled;
            if enabled {
                if !self.enabled_order.iter().any(|id| id == mod_id) {
                    self.enabled_order.push(mod_id.to_string());
                }
            } else {
                self.enabled_order.retain(|id| id != mod_id);
            }
            Ok(enabled)
        } else {
            Err(format!("Unknown mod '{}'", mod_id))
        }
    }

    fn sorted_active_mods(&self) -> Vec<&ModEntry> {
        let mut active: Vec<&ModEntry> = self.mods.iter().filter(|m| m.meta.enabled).collect();
        active.sort_by_key(|m| {
            self.enabled_order
                .iter()
                .position(|id| id == &m.meta.id)
                .unwrap_or(usize::MAX)
        });
        active
    }

    pub fn get_enabled_mod_ids(&self) -> Vec<String> {
        self.sorted_active_mods()
            .iter()
            .map(|m| m.meta.id.clone())
            .collect()
    }

    pub fn get_active_descriptors(&self) -> Vec<ActiveModDescriptor> {
        self.sorted_active_mods()
            .iter()
            .map(|m| ActiveModDescriptor {
                id: m.meta.id.clone(),
                root_dir: m.root_dir.clone(),
                trees: m.trees.clone(),
                routes: m.routes.clone(),
            })
            .collect()
    }

    pub fn get_active_vue_hooks(&self) -> Vec<VueHookDescriptor> {
        self.sorted_active_mods()
            .iter()
            .flat_map(|m| m.meta.vue_hooks.clone())
            .collect()
    }

    pub fn has_active_page_text_hooks(&self) -> bool {
        self.mods.iter().filter(|m| m.meta.enabled).any(|m| {
            m.meta
                .vue_hooks
                .iter()
                .any(|h| h.match_name == "pageText" && h.log_button_open_prefix.is_some())
        })
    }

    pub fn evaluate_log_button_text(&self, content: &str, log_hidden: bool) -> String {
        let active = self.has_active_page_text_hooks();
        evaluate_log_button_text_fn(content, log_hidden, active)
    }

    pub fn load_active_patches(&self) -> HashMap<String, StoryPagePatch> {
        let mut combined: HashMap<String, StoryPagePatch> = HashMap::new();
        // Load in reverse order so higher-priority (earlier in enabled_order) overrides lower-priority
        for m in self.sorted_active_mods().into_iter().rev() {
            let mut files_to_load = m.translation_files.clone();
            if files_to_load.is_empty() {
                let default_tp = m.root_dir.join("translation2.json");
                if default_tp.is_file() {
                    files_to_load.push("translation2.json".to_string());
                }
            }

            for file in files_to_load {
                let p = m.root_dir.join(file);
                if p.is_file() {
                    if let Ok(patches) = patcher::load_translation_patches(&p) {
                        for (key, patch) in patches {
                            if let Some(existing) = combined.get_mut(&key) {
                                existing.merge(&patch);
                            } else {
                                combined.insert(key, patch);
                            }
                        }
                    }
                }
            }
        }
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_registry_discovery_and_priority_ordering() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mods_dir = temp_dir.path().join("mods");
        std::fs::create_dir_all(&mods_dir).unwrap();

        // 1. Directory mod "ModA" (Higher Priority)
        let mod_a_dir = mods_dir.join("ModA");
        std::fs::create_dir_all(&mod_a_dir).unwrap();
        let mod_a_js = r#"
module.exports = {
    title: "Mod A High Priority",
    author: "Author A",
    version: 1.0,
    edit: true,
    trees: { "./Assets/": "assets://" },
    computed(api) {
        const t = api.readJson('./translation2.json');
    }
};
"#;
        std::fs::write(mod_a_dir.join("mod.js"), mod_a_js).unwrap();
        let mod_a_trans = serde_json::json!({
            "001901": {
                "title": "Mod A Title Overwrite"
            }
        });
        std::fs::write(
            mod_a_dir.join("translation2.json"),
            serde_json::to_string(&mod_a_trans).unwrap(),
        )
        .unwrap();

        // 2. Single-file mod "ModB.js" (Lower Priority)
        let mod_b_js = r#"
module.exports = {
    title: "Mod B Low Priority",
    author: "Author B",
    version: 2.0,
    edit: true,
    computed(api) {
        const t = api.readJson('./b_trans.json');
    }
};
"#;
        std::fs::write(mods_dir.join("ModB.js"), mod_b_js).unwrap();
        let mod_b_trans = serde_json::json!({
            "001901": {
                "title": "Mod B Title (Should be overwritten)",
                "content": "Mod B Content (Should be preserved)"
            }
        });
        std::fs::write(
            mods_dir.join("b_trans.json"),
            serde_json::to_string(&mod_b_trans).unwrap(),
        )
        .unwrap();

        // Enabled order: ModA is first (priority 0), ModB is second (priority 1)
        let enabled = vec!["ModA".to_string(), "ModB".to_string()];
        let mut registry = ModRegistry::load_from_dir(temp_dir.path(), &enabled);

        let available = registry.get_available_mods();
        assert_eq!(available.len(), 2);
        assert!(available.iter().any(|m| m.id == "ModA" && m.has_trees));
        assert!(available.iter().any(|m| m.id == "ModB" && !m.has_trees));

        let enabled_ids = registry.get_enabled_mod_ids();
        assert_eq!(enabled_ids, vec!["ModA".to_string(), "ModB".to_string()]);

        // Verify patch priority: Mod A's title wins, Mod B's content is preserved
        let patches = registry.load_active_patches();
        let p1901 = patches.get("001901").expect("Page 001901 must be patched");
        assert_eq!(p1901.title.as_deref(), Some("Mod A Title Overwrite"));
        assert_eq!(
            p1901.content.as_deref(),
            Some("Mod B Content (Should be preserved)")
        );

        // Reverse priority order: ModB first, ModA second
        registry.rescan(&["ModB".to_string(), "ModA".to_string()]);
        let patches_rev = registry.load_active_patches();
        let p1901_rev = patches_rev.get("001901").unwrap();
        assert_eq!(
            p1901_rev.title.as_deref(),
            Some("Mod B Title (Should be overwritten)")
        );
        assert_eq!(
            p1901_rev.content.as_deref(),
            Some("Mod B Content (Should be preserved)")
        );

        // Toggle ModB off
        let _ = registry.toggle_mod("ModB", false);
        assert_eq!(registry.get_enabled_mod_ids(), vec!["ModA".to_string()]);
        let patches_a_only = registry.load_active_patches();
        let p1901_a = patches_a_only.get("001901").unwrap();
        assert_eq!(p1901_a.title.as_deref(), Some("Mod A Title Overwrite"));
        assert!(p1901_a.content.is_none());
    }
}
