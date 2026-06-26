use crate::config::{find_project_root, get_config_path, load_config, save_config};
use anyhow::{anyhow, Context, Result};
use std::fs;

pub fn run_upgrade() -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let config_path = get_config_path(&project_root);
    if !config_path.exists() {
        return Err(anyhow!("Error: Config file not found at {:?}", config_path));
    }

    let backup_path = config_path.with_extension("yml.bak");
    fs::copy(&config_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;
    println!("Backup created at {:?}", backup_path);

    let mut config = load_config(&project_root)?;
    config.upgrade();

    save_config(&project_root, &config)
        .with_context(|| format!("Failed to save upgraded config to {:?}", config_path))?;

    println!("Configuration successfully upgraded to the new format.");
    Ok(())
}
