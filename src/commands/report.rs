use crate::config::find_project_root;
use crate::index::load_index;
use crate::note::read_note_content;
use anyhow::{anyhow, Result};

pub fn run(n: usize) -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let index = load_index(&project_root)
        .map_err(|_| anyhow!("Error: File not found in index. Run `cw scan` first."))?;

    let mut noted_files = Vec::new();

    for (hash, file_info) in &index.files {
        if let Some(content) = read_note_content(&project_root, hash) {
            noted_files.push((file_info, content));
        }
    }

    if noted_files.is_empty() {
        println!("No notes found. Add notes with: cw note <file>");
        return Ok(());
    }

    // Sort by score in descending order
    noted_files.sort_by(|a, b| {
        b.0.score
            .partial_cmp(&a.0.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let limit = std::cmp::min(n, noted_files.len());

    println!("# Code Reading Report");
    println!();

    for (idx, (file_info, content)) in noted_files.iter().take(limit).enumerate() {
        if idx > 0 {
            println!();
        }
        println!("## {}. {} (score: {:.1})", idx + 1, file_info.path, file_info.score);
        println!();
        println!("{}", content.trim_end());
        println!();
        println!("---");
    }

    Ok(())
}
