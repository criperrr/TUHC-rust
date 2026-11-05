pub mod parser;
pub mod patcher;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::data::story::StoryPagePatch;
use crate::server::vfs::ActiveModDescriptor;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModMetadata {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: f64,
    pub description: String,
    pub enabled: bool,
    #[serde(default)]
    pub has_trees: bool,
    #[serde(default)]
    pub has_edit: bool,
}

#[derive(Clone)]
struct ModEntry {
    meta: ModMetadata,
    root_dir: PathBuf,
    trees: HashMap<String, String>,
    routes: HashMap<String, String>,
}

pub struct ModRegistry {
    mods_dir: PathBuf,
    mods: Vec<ModEntry>,
}

impl ModRegistry {
    pub fn load_from_dir(asset_dir: &Path, enabled_list: &[String]) -> Self {
        let mods_dir = asset_dir.join("mods");
        let mut registry = Self {
            mods_dir,
            mods: Vec::new(),
        };
        registry.rescan(enabled_list);
        registry
    }

    pub fn rescan(&mut self, enabled_list: &[String]) {
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
                            let enabled = enabled_list.contains(&id);
                            let has_trees = !parsed.trees.is_empty();
                            let has_edit = parsed.edit || path.join("translation2.json").is_file();
                            let desc = parsed
                                .summary
                                .clone()
                                .unwrap_or_else(|| format!("{} mod for Homestuck", parsed.title));

                            let meta = ModMetadata {
                                id: id.clone(),
                                title: parsed.title,
                                author: parsed
                                    .author
                                    .unwrap_or_else(|| "Unknown Author".to_string()),
                                version: parsed.version,
                                description: desc,
                                enabled,
                                has_trees,
                                has_edit,
                            };

                            self.mods.push(ModEntry {
                                meta,
                                root_dir: path,
                                trees: parsed.trees,
                                routes: parsed.routes,
                            });
                        }
                    }
                }
            }
        }
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
            Ok(enabled)
        } else {
            Err(format!("Unknown mod '{}'", mod_id))
        }
    }

    pub fn get_enabled_mod_ids(&self) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| m.meta.enabled)
            .map(|m| m.meta.id.clone())
            .collect()
    }

    pub fn get_active_descriptors(&self) -> Vec<ActiveModDescriptor> {
        self.mods
            .iter()
            .filter(|m| m.meta.enabled)
            .map(|m| ActiveModDescriptor {
                id: m.meta.id.clone(),
                root_dir: m.root_dir.clone(),
                trees: m.trees.clone(),
                routes: m.routes.clone(),
            })
            .collect()
    }

    pub fn load_active_patches(&self) -> HashMap<String, StoryPagePatch> {
        let mut combined = HashMap::new();
        for m in self.mods.iter().filter(|m| m.meta.enabled) {
            let translation_path = m.root_dir.join("translation2.json");
            if translation_path.is_file() {
                if let Ok(patches) = patcher::load_translation_patches(&translation_path) {
                    combined.extend(patches);
                }
            }
        }
        combined
    }
}
