use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: u32,
    pub target_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            target_extensions: vec![
                ".ts".to_string(),
                ".js".to_string(),
                ".tsx".to_string(),
                ".jsx".to_string(),
            ],
            exclude_patterns: vec![
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "*.test.ts".to_string(),
                "*.spec.ts".to_string(),
            ],
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
