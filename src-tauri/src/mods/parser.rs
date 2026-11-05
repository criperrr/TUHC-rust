use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedModInfo {
    pub title: String,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub version: f64,
    pub edit: bool,
    pub trees: HashMap<String, String>,
    pub routes: HashMap<String, String>,
}

pub fn parse_mod_js(path: &Path) -> Option<ParsedModInfo> {
    let content = std::fs::read_to_string(path).ok()?;

    let title_re = Regex::new(r#"title\s*:\s*["']([^"']+)["']"#).ok()?;
    let summary_re = Regex::new(r#"summary\s*:\s*["']([^"']+)["']"#).ok()?;
    let author_re = Regex::new(r#"author\s*:\s*["']([^"']+)["']"#).ok()?;
    let version_re = Regex::new(r#"modVersion\s*:\s*([0-9.]+)"#).ok()?;

    let title = title_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown Mod".to_string())
        });

    let summary = summary_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let author = author_re
        .captures(&content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let version = version_re
        .captures(&content)
        .and_then(|c| c.get(1).and_then(|m| m.as_str().parse::<f64>().ok()))
        .unwrap_or(1.0);

    let edit = content.contains("edit: true") || content.contains("edit:true");

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
    let routes_block_re = Regex::new(r#"routes\s*:\s*\{([^}]+)\}"#).ok()?;
    if let Some(caps) = routes_block_re.captures(&content) {
        if let Some(block) = caps.get(1) {
            for pair in pair_re.captures_iter(block.as_str()) {
                if let (Some(k), Some(v)) = (pair.get(1), pair.get(2)) {
                    routes.insert(k.as_str().to_string(), v.as_str().to_string());
                }
            }
        }
    }

    Some(ParsedModInfo {
        title,
        summary,
        author,
        version,
        edit,
        trees,
        routes,
    })
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
    }
}
