use crate::config::find_project_root;
use crate::index::{load_index, path_to_hash, resolve_to_relative_path};
use crate::note::{create_note_template, get_note_path, open_in_editor};
use anyhow::{anyhow, Result};

pub fn run(file: &str) -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let index = load_index(&project_root)?;

    let relative_path = resolve_to_relative_path(&project_root, file)
        .ok_or_else(|| anyhow!("Error: File not found in index. Run `cw scan` first."))?;

    let hash = path_to_hash(&relative_path);

    // Check if file is in index
    if !index.files.contains_key(&hash) {
        return Err(anyhow!(
            "Error: File not found in index. Run `cw scan` first."
        ));
    }

    let note_path = get_note_path(&project_root, &hash);
    if !note_path.is_file() {
        create_note_template(&project_root, &hash, &relative_path)?;
    }

    let editor_env = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    println!("Opening notes in {}...", editor_env);

    open_in_editor(&note_path)?;

    Ok(())
}
