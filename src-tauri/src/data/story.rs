use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryPage {
    #[serde(alias = "page_id")]
    pub page_id: String,
    pub title: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub media: Vec<String>,
    #[serde(default)]
    pub flag: Vec<String>,
    #[serde(default)]
    pub next: Vec<String>,
    #[serde(default)]
    pub previous: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default, alias = "scratch_banner")]
    pub scratch_banner: Option<String>,
    #[serde(default, alias = "scratch_tooltip")]
    pub scratch_tooltip: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryPageSummary {
    #[serde(rename = "pageId")]
    pub page_id: String,
    pub title: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub media: Vec<String>,
    #[serde(default)]
    pub flag: Vec<String>,
    #[serde(default)]
    pub timestamp: String,
}

fn default_theme() -> String {
    "default".to_string()
}

impl From<&StoryPage> for StoryPageSummary {
    fn from(p: &StoryPage) -> Self {
        Self {
            page_id: p.page_id.clone(),
            title: p.title.clone(),
            theme: p.theme.clone().unwrap_or_else(|| "default".to_string()),
            media: p.media.clone(),
            flag: p.flag.clone(),
            timestamp: p.timestamp.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StoryPagePatch {
    pub title: Option<String>,
    pub content: Option<String>,
    pub media: Option<Vec<String>>,
    pub flag: Option<Vec<String>>,
    pub next: Option<Vec<String>>,
    pub previous: Option<String>,
    pub theme: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl StoryPage {
    pub fn apply_patch(&mut self, patch: &StoryPagePatch) {
        if let Some(ref title) = patch.title {
            self.title = title.clone();
        }
        if let Some(ref content) = patch.content {
            self.content = content.clone();
        }
        if let Some(ref media) = patch.media {
            self.media = media.clone();
        }
        if let Some(ref flag) = patch.flag {
            self.flag = flag.clone();
        }
        if let Some(ref next) = patch.next {
            self.next = next.clone();
        }
        if patch.previous.is_some() {
            self.previous = patch.previous.clone();
        }
        if patch.theme.is_some() {
            self.theme = patch.theme.clone();
        }
        for (k, v) in &patch.extra {
            self.extra.insert(k.clone(), v.clone());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryArchive {
    pub story: HashMap<String, StoryPage>,
    #[serde(default)]
    pub ryanquest: HashMap<String, StoryPage>,
    #[serde(default, alias = "ps_extras")]
    pub ps_extras: HashMap<String, StoryPage>,
    #[serde(default)]
    pub wv: HashMap<String, StoryPage>,
    #[serde(default)]
    pub faqs: HashMap<String, StoryPage>,
    #[serde(default)]
    pub damara: Vec<String>,
    #[serde(default, alias = "oil_retcon")]
    pub oil_retcon: String,
}

pub struct StoryIndex {
    pub order: Vec<String>,
    pub position_map: HashMap<String, usize>,
}

impl StoryIndex {
    pub fn build(story: &HashMap<String, StoryPage>) -> Self {
        let mut numeric_keys: Vec<(u32, String)> = Vec::new();
        let mut special_keys: Vec<String> = Vec::new();

        for key in story.keys() {
            if let Ok(num) = key.parse::<u32>() {
                numeric_keys.push((num, key.clone()));
            } else {
                special_keys.push(key.clone());
            }
        }
        numeric_keys.sort_by_key(|(n, _)| *n);

        let mut order: Vec<String> = numeric_keys.into_iter().map(|(_, k)| k).collect();
        special_keys.sort();
        order.extend(special_keys);

        let mut position_map = HashMap::with_capacity(order.len());
        for (idx, key) in order.iter().enumerate() {
            position_map.insert(key.clone(), idx);
        }

        Self {
            order,
            position_map,
        }
    }
}

pub struct StoryStore {
    base: Arc<StoryArchive>,
    active: Arc<RwLock<StoryArchive>>,
    index: Arc<StoryIndex>,
}

impl StoryStore {
    pub fn new(archive: StoryArchive) -> Self {
        let index = Arc::new(StoryIndex::build(&archive.story));
        let base = Arc::new(archive.clone());
        let active = Arc::new(RwLock::new(archive));
        Self {
            base,
            active,
            index,
        }
    }

    pub async fn get_page(&self, page_id: &str) -> Option<StoryPage> {
        let active_guard = self.active.read().await;
        // 1. Direct key lookup
        if let Some(page) = active_guard.story.get(page_id) {
            return Some(page.clone());
        }

        // 2. Numeric normalization to 6-digit zero-padding
        if let Ok(num) = page_id.parse::<u32>() {
            let padded = format!("{:06}", num);
            if let Some(page) = active_guard.story.get(&padded) {
                return Some(page.clone());
            }
        }

        None
    }

    pub async fn get_range(&self, start: usize, count: usize) -> Vec<StoryPageSummary> {
        let active_guard = self.active.read().await;
        let total = self.index.order.len();
        if total == 0 {
            return Vec::new();
        }

        // If start looks like a page number (e.g. 1901), resolve its position in index
        let resolved_start =
            if let Some(&pos) = self.index.position_map.get(&format!("{:06}", start)) {
                pos
            } else {
                start
            };

        if resolved_start >= total {
            return Vec::new();
        }

        let end = (resolved_start + count).min(total);
        self.index.order[resolved_start..end]
            .iter()
            .filter_map(|key| active_guard.story.get(key).map(StoryPageSummary::from))
            .collect()
    }

    pub async fn apply_patches(&self, patches: &HashMap<String, StoryPagePatch>) {
        let mut active_guard = self.active.write().await;
        *active_guard = (*self.base).clone();
        for (page_id, patch) in patches {
            if let Some(page) = active_guard.story.get_mut(page_id) {
                page.apply_patch(patch);
            }
        }
    }

    pub async fn reset_to_base(&self) {
        let mut active_guard = self.active.write().await;
        *active_guard = (*self.base).clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_real_mspa_json() {
        let path =
            std::path::Path::new("/home/criper/Downloads/Asset Pack V2/archive/data/mspa.json");
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap();
            let res = serde_json::from_str::<StoryArchive>(&content);
            if let Err(e) = res {
                panic!("Deserialization error on mspa.json: {}", e);
            }
        }
    }

    fn create_test_archive() -> StoryArchive {
        let mut story = HashMap::new();
        story.insert(
            "001901".to_string(),
            StoryPage {
                page_id: "001901".to_string(),
                title: "Homestuck".to_string(),
                timestamp: "1239634200".to_string(),
                content: "A young man stands in his bedroom.".to_string(),
                media: vec!["storyfiles/hs2/00001.gif".to_string()],
                flag: vec![],
                next: vec!["001902".to_string()],
                previous: None,
                theme: Some("default".to_string()),
                scratch_banner: None,
                scratch_tooltip: None,
                extra: Default::default(),
            },
        );
        story.insert(
            "001902".to_string(),
            StoryPage {
                page_id: "001902".to_string(),
                title: "Enter name.".to_string(),
                timestamp: "1239634260".to_string(),
                content: "Your name is JOHN.".to_string(),
                media: vec!["storyfiles/hs2/00002.gif".to_string()],
                flag: vec![],
                next: vec!["001903".to_string()],
                previous: Some("001901".to_string()),
                theme: Some("default".to_string()),
                scratch_banner: None,
                scratch_tooltip: None,
                extra: Default::default(),
            },
        );
        story.insert(
            "pony".to_string(),
            StoryPage {
                page_id: "pony".to_string(),
                title: "PONY".to_string(),
                timestamp: "1240000000".to_string(),
                content: "Neigh!".to_string(),
                media: vec![],
                flag: vec![],
                next: vec![],
                previous: None,
                theme: None,
                scratch_banner: None,
                scratch_tooltip: None,
                extra: Default::default(),
            },
        );

        StoryArchive {
            story,
            ryanquest: Default::default(),
            ps_extras: Default::default(),
            wv: Default::default(),
            faqs: Default::default(),
            damara: Default::default(),
            oil_retcon: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_story_store_lookups() {
        let store = StoryStore::new(create_test_archive());

        // Exact match
        let p1 = store.get_page("001901").await.unwrap();
        assert_eq!(p1.title, "Homestuck");

        // Numeric without padding fallback
        let p1_num = store.get_page("1901").await.unwrap();
        assert_eq!(p1_num.page_id, "001901");

        // Special alphanumeric key
        let pony = store.get_page("pony").await.unwrap();
        assert_eq!(pony.title, "PONY");

        // Non-existent
        assert!(store.get_page("999999").await.is_none());
    }

    #[tokio::test]
    async fn test_story_store_range() {
        let store = StoryStore::new(create_test_archive());
        let range = store.get_range(1901, 2).await;
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].page_id, "001901");
        assert_eq!(range[1].page_id, "001902");
    }

    #[tokio::test]
    async fn test_story_store_patches() {
        let store = StoryStore::new(create_test_archive());

        let mut patches = HashMap::new();
        patches.insert(
            "001901".to_string(),
            StoryPagePatch {
                title: Some("Homestuck PT-BR".to_string()),
                content: Some("Um jovem rapaz está em seu quarto.".to_string()),
                media: None,
                flag: None,
                next: None,
                previous: None,
                theme: None,
                extra: Default::default(),
            },
        );

        store.apply_patches(&patches).await;
        let p_patched = store.get_page("001901").await.unwrap();
        assert_eq!(p_patched.title, "Homestuck PT-BR");
        assert_eq!(p_patched.content, "Um jovem rapaz está em seu quarto.");

        // Reset restores original
        store.reset_to_base().await;
        let p_base = store.get_page("001901").await.unwrap();
        assert_eq!(p_base.title, "Homestuck");
    }
}
