use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistCredit {
    pub who: String,
    #[serde(default)]
    pub what: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributorCredit {
    pub who: String,
    #[serde(default)]
    pub what: Option<String>,
}

pub fn deserialize_urls<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum UrlsHelper {
        List(Vec<String>),
        #[allow(dead_code)]
        Bool(bool),
    }

    match UrlsHelper::deserialize(deserializer)? {
        UrlsHelper::List(v) => Ok(v),
        UrlsHelper::Bool(_) => Ok(Vec::new()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub name: String,
    pub directory: String,
    pub date: Option<String>,
    pub tracks: Vec<String>,
    #[serde(default)]
    pub album_cover_artists: Vec<ArtistCredit>,
    #[serde(default)]
    pub artists: Option<Vec<ArtistCredit>>,
    #[serde(default)]
    pub bonus: Vec<String>,
    #[serde(default)]
    pub commentary: Option<String>,
    #[serde(default)]
    pub has_track_art: bool,
    #[serde(default, deserialize_with = "deserialize_urls")]
    pub urls: Vec<String>,
    #[serde(default)]
    pub uses_groups: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub name: String,
    pub directory: String,
    pub album: Vec<String>,
    pub artists: Vec<ArtistCredit>,
    pub duration: u32,
    pub group: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub bandcamp_id: Option<u64>,
    #[serde(default)]
    pub commentary: Option<String>,
    #[serde(default)]
    pub contributors: Vec<ContributorCredit>,
    #[serde(default)]
    pub cover_artists: Option<Vec<ArtistCredit>>,
    #[serde(default)]
    pub lyrics: Option<String>,
    #[serde(default)]
    pub pages: Vec<String>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub referenced_by: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_urls")]
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub name: String,
    pub directory: String,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default, deserialize_with = "deserialize_urls")]
    pub urls: Vec<String>,
    #[serde(default)]
    pub credits: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashSync {
    pub date: String,
    pub tracks: Vec<String>,
    #[serde(default)]
    pub bolin: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicArchive {
    pub albums: HashMap<String, Album>,
    pub tracks: HashMap<String, Track>,
    pub artists: HashMap<String, Artist>,
    pub flashes: HashMap<String, FlashSync>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct TestItem {
        #[serde(deserialize_with = "deserialize_urls")]
        urls: Vec<String>,
    }

    #[test]
    fn test_deserialize_urls_with_bool() {
        let json_false = r#"{"urls": false}"#;
        let item: TestItem = serde_json::from_str(json_false).unwrap();
        assert!(item.urls.is_empty());
    }

    #[test]
    fn test_deserialize_urls_with_list() {
        let json_list = r#"{"urls": ["https://youtube.com/watch?v=123"]}"#;
        let item: TestItem = serde_json::from_str(json_list).unwrap();
        assert_eq!(item.urls.len(), 1);
        assert_eq!(item.urls[0], "https://youtube.com/watch?v=123");
    }
}
