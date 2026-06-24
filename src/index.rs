use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileIndex {
    pub path: String, // Project relative path
    pub score: f64,
    pub git_commits: u32,
    pub file_size: u64,
    pub ref_count: u32,
    pub last_modified: DateTime<Utc>,
    pub score_updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Index {
    pub version: u32,
    pub last_scan: DateTime<Utc>,
    pub files: HashMap<String, FileIndex>, // Hash -> FileIndex
}

pub fn get_index_path(project_root: &Path) -> PathBuf {
    project_root.join(".codewatch").join("index.json")
}

pub fn load_index(project_root: &Path) -> Result<Index> {
    let path = get_index_path(project_root);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read index file at {:?}", path))?;
    let index: Index = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse index file at {:?}", path))?;
    Ok(index)
}

pub fn save_index(project_root: &Path, index: &Index) -> Result<()> {
    let path = get_index_path(project_root);
    let content = serde_json::to_string_pretty(index)?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write index file to {:?}", path))?;
    Ok(())
}

pub fn path_to_hash(relative_path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(relative_path.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)[..16].to_string()
}

pub fn normalize_relative_path(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts.pop();
            }
            Component::CurDir => {}
            _ => {}
        }
    }
    parts.join("/")
}

pub fn resolve_to_relative_path(project_root: &Path, input_path: &str) -> Option<String> {
    let input_path_buf = PathBuf::from(input_path);

    // 1. Try resolving relative to CWD
    if let Ok(cwd) = env::current_dir() {
        let abs_path = if input_path_buf.is_absolute() {
            input_path_buf.clone()
        } else {
            cwd.join(&input_path_buf)
        };
        // Canonicalize to resolve any .. or symlinks if the file exists on disk
        let abs_path_canonical = abs_path.canonicalize().unwrap_or(abs_path);

        if let Ok(rel) = abs_path_canonical.strip_prefix(project_root) {
            return Some(normalize_relative_path(rel));
        }
    }

    // 2. Try resolving relative to project root directly (if the file doesn't exist on disk anymore but user specified root-relative path)
    let root_rel = if input_path_buf.is_absolute() {
        if let Ok(rel) = input_path_buf.strip_prefix(project_root) {
            Some(rel.to_path_buf())
        } else {
            None
        }
    } else {
        Some(input_path_buf)
    };

    if let Some(rel_path) = root_rel {
        let normalized = normalize_relative_path(&rel_path);
        return Some(normalized);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_path_to_hash() {
        let hash1 = path_to_hash("src/main.rs");
        let hash2 = path_to_hash("src/main.rs");
        let hash3 = path_to_hash("src/commands/mod.rs");

        assert_eq!(hash1.len(), 16);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_normalize_relative_path() {
        assert_eq!(normalize_relative_path(Path::new("a/b/../c")), "a/c");
        assert_eq!(normalize_relative_path(Path::new("a/./b")), "a/b");
        assert_eq!(normalize_relative_path(Path::new("a/b/c")), "a/b/c");
    }

    #[test]
    fn test_resolve_to_relative_path() {
        let root = Path::new("/workspace");
        // Relative input
        let resolved = resolve_to_relative_path(root, "src/main.rs").unwrap();
        assert_eq!(resolved, "src/main.rs");

        // Absolute input within root
        let resolved_abs = resolve_to_relative_path(root, "/workspace/src/commands/init.rs").unwrap();
        assert_eq!(resolved_abs, "src/commands/init.rs");
    }
}

