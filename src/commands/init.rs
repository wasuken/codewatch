use crate::config::{find_project_root, save_config, Config};
use crate::index::{save_index, Index};
use anyhow::{Context, Result};
use std::env;
use std::fs;

pub fn run() -> Result<()> {
    if let Some(root) = find_project_root() {
        eprintln!("Error: .codewatch/ already exists in {:?}", root);
        std::process::exit(1);
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let codewatch_dir = current_dir.join(".codewatch");

    // Create directories
    fs::create_dir_all(&codewatch_dir).context("Failed to create .codewatch directory")?;

    fs::create_dir_all(codewatch_dir.join("notes")).context("Failed to create notes directory")?;

    // Create config.yml
    let config = Config::default();
    save_config(&current_dir, &config).context("Failed to save default config")?;

    // Create empty index.json
    let index = Index {
        version: 1,
        last_scan: chrono::Utc::now(),
        files: std::collections::HashMap::new(),
    };
    save_index(&current_dir, &index).context("Failed to save empty index")?;

    println!("Initialized .codewatch/ in {}", current_dir.display());
    Ok(())
}
