use crate::config::find_project_root;
use crate::index::{load_index, FileIndex};
use crate::scorer::format_size;
use anyhow::{anyhow, Result};

pub fn run(n: usize, offset: usize) -> Result<()> {
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

    let start = std::cmp::min(offset, files_list.len());
    let end = std::cmp::min(start + n, files_list.len());
    let sub_list = &files_list[start..end];

    let max_path_len = sub_list
        .iter()
        .map(|f| f.path.len())
        .max()
        .unwrap_or(0);

    println!();
    println!("Top {} files by importance:", n);

    for (idx, f) in sub_list.iter().enumerate() {
        let size_str = format_size(f.file_size);
        println!(
            "{:>2}. {:>4.1}  {:<width$}  (commits: {}, size: {}, refs: {})",
            offset + idx + 1,
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_top_sorting_and_paging() {
        let f1 = FileIndex {
            path: "a.rs".to_string(),
            score: 50.0,
            git_commits: 10,
            file_size: 100,
            ref_count: 5,
            last_modified: Utc::now(),
            score_updated_at: Utc::now(),
        };
        let f2 = FileIndex {
            path: "b.rs".to_string(),
            score: 80.0,
            git_commits: 2,
            file_size: 500,
            ref_count: 1,
            last_modified: Utc::now(),
            score_updated_at: Utc::now(),
        };
        let f3 = FileIndex {
            path: "c.rs".to_string(),
            score: 60.0,
            git_commits: 2,
            file_size: 500,
            ref_count: 1,
            last_modified: Utc::now(),
            score_updated_at: Utc::now(),
        };

        let files = vec![&f1, &f2, &f3];
        let mut files_list = files.clone();
        files_list.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sorted should be f2 (80.0), f3 (60.0), f1 (50.0)
        assert_eq!(files_list[0].path, "b.rs");
        assert_eq!(files_list[1].path, "c.rs");
        assert_eq!(files_list[2].path, "a.rs");

        // Page with n=2, offset=0
        let start = std::cmp::min(0, files_list.len());
        let end = std::cmp::min(start + 2, files_list.len());
        let sub_list = &files_list[start..end];
        assert_eq!(sub_list.len(), 2);
        assert_eq!(sub_list[0].path, "b.rs");
        assert_eq!(sub_list[1].path, "c.rs");

        // Page with n=2, offset=1
        let start = std::cmp::min(1, files_list.len());
        let end = std::cmp::min(start + 2, files_list.len());
        let sub_list = &files_list[start..end];
        assert_eq!(sub_list.len(), 2);
        assert_eq!(sub_list[0].path, "c.rs");
        assert_eq!(sub_list[1].path, "a.rs");

        // Page with n=2, offset=3
        let start = std::cmp::min(3, files_list.len());
        let end = std::cmp::min(start + 2, files_list.len());
        let sub_list = &files_list[start..end];
        assert!(sub_list.is_empty());
    }
}
