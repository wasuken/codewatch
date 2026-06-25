use crate::config::find_project_root;
use crate::index::{load_index, FileIndex};
use crate::scorer::format_size;
use anyhow::{anyhow, Result};

pub fn run(n: usize) -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let index = load_index(&project_root)
        .map_err(|_| anyhow!("Error: File not found in index. Run `cw scan` first."))?;

    let mut files_list: Vec<&FileIndex> = index.files.values().collect();
    files_list.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_count = std::cmp::min(n, files_list.len());
    let max_path_len = files_list[..top_count]
        .iter()
        .map(|f| f.path.len())
        .max()
        .unwrap_or(0);

    println!();
    println!("Top {} files by importance:", n);

    for (idx, f) in files_list.iter().take(top_count).enumerate() {
        let size_str = format_size(f.file_size);
        println!(
            "{:>2}. {:>4.1}  {:<width$}  (commits: {}, size: {}, refs: {})",
            idx + 1,
            f.score,
            f.path,
            f.git_commits,
            size_str,
            f.ref_count,
            width = max_path_len
        );
    }

    println!();

    Ok(())
}
