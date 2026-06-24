use crate::config::find_project_root;
use crate::index::{load_index, path_to_hash, resolve_to_relative_path};
use crate::note::read_note_content;
use crate::scorer::format_size;
use anyhow::{anyhow, Result};

pub fn run(file: &str) -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let index = load_index(&project_root)?;

    let relative_path = resolve_to_relative_path(&project_root, file)
        .ok_or_else(|| anyhow!("Error: File not found in index. Run `cw scan` first."))?;

    let hash = path_to_hash(&relative_path);
    let file_info = index
        .files
        .get(&hash)
        .ok_or_else(|| anyhow!("Error: File not found in index. Run `cw scan` first."))?;

    // Calculate individual normalized scores on the fly
    let mut min_git = f64::MAX;
    let mut max_git = f64::MIN;
    let mut min_size = f64::MAX;
    let mut max_size = f64::MIN;
    let mut min_ref = f64::MAX;
    let mut max_ref = f64::MIN;

    for f in index.files.values() {
        let git = f.git_commits as f64;
        let size = f.file_size as f64;
        let rcount = f.ref_count as f64;

        if git < min_git {
            min_git = git;
        }
        if git > max_git {
            max_git = git;
        }
        if size < min_size {
            min_size = size;
        }
        if size > max_size {
            max_size = size;
        }
        if rcount < min_ref {
            min_ref = rcount;
        }
        if rcount > max_ref {
            max_ref = rcount;
        }
    }

    let normalize = |val: f64, min: f64, max: f64| -> f64 {
        if max == min {
            if max > 0.0 {
                100.0
            } else {
                0.0
            }
        } else {
            ((val - min) / (max - min)) * 100.0
        }
    };

    let git_score = normalize(file_info.git_commits as f64, min_git, max_git);
    let size_score = normalize(file_info.file_size as f64, min_size, max_size);
    let ref_score = normalize(file_info.ref_count as f64, min_ref, max_ref);

    let last_modified_str = file_info.last_modified.format("%Y-%m-%d").to_string();
    let last_scanned_str = index.last_scan.format("%Y-%m-%d").to_string();

    println!("File: {}", file_info.path);
    println!("Hash: {}", hash);
    println!("Score: {:.1}", file_info.score);
    println!();
    println!(
        "  Git commits:  {}  (score: {:.1})",
        file_info.git_commits, git_score
    );
    println!(
        "  File size:    {} (score: {:.1})",
        format_size(file_info.file_size),
        size_score
    );
    println!(
        "  Ref count:    {}    (score: {:.1})",
        file_info.ref_count, ref_score
    );
    println!();
    println!("Last modified: {}", last_modified_str);
    println!("Last scanned:  {}", last_scanned_str);
    println!();

    println!("--- Notes ---");
    if let Some(notes) = read_note_content(&project_root, &hash) {
        print!("{}", notes);
    } else {
        println!("No notes yet. Run `cw note {}` to add.", file_info.path);
    }

    Ok(())
}
