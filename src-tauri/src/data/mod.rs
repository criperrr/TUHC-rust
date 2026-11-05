pub mod music;
pub mod story;

use std::fs;
use std::path::Path;
use std::sync::Arc;

pub use music::MusicArchive;
pub use story::{StoryArchive, StoryPage, StoryPagePatch, StoryPageSummary, StoryStore};

pub struct ArchiveStore {
    pub story: Arc<StoryStore>,
    pub music: Arc<MusicArchive>,
    pub music_raw: Arc<serde_json::Value>,
}

impl ArchiveStore {
    pub fn load_from_dir(asset_dir: &Path) -> Self {
        let data_dir = asset_dir.join("archive").join("data");

        // 1. Load mspa.json
        let mspa_path = data_dir.join("mspa.json");
        let story_archive: StoryArchive = if mspa_path.is_file() {
            match fs::read_to_string(&mspa_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(arc) => arc,
                    Err(e) => {
                        log::error!("Failed to parse mspa.json: {}", e);
                        StoryArchive {
                            story: Default::default(),
                            ryanquest: Default::default(),
                            ps_extras: Default::default(),
                            wv: Default::default(),
                            faqs: Default::default(),
                            damara: Default::default(),
                            oil_retcon: Default::default(),
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to read mspa.json: {}", e);
                    StoryArchive {
                        story: Default::default(),
                        ryanquest: Default::default(),
                        ps_extras: Default::default(),
                        wv: Default::default(),
                        faqs: Default::default(),
                        damara: Default::default(),
                        oil_retcon: Default::default(),
                    }
                }
            }
        } else {
            log::warn!("mspa.json not found at {:?}", mspa_path);
            StoryArchive {
                story: Default::default(),
                ryanquest: Default::default(),
                ps_extras: Default::default(),
                wv: Default::default(),
                faqs: Default::default(),
                damara: Default::default(),
                oil_retcon: Default::default(),
            }
        };

        // 2. Load music.json
        let music_path = data_dir.join("music.json");
        let (music_archive, music_raw) = if music_path.is_file() {
            match fs::read_to_string(&music_path) {
                Ok(content) => {
                    let raw: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                    let arc: MusicArchive = serde_json::from_str(&content).unwrap_or_else(|e| {
                        log::error!("Failed to parse music.json into struct: {}", e);
                        MusicArchive {
                            albums: Default::default(),
                            tracks: Default::default(),
                            artists: Default::default(),
                            flashes: Default::default(),
                        }
                    });
                    (arc, raw)
                }
                Err(e) => {
                    log::error!("Failed to read music.json: {}", e);
                    (
                        MusicArchive {
                            albums: Default::default(),
                            tracks: Default::default(),
                            artists: Default::default(),
                            flashes: Default::default(),
                        },
                        serde_json::Value::Null,
                    )
                }
            }
        } else {
            log::warn!("music.json not found at {:?}", music_path);
            (
                MusicArchive {
                    albums: Default::default(),
                    tracks: Default::default(),
                    artists: Default::default(),
                    flashes: Default::default(),
                },
                serde_json::Value::Null,
            )
        };

        Self {
            story: Arc::new(StoryStore::new(story_archive)),
            music: Arc::new(music_archive),
            music_raw: Arc::new(music_raw),
        }
    }

    pub async fn get_story_page(&self, page_id: &str) -> Option<StoryPage> {
        self.story.get_page(page_id).await
    }

    pub async fn get_story_range(&self, start: usize, count: usize) -> Vec<StoryPageSummary> {
        self.story.get_range(start, count).await
    }

    pub fn get_music_raw(&self) -> serde_json::Value {
        (*self.music_raw).clone()
    }
}
