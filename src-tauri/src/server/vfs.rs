use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Descriptor of an active mod's assets and tree mappings
#[derive(Clone, Debug)]
pub struct ActiveModDescriptor {
    pub id: String,
    pub root_dir: PathBuf,
    pub trees: HashMap<String, String>, // e.g. "./Assets/" -> "assets://"
    pub routes: HashMap<String, String>, // e.g. "assets://file.gif" -> "./file.gif"
}

#[derive(Clone)]
pub struct Vfs {
    base_dir: PathBuf,
    overlay_map: Arc<RwLock<HashMap<String, PathBuf>>>,
    active_mods: Arc<RwLock<Vec<ActiveModDescriptor>>>,
}

impl Vfs {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            overlay_map: Arc::new(RwLock::new(HashMap::new())),
            active_mods: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Normalizes, percent-decodes, and sanitizes incoming URI paths
    pub fn sanitize_path(raw_path: &str) -> Option<String> {
        // Quick null byte rejection
        if raw_path.contains('\0') {
            return None;
        }

        // 1. URL percent-decoding
        let decoded = percent_encoding::percent_decode_str(raw_path)
            .decode_utf8()
            .ok()?;

        if decoded.contains('\0') {
            return None;
        }

        // 2. Strip query string and fragment
        let clean = decoded.split('?').next().unwrap_or(&decoded);
        let clean = clean.split('#').next().unwrap_or(clean);

        // 3. Strip scheme and prefix slashes
        let clean = clean.strip_prefix("assets://").unwrap_or(clean);
        let clean = clean.strip_prefix('/').unwrap_or(clean);
        let clean = clean.strip_prefix("assets/").unwrap_or(clean);
        let clean = clean.trim_start_matches('/');

        // Reject if path starts with slash or was effectively empty
        if clean.is_empty() {
            return None;
        }

        // 4. Legacy Sfiles quirk normalization (resources.js:146)
        let clean = clean.replace("storyfiles/hs2/Sfiles/", "storyfiles/hs2/");

        // 5. Traversal defense: reject any ".." path segments
        for seg in clean.split('/') {
            if seg == ".." || seg.is_empty() && clean.contains("//") {
                return None;
            }
        }

        Some(clean)
    }

    /// Resolves a requested URI path to an absolute disk path in O(1)
    pub async fn resolve(&self, raw_path: &str) -> Option<PathBuf> {
        let clean = Self::sanitize_path(raw_path)?;

        // 1. Check mod overlay map (O(1) in-memory lookup)
        {
            let map = self.overlay_map.read().await;
            if let Some(target) = map.get(&clean) {
                if target.is_file() {
                    return Some(target.clone());
                }
            }
        }

        // 2. Fallback to base Asset Pack directory
        let candidate = self.base_dir.join(&clean);
        if candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    }

    /// Rebuilds the in-memory overlay map from active mods in <2ms
    pub async fn rebuild_overlay(&self, mods: Vec<ActiveModDescriptor>) -> std::io::Result<usize> {
        let mut new_overlay = HashMap::new();

        // Process mods in reverse order so earlier mods take priority
        for m in mods.iter().rev() {
            // Process trees (e.g. "./Assets/" -> "assets://")
            for (mod_tree_rel, asset_tree) in &m.trees {
                let mod_tree_dir = m.root_dir.join(mod_tree_rel.trim_start_matches("./"));
                if !mod_tree_dir.is_dir() {
                    continue;
                }

                let target_prefix = asset_tree
                    .strip_prefix("assets://")
                    .unwrap_or(asset_tree)
                    .trim_matches('/');

                for entry in walkdir::WalkDir::new(&mod_tree_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        if let Ok(rel) = entry.path().strip_prefix(&mod_tree_dir) {
                            let rel_str = rel.to_string_lossy().replace('\\', "/");
                            let asset_key = if target_prefix.is_empty() {
                                rel_str
                            } else {
                                format!("{}/{}", target_prefix, rel_str)
                            };
                            new_overlay.insert(asset_key, entry.path().to_path_buf());
                        }
                    }
                }
            }

            // Process explicit routes (e.g. "assets://file.gif" -> "./file.gif")
            for (asset_uri, mod_file_rel) in &m.routes {
                if let Some(clean_key) = Self::sanitize_path(asset_uri) {
                    let target_path = m.root_dir.join(mod_file_rel.trim_start_matches("./"));
                    if target_path.is_file() {
                        new_overlay.insert(clean_key, target_path);
                    }
                }
            }
        }

        let count = new_overlay.len();
        {
            let mut map = self.overlay_map.write().await;
            *map = new_overlay;
        }
        {
            let mut active = self.active_mods.write().await;
            *active = mods;
        }

        Ok(count)
    }

    pub async fn overlay_count(&self) -> usize {
        self.overlay_map.read().await.len()
    }

    pub async fn get_active_mod_ids(&self) -> Vec<String> {
        self.active_mods
            .read()
            .await
            .iter()
            .map(|m| m.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_valid() {
        assert_eq!(
            Vfs::sanitize_path("/assets/advimgs/bq/bq001.gif"),
            Some("advimgs/bq/bq001.gif".to_string())
        );
        assert_eq!(
            Vfs::sanitize_path("assets://storyfiles/hs2/00001.gif"),
            Some("storyfiles/hs2/00001.gif".to_string())
        );
        assert_eq!(
            Vfs::sanitize_path("/archive/tbiy/The%20Baby%20Is%20You%20-%20Complete.mp3"),
            Some("archive/tbiy/The Baby Is You - Complete.mp3".to_string())
        );
        assert_eq!(
            Vfs::sanitize_path("/storyfiles/hs2/Sfiles/06379/06379.swf"),
            Some("storyfiles/hs2/06379/06379.swf".to_string())
        );
    }

    #[test]
    fn test_sanitize_path_security() {
        assert_eq!(Vfs::sanitize_path("/assets/../../etc/passwd"), None);
        assert_eq!(Vfs::sanitize_path("/assets/%2e%2e%2fetc/passwd"), None);
        assert_eq!(
            Vfs::sanitize_path("/assets/advimgs/bq/bq001.gif\0.txt"),
            None
        );
        assert_eq!(Vfs::sanitize_path("/assets/"), None);
        assert_eq!(Vfs::sanitize_path(""), None);
    }

    #[tokio::test]
    async fn test_vfs_overlay_resolution() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_dir = temp_dir.path().join("base");
        let mod_dir = temp_dir.path().join("mods").join("test_mod");
        std::fs::create_dir_all(base_dir.join("storyfiles/hs2")).unwrap();
        std::fs::create_dir_all(mod_dir.join("Assets/storyfiles/hs2")).unwrap();

        let base_file = base_dir.join("storyfiles/hs2/00001.gif");
        let base_file2 = base_dir.join("storyfiles/hs2/00002.gif");
        let mod_file2 = mod_dir.join("Assets/storyfiles/hs2/00002.gif");

        std::fs::write(&base_file, b"base 1").unwrap();
        std::fs::write(&base_file2, b"base 2").unwrap();
        std::fs::write(&mod_file2, b"mod 2").unwrap();

        let vfs = Vfs::new(base_dir);

        // Before mod overlay: both resolve to base
        assert_eq!(
            vfs.resolve("/storyfiles/hs2/00001.gif").await,
            Some(base_file.clone())
        );
        assert_eq!(
            vfs.resolve("/storyfiles/hs2/00002.gif").await,
            Some(base_file2.clone())
        );

        // Add mod overlay
        let mut trees = HashMap::new();
        trees.insert("./Assets/".to_string(), "assets://".to_string());
        let desc = ActiveModDescriptor {
            id: "test_mod".to_string(),
            root_dir: mod_dir,
            trees,
            routes: HashMap::new(),
        };

        let count = vfs.rebuild_overlay(vec![desc]).await.unwrap();
        assert_eq!(count, 1);

        // After overlay: 00001.gif is base, 00002.gif is overridden by mod
        assert_eq!(
            vfs.resolve("/storyfiles/hs2/00001.gif").await,
            Some(base_file.clone())
        );
        assert_eq!(
            vfs.resolve("/storyfiles/hs2/00002.gif").await,
            Some(mod_file2.clone())
        );

        // Clearing overlay restores base
        vfs.rebuild_overlay(vec![]).await.unwrap();
        assert_eq!(
            vfs.resolve("/storyfiles/hs2/00002.gif").await,
            Some(base_file2)
        );
    }
}
