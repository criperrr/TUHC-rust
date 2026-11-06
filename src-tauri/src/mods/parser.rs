use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VueHookDescriptor {
    pub match_name: String,
    pub computed_keys: Vec<String>,
    pub log_button_open_prefix: Option<String>,
    pub log_button_close_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedModInfo {
    pub title: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: f64,
    pub edit: bool,
    pub translation_files: Vec<String>,
    pub trees: HashMap<String, String>,
    pub routes: HashMap<String, String>,
    pub vue_hooks: Vec<VueHookDescriptor>,
}

pub fn parse_mod_js(path: &Path) -> Option<ParsedModInfo> {
    let content = std::fs::read_to_string(path).ok()?;

    let title_re = Regex::new(r#"title\s*:\s*["']([^"']+)["']"#).ok()?;
    let summary_re = Regex::new(r#"summary\s*:\s*["']([^"']+)["']"#).ok()?;
    let desc_re = Regex::new(r#"description\s*:\s*["']([^"']+)["']"#).ok()?;
    let author_re = Regex::new(r#"author\s*:\s*["']([^"']+)["']"#).ok()?;
    let version_re = Regex::new(r#"(?:modVersion|version)\s*:\s*([0-9.]+)"#).ok()?;

    let title = title_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| {
            if path.is_file() && path.file_name().map(|f| f == "mod.js").unwrap_or(false) {
                path.parent()
                    .and_then(|p| p.file_name())
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown Mod".to_string())
            } else {
                path.file_stem()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown Mod".to_string())
            }
        });

    let summary = summary_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let description = desc_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let author = author_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let version = version_re
        .captures(&content)
        .and_then(|c| c.get(1).and_then(|m| m.as_str().parse::<f64>().ok()))
        .unwrap_or(1.0);

    let mut translation_files = Vec::new();
    if let Ok(read_json_re) = Regex::new(r#"api\.readJson(?:Async)?\s*\(\s*["']([^"']+)["']\s*\)"#)
    {
        for cap in read_json_re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                let raw_path = m.as_str().trim();
                let clean = raw_path
                    .trim_start_matches('.')
                    .trim_start_matches('/')
                    .to_string();
                if !clean.contains("..") && !clean.is_empty() && !translation_files.contains(&clean)
                {
                    translation_files.push(clean);
                }
            }
        }
    }

    if translation_files.is_empty() {
        if let Some(parent) = path.parent() {
            if parent.join("translation2.json").is_file() {
                translation_files.push("translation2.json".to_string());
            }
        }
    }

    let edit = content.contains("edit: true")
        || content.contains("edit:true")
        || content.contains("edit(archive)")
        || !translation_files.is_empty();

    let mut trees = HashMap::new();
    let trees_block_re = Regex::new(r#"trees\s*:\s*\{([^}]+)\}"#).ok()?;
    let pair_re = Regex::new(r#"["']([^"']+)["']\s*:\s*["']([^"']+)["']"#).ok()?;

    if let Some(caps) = trees_block_re.captures(&content) {
        if let Some(block) = caps.get(1) {
            for pair in pair_re.captures_iter(block.as_str()) {
                if let (Some(k), Some(v)) = (pair.get(1), pair.get(2)) {
                    trees.insert(k.as_str().to_string(), v.as_str().to_string());
                }
            }
        }
    }

    let mut routes = HashMap::new();
    if let Ok(routes_block_re) = Regex::new(r#"routes\s*:\s*\{([^}]+)\}"#) {
        if let Some(caps) = routes_block_re.captures(&content) {
            if let Some(block) = caps.get(1) {
                for pair in pair_re.captures_iter(block.as_str()) {
                    if let (Some(k), Some(v)) = (pair.get(1), pair.get(2)) {
                        routes.insert(k.as_str().to_string(), v.as_str().to_string());
                    }
                }
            }
        }
    }

    let mut vue_hooks = Vec::new();
    if content.contains("vueHooks") {
        let match_name_re = Regex::new(r#"matchName\s*:\s*["']([^"']+)["']"#).ok();
        let match_name = match_name_re
            .and_then(|r| r.captures(&content))
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
            .unwrap_or_else(|| "pageText".to_string());

        let mut computed_keys = Vec::new();
        let mut log_button_open_prefix = None;
        let mut log_button_close_prefix = None;

        if content.contains("logButtonText") {
            computed_keys.push("logButtonText".to_string());
            let ternary_re =
                Regex::new(r#"logHidden\s*\)?\s*\?\s*["']([^"']+)["']\s*:\s*["']([^"']+)["']"#)
                    .ok();
            if let Some(re) = ternary_re {
                if let Some(cap) = re.captures(&content) {
                    log_button_open_prefix = cap.get(1).map(|m| m.as_str().to_string());
                    log_button_close_prefix = cap.get(2).map(|m| m.as_str().to_string());
                }
            }
            if log_button_open_prefix.is_none()
                && content.contains("'Abrir '")
                && content.contains("'Fechar '")
            {
                log_button_open_prefix = Some("Abrir ".to_string());
                log_button_close_prefix = Some("Fechar ".to_string());
            }
        }

        vue_hooks.push(VueHookDescriptor {
            match_name,
            computed_keys,
            log_button_open_prefix,
            log_button_close_prefix,
            raw_body: None,
        });
    }

    Some(ParsedModInfo {
        title,
        summary,
        description,
        author,
        version,
        edit,
        translation_files,
        trees,
        routes,
        vue_hooks,
    })
}

pub fn evaluate_log_button_text(
    raw_content: &str,
    log_hidden: bool,
    vue_hooks_active: bool,
) -> String {
    let re = match Regex::new(r#"\|(.*?)\|"#) {
        Ok(r) => r,
        Err(_) => {
            return if log_hidden {
                "Show pesterlog".to_string()
            } else {
                "Hide pesterlog".to_string()
            }
        }
    };

    let text = if let Some(caps) = re.captures(raw_content) {
        caps.get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "pesterlog".to_string())
    } else {
        "pesterlog".to_string()
    };

    let text_upper = text.to_uppercase();
    if text_upper.contains("P4SSWORD H1NT") {
        return text;
    }

    let normalized_text = if text_upper.contains("TRKSTRLOG") {
        "tricksterlog".to_string()
    } else if text_upper.contains("SRIOUSBIZ") {
        "serious business".to_string()
    } else {
        text.to_lowercase()
    };

    let prefix = if vue_hooks_active {
        if log_hidden {
            "Abrir "
        } else {
            "Fechar "
        }
    } else if log_hidden {
        "Show "
    } else {
        "Hide "
    };

    format!("{}{}", prefix, normalized_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_legacy_mod_js() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let content = r#"
module.exports = {
  title: "Homestuck PT-BR",
  summary: "Uma história sobre um garoto...",
  author: "Homestuck BR Team",
  modVersion: 2.0,
  edit: true,
  trees: {
    "./Assets/": "assets://"
  },
  vueHooks: [{
    matchName: "pageText",
    computed: {
      logButtonText() {
        let text = this.content.match(/\|(.*?)\|/)[1]
        const state = (this.logHidden) ? 'Abrir ' : 'Fechar '
        if (/P4SSWORD H1NT/i.test(text)){
            return text
        }
        else if (/trkstrlog/i.test(text)){
            text = 'tricksterlog'
        }
        else if (/sriousbiz/i.test(text)){
            text = 'serious business'
        }
        return state + text.toLowerCase()
      }
    }
  }],
  computed(api) {
    const translation = api.readJson('./translation2.json')
    api.logger.info(translation)
  }
}
        "#;
        std::fs::write(temp.path(), content).unwrap();

        let parsed = parse_mod_js(temp.path()).unwrap();
        assert_eq!(parsed.title, "Homestuck PT-BR");
        assert_eq!(parsed.author.as_deref(), Some("Homestuck BR Team"));
        assert_eq!(parsed.version, 2.0);
        assert!(parsed.edit);
        assert_eq!(
            parsed.trees.get("./Assets/").map(|s| s.as_str()),
            Some("assets://")
        );
        assert_eq!(parsed.translation_files, vec!["translation2.json"]);
        assert_eq!(parsed.vue_hooks.len(), 1);
        let hook = &parsed.vue_hooks[0];
        assert_eq!(hook.match_name, "pageText");
        assert_eq!(hook.log_button_open_prefix.as_deref(), Some("Abrir "));
        assert_eq!(hook.log_button_close_prefix.as_deref(), Some("Fechar "));
    }

    #[test]
    fn test_evaluate_log_button_text_table() {
        // EN / inactive
        assert_eq!(
            evaluate_log_button_text("|PESTERLOG|\nHello", true, false),
            "Show pesterlog"
        );
        assert_eq!(
            evaluate_log_button_text("|PESTERLOG|\nHello", false, false),
            "Hide pesterlog"
        );
        assert_eq!(
            evaluate_log_button_text("|TRKSTRLOG|\nHello", true, false),
            "Show tricksterlog"
        );
        assert_eq!(
            evaluate_log_button_text("|SRIOUSBIZ|\nHello", true, false),
            "Show serious business"
        );
        assert_eq!(
            evaluate_log_button_text("|P4SSWORD H1NT|\nSecret", true, false),
            "P4SSWORD H1NT"
        );
        assert_eq!(
            evaluate_log_button_text("|P4SSWORD H1NT|\nSecret", false, false),
            "P4SSWORD H1NT"
        );

        // PT-BR / active
        assert_eq!(
            evaluate_log_button_text("|PESTERLOG|\nOlá", true, true),
            "Abrir pesterlog"
        );
        assert_eq!(
            evaluate_log_button_text("|PESTERLOG|\nOlá", false, true),
            "Fechar pesterlog"
        );
        assert_eq!(
            evaluate_log_button_text("|SPRITELOG|\nOlá", true, true),
            "Abrir spritelog"
        );
        assert_eq!(
            evaluate_log_button_text("|TRKSTRLOG|\nOlá", true, true),
            "Abrir tricksterlog"
        );
        assert_eq!(
            evaluate_log_button_text("|SRIOUSBIZ|\nOlá", true, true),
            "Abrir serious business"
        );
        assert_eq!(
            evaluate_log_button_text("|P4SSWORD H1NT|\nSegredo", true, true),
            "P4SSWORD H1NT"
        );
    }
}
