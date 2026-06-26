use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LanguageConfig {
    pub extensions: Vec<String>,
    pub import_pattern: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Config {
    pub version: u32,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub languages: HashMap<String, LanguageConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut languages = HashMap::new();
        languages.insert(
            "typescript".to_string(),
            LanguageConfig {
                extensions: vec![".ts".to_string(), ".tsx".to_string()],
                import_pattern: r#"from\s+['"]([^'"]+)['"]"#.to_string(),
            },
        );
        languages.insert(
            "javascript".to_string(),
            LanguageConfig {
                extensions: vec![".js".to_string(), ".jsx".to_string()],
                import_pattern: r#"from\s+['"]([^'"]+)['"]"#.to_string(),
            },
        );
        languages.insert(
            "rust".to_string(),
            LanguageConfig {
                extensions: vec![".rs".to_string()],
                import_pattern: r#"use\s+([\w:]+)"#.to_string(),
            },
        );
        languages.insert(
            "python".to_string(),
            LanguageConfig {
                extensions: vec![".py".to_string()],
                import_pattern: r#"^(?:from|import)\s+([\w\.]+)"#.to_string(),
            },
        );

        Self {
            version: 1,
            languages,
            target_extensions: Vec::new(),
            exclude_patterns: vec![
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn get_import_pattern(&self, ext: &str) -> Option<&str> {
        for lang_config in self.languages.values() {
            if lang_config.extensions.iter().any(|e| e == ext) {
                return Some(&lang_config.import_pattern);
            }
        }
        if self.target_extensions.iter().any(|e| e == ext) {
            // Old format extensions are treated as typescript/javascript
            return Some(r#"from\s+['"]([^'"]+)['"]"#);
        }
        None
    }

    pub fn find_matched_extension(&self, rel_path: &str) -> Option<String> {
        for lang_config in self.languages.values() {
            for ext in &lang_config.extensions {
                if rel_path.ends_with(ext) {
                    return Some(ext.clone());
                }
            }
        }
        for ext in &self.target_extensions {
            if rel_path.ends_with(ext) {
                return Some(ext.clone());
            }
        }
        None
    }

    pub fn upgrade(&mut self) {
        if !self.target_extensions.is_empty() {
            let mut ts_exts = Vec::new();
            let mut js_exts = Vec::new();
            
            for ext in &self.target_extensions {
                if ext == ".ts" || ext == ".tsx" {
                    ts_exts.push(ext.clone());
                } else if ext == ".js" || ext == ".jsx" {
                    js_exts.push(ext.clone());
                } else {
                    js_exts.push(ext.clone());
                }
            }

            if !ts_exts.is_empty() {
                self.languages.insert(
                    "typescript".to_string(),
                    LanguageConfig {
                        extensions: ts_exts,
                        import_pattern: r#"from\s+['"]([^'"]+)['"]"#.to_string(),
                    },
                );
            }
            if !js_exts.is_empty() {
                self.languages.insert(
                    "javascript".to_string(),
                    LanguageConfig {
                        extensions: js_exts,
                        import_pattern: r#"from\s+['"]([^'"]+)['"]"#.to_string(),
                    },
                );
            }

            self.target_extensions.clear();
        }
    }
}

pub fn find_project_root() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut dir = current_dir.as_path();
    loop {
        let codewatch_dir = dir.join(".codewatch");
        if codewatch_dir.is_dir() {
            return Some(dir.to_path_buf());
        }
        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }
    None
}

pub fn get_config_path(project_root: &Path) -> PathBuf {
    project_root.join(".codewatch").join("config.yml")
}

pub fn load_config(project_root: &Path) -> Result<Config> {
    let path = get_config_path(project_root);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;
    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file at {:?}", path))?;
    Ok(config)
}

pub fn save_config(project_root: &Path, config: &Config) -> Result<()> {
    let path = get_config_path(project_root);
    let content = serde_yaml::to_string(config)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write config file to {:?}", path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing_new_format() {
        let yaml = r#"
version: 1
languages:
  typescript:
    extensions: [.ts, .tsx]
    import_pattern: 'from\s+[''"]([^''"]+)[''"]'
  rust:
    extensions: [.rs]
    import_pattern: 'use\s+([\w:]+)'
exclude_patterns:
  - node_modules
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.languages.contains_key("typescript"));
        assert!(config.languages.contains_key("rust"));
        assert_eq!(config.get_import_pattern(".ts"), Some(r#"from\s+['"]([^'"]+)['"]"#));
        assert_eq!(config.get_import_pattern(".rs"), Some(r#"use\s+([\w:]+)"#));
        assert_eq!(config.get_import_pattern(".py"), None);
    }

    #[test]
    fn test_config_parsing_old_format() {
        let yaml = r#"
version: 1
target_extensions:
  - .ts
  - .tsx
exclude_patterns:
  - node_modules
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.languages.is_empty());
        assert_eq!(config.target_extensions, vec![".ts".to_string(), ".tsx".to_string()]);
        assert_eq!(config.get_import_pattern(".ts"), Some(r#"from\s+['"]([^'"]+)['"]"#));
        assert_eq!(config.get_import_pattern(".rs"), None);
    }

    #[test]
    fn test_config_upgrade() {
        let mut config = Config {
            version: 1,
            languages: HashMap::new(),
            target_extensions: vec![
                ".ts".to_string(),
                ".tsx".to_string(),
                ".js".to_string(),
                ".jsx".to_string(),
                ".custom".to_string(),
            ],
            exclude_patterns: vec!["node_modules".to_string()],
        };

        config.upgrade();

        assert!(config.target_extensions.is_empty());
        assert_eq!(config.languages.len(), 2);
        
        let ts_config = config.languages.get("typescript").unwrap();
        assert_eq!(ts_config.extensions, vec![".ts".to_string(), ".tsx".to_string()]);
        assert_eq!(ts_config.import_pattern, r#"from\s+['"]([^'"]+)['"]"#);

        let js_config = config.languages.get("javascript").unwrap();
        assert_eq!(js_config.extensions, vec![".js".to_string(), ".jsx".to_string(), ".custom".to_string()]);
        assert_eq!(js_config.import_pattern, r#"from\s+['"]([^'"]+)['"]"#);
    }
}

