use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::data::story::StoryPagePatch;

pub fn load_translation_patches(path: &Path) -> Result<HashMap<String, StoryPagePatch>, String> {
    if !path.is_file() {
        return Ok(HashMap::new());
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read translation file: {}", e))?;
    let patches: HashMap<String, StoryPagePatch> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse translation JSON: {}", e))?;

    log::info!("Loaded {} page patches from {:?}", patches.len(), path);
    Ok(patches)
}
