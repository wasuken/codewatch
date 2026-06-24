use anyhow::{Context, Result};
use chrono::Local;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn get_note_path(project_root: &Path, file_hash: &str) -> PathBuf {
    project_root
        .join(".codewatch")
        .join("notes")
        .join(format!("{}.md", file_hash))
}

pub fn read_note_content(project_root: &Path, file_hash: &str) -> Option<String> {
    let path = get_note_path(project_root, file_hash);
    if path.is_file() {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

pub fn create_note_template(
    project_root: &Path,
    file_hash: &str,
    relative_path: &str,
) -> Result<()> {
    let path = get_note_path(project_root, file_hash);

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let template = format!(
        "# {}\n\n理解度: 0/5\n最終確認: {}\n\n<!-- ここにメモを書く -->\n",
        relative_path, today
    );

    fs::write(&path, template)
        .with_context(|| format!("Failed to write note template to {:?}", path))?;

    Ok(())
}

pub fn open_in_editor(note_path: &Path) -> Result<()> {
    let editor_env = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut parts = editor_env.split_whitespace();
    if let Some(editor_cmd) = parts.next() {
        let mut cmd = Command::new(editor_cmd);
        for arg in parts {
            cmd.arg(arg);
        }
        cmd.arg(note_path);

        let status = cmd
            .status()
            .with_context(|| format!("Failed to run editor '{}'", editor_env))?;
        if !status.success() {
            return Err(anyhow::anyhow!("Editor exited with non-zero status"));
        }
    } else {
        return Err(anyhow::anyhow!("EDITOR env variable was empty"));
    }
    Ok(())
}
