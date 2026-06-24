use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{find_project_root, load_config};
use crate::git::{check_git_availability, get_git_commits};
use crate::index::{normalize_relative_path, path_to_hash, save_index, FileIndex, Index};
use crate::parser::parse_imports;
use crate::scorer::{calculate_scores, format_size, ScorerInput};

pub struct ExcludeMatcher {
    regexes: Vec<Regex>,
}

impl ExcludeMatcher {
    pub fn new(patterns: &[String]) -> Result<Self> {
        let mut regexes = Vec::new();
        for pattern in patterns {
            let mut regex_str = String::new();
            regex_str.push_str("^");
            for c in pattern.chars() {
                match c {
                    '*' => regex_str.push_str(".*"),
                    '?' => regex_str.push_str("."),
                    '.' | '+' | '(' | ')' | '[' | ']' | '^' | '$' | '{' | '}' | '\\' | '|' => {
                        regex_str.push('\\');
                        regex_str.push(c);
                    }
                    _ => regex_str.push(c),
                }
            }
            regex_str.push_str("$");
            let re = Regex::new(&regex_str)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            regexes.push(re);
        }
        Ok(Self { regexes })
    }

    pub fn is_excluded(&self, relative_path: &str) -> bool {
        let components: Vec<&str> = relative_path.split('/').collect();
        for re in &self.regexes {
            for comp in &components {
                if re.is_match(comp) {
                    return true;
                }
            }
            if re.is_match(relative_path) {
                return true;
            }
        }
        false
    }
}

pub fn print_progress(current: usize, total: usize) {
    let width = 20;
    let progress = if total > 0 {
        (current * width) / total
    } else {
        width
    };

    let bar: String = std::iter::repeat('=')
        .take(progress)
        .chain(std::iter::repeat(' ').take(width - progress))
        .collect();

    print!("\r[{}] {}/{}", bar, current, total);
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}

pub fn run() -> Result<()> {
    let project_root = find_project_root()
        .ok_or_else(|| anyhow!("Error: .codewatch/ not found. Run `cw init` first."))?;

    let config = load_config(&project_root)?;
    let exclude_matcher = ExcludeMatcher::new(&config.exclude_patterns)?;

    // Check git availability
    let git_available = check_git_availability();
    if !git_available {
        eprintln!("Warning: git not found. Git commit scores will be 0.");
    }

    // Walk directory to find files
    let mut files = Vec::new();
    let walker = ignore::WalkBuilder::new(&project_root)
        .standard_filters(true)
        .build();

    for entry in walker {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel_path) = path.strip_prefix(&project_root) {
                    let rel_str = normalize_relative_path(rel_path);

                    if rel_str.starts_with(".codewatch") {
                        continue;
                    }

                    let has_target_ext = config
                        .target_extensions
                        .iter()
                        .any(|ext| rel_str.ends_with(ext));

                    if has_target_ext && !exclude_matcher.is_excluded(&rel_str) {
                        files.push((path.to_path_buf(), rel_str));
                    }
                }
            }
        }
    }

    let total_files = files.len();
    println!("Scanning {} files...", total_files);

    // Parse imports and compute sizes, git commits
    let mut raw_data = Vec::new();
    let mut all_imports = HashMap::new();

    print_progress(0, total_files);

    for (idx, (abs_path, rel_path)) in files.iter().enumerate() {
        let file_size = fs::metadata(abs_path).map(|m| m.len()).unwrap_or(0);

        let last_modified = fs::metadata(abs_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| std::time::SystemTime::now());
        let last_modified_dt: chrono::DateTime<Utc> = last_modified.into();

        let git_commits = if git_available {
            get_git_commits(&project_root, rel_path).unwrap_or(0)
        } else {
            0
        };

        let content = fs::read_to_string(abs_path).unwrap_or_default();
        let imports = parse_imports(&content);
        all_imports.insert(rel_path.clone(), imports);

        raw_data.push((rel_path.clone(), git_commits, file_size, last_modified_dt));
        print_progress(idx + 1, total_files);
    }
    println!();

    // Compute reference counts
    let mut ref_counts = HashMap::new();
    for (rel_path, _, _, _) in &raw_data {
        let file_stem = Path::new(rel_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let mut count = 0;
        if !file_stem.is_empty() {
            for (other_path, other_imports) in &all_imports {
                if other_path != rel_path {
                    for imp in other_imports {
                        if imp.contains(file_stem) {
                            count += 1;
                        }
                    }
                }
            }
        }
        ref_counts.insert(rel_path.clone(), count);
    }

    // Compute scores
    let scorer_inputs: Vec<ScorerInput> = raw_data
        .iter()
        .map(|(rel_path, commits, size, _)| {
            let refs = *ref_counts.get(rel_path).unwrap_or(&0);
            ScorerInput {
                git_commits: *commits,
                file_size: *size,
                ref_count: refs,
            }
        })
        .collect();

    let scorer_outputs = calculate_scores(&scorer_inputs);

    // Build index
    let scan_time = Utc::now();
    let mut index_files = HashMap::new();

    for (idx, (rel_path, commits, size, last_modified)) in raw_data.iter().enumerate() {
        let refs = *ref_counts.get(rel_path).unwrap_or(&0);
        let score_out = &scorer_outputs[idx];
        let hash = path_to_hash(rel_path);

        index_files.insert(
            hash,
            FileIndex {
                path: rel_path.clone(),
                score: score_out.score,
                git_commits: *commits,
                file_size: *size,
                ref_count: refs,
                last_modified: *last_modified,
                score_updated_at: scan_time,
            },
        );
    }

    let index = Index {
        version: 1,
        last_scan: scan_time,
        files: index_files,
    };

    save_index(&project_root, &index)?;

    // Display top 10
    println!();
    println!("Top 10 files by importance:");

    let mut files_list: Vec<&FileIndex> = index.files.values().collect();
    files_list.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_count = std::cmp::min(10, files_list.len());
    let max_path_len = files_list[..top_count]
        .iter()
        .map(|f| f.path.len())
        .max()
        .unwrap_or(0);

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
    println!("Scan complete. {} files indexed.", total_files);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclude_matcher() {
        let patterns = vec![
            "node_modules".to_string(),
            "dist".to_string(),
            "*.test.ts".to_string(),
        ];
        let matcher = ExcludeMatcher::new(&patterns).unwrap();

        // Exact component match
        assert!(matcher.is_excluded("node_modules/abc/def.ts"));
        assert!(matcher.is_excluded("src/dist/def.ts"));

        // Wildcard match
        assert!(matcher.is_excluded("src/components/button.test.ts"));

        // Non-excluded
        assert!(!matcher.is_excluded("src/components/button.ts"));
        assert!(!matcher.is_excluded("src/main.rs"));
    }
}

