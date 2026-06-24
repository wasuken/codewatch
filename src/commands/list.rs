use crate::config::find_project_root;
use crate::index::{load_index, FileIndex};
use anyhow::{anyhow, Result};

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum SortOrder {
    Score,
    Recent,
}

pub fn run(sort: &SortOrder, noted: bool, limit: Option<usize>) -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let index = load_index(&project_root)
        .map_err(|_| anyhow!("Error: File not found in index. Run `cw scan` first."))?;

    let mut files: Vec<&FileIndex> = index.files.values().collect();

    if noted {
        files.retain(|f| {
            let hash = crate::index::path_to_hash(&f.path);
            let path = crate::note::get_note_path(&project_root, &hash);
            path.is_file()
        });
    }

    match sort {
        SortOrder::Score => {
            files.sort_by(|a, b| {
                b.score.partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.path.cmp(&b.path))
            });
        }
        SortOrder::Recent => {
            let mut files_with_time: Vec<(&FileIndex, Option<std::time::SystemTime>)> = files
                .into_iter()
                .map(|f| {
                    let hash = crate::index::path_to_hash(&f.path);
                    let path = crate::note::get_note_path(&project_root, &hash);
                    let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
                    (f, mtime)
                })
                .collect();

            files_with_time.sort_by(|a, b| {
                match (a.1, b.1) {
                    (Some(ta), Some(tb)) => tb.cmp(&ta).then_with(|| a.0.path.cmp(&b.0.path)),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.0.path.cmp(&b.0.path),
                }
            });

            files = files_with_time.into_iter().map(|(f, _)| f).collect();
        }
    }

    let display_count = if let Some(l) = limit {
        std::cmp::min(l, files.len())
    } else {
        files.len()
    };

    if display_count == 0 {
        println!("No files in index.");
        return Ok(());
    }

    for f in files.iter().take(display_count) {
        println!("{}", f.path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::index::FileIndex;
    use chrono::Utc;

    #[test]
    fn test_sorting_logic_score() {
        let f1 = FileIndex {
            path: "b.rs".to_string(),
            score: 50.0,
            git_commits: 10,
            file_size: 100,
            ref_count: 5,
            last_modified: Utc::now(),
            score_updated_at: Utc::now(),
        };
        let f2 = FileIndex {
            path: "a.rs".to_string(),
            score: 50.0,
            git_commits: 2,
            file_size: 500,
            ref_count: 1,
            last_modified: Utc::now(),
            score_updated_at: Utc::now(),
        };
        let f3 = FileIndex {
            path: "c.rs".to_string(),
            score: 80.0,
            git_commits: 2,
            file_size: 500,
            ref_count: 1,
            last_modified: Utc::now(),
            score_updated_at: Utc::now(),
        };

        let mut files = vec![&f1, &f2, &f3];

        files.sort_by(|a, b| {
            b.score.partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
        });

        assert_eq!(files[0].path, "c.rs"); // score 80.0
        assert_eq!(files[1].path, "a.rs"); // score 50.0, path a.rs
        assert_eq!(files[2].path, "b.rs"); // score 50.0, path b.rs
    }

    #[test]
    fn test_sorting_logic_recent() {
        use std::fs;
        use std::thread;
        use std::time::Duration;

        let temp_dir = std::env::temp_dir().join(format!("cw_test_{}", Utc::now().timestamp_millis()));
        let notes_dir = temp_dir.join(".codewatch").join("notes");
        fs::create_dir_all(&notes_dir).unwrap();

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

        let hash1 = crate::index::path_to_hash(&f1.path);
        let note_path1 = notes_dir.join(format!("{}.md", hash1));
        fs::write(&note_path1, "note 1").unwrap();

        thread::sleep(Duration::from_millis(100));

        let hash2 = crate::index::path_to_hash(&f2.path);
        let note_path2 = notes_dir.join(format!("{}.md", hash2));
        fs::write(&note_path2, "note 2").unwrap();

        let files = vec![&f1, &f2, &f3];

        let mut files_with_time: Vec<(&FileIndex, Option<std::time::SystemTime>)> = files
            .into_iter()
            .map(|f| {
                let hash = crate::index::path_to_hash(&f.path);
                let path = notes_dir.join(format!("{}.md", hash));
                let mtime = fs::metadata(path).and_then(|m| m.modified()).ok();
                (f, mtime)
            })
            .collect();

        files_with_time.sort_by(|a, b| {
            match (a.1, b.1) {
                (Some(ta), Some(tb)) => tb.cmp(&ta).then_with(|| a.0.path.cmp(&b.0.path)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.0.path.cmp(&b.0.path),
            }
        });

        let sorted: Vec<&FileIndex> = files_with_time.into_iter().map(|(f, _)| f).collect();

        assert_eq!(sorted[0].path, "b.rs"); // most recent note
        assert_eq!(sorted[1].path, "a.rs"); // older note
        assert_eq!(sorted[2].path, "c.rs"); // no note

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
