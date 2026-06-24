pub struct ScorerInput {
    pub git_commits: u32,
    pub file_size: u64,
    pub ref_count: u32,
}

#[allow(dead_code)]
pub struct ScorerOutput {
    pub score: f64,
    pub git_commits_score: f64,
    pub file_size_score: f64,
    pub ref_count_score: f64,
}

pub fn calculate_scores(inputs: &[ScorerInput]) -> Vec<ScorerOutput> {
    if inputs.is_empty() {
        return Vec::new();
    }

    let mut min_git = inputs[0].git_commits as f64;
    let mut max_git = inputs[0].git_commits as f64;
    let mut min_size = inputs[0].file_size as f64;
    let mut max_size = inputs[0].file_size as f64;
    let mut min_ref = inputs[0].ref_count as f64;
    let mut max_ref = inputs[0].ref_count as f64;

    for input in inputs {
        let git = input.git_commits as f64;
        let size = input.file_size as f64;
        let rcount = input.ref_count as f64;

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

    inputs
        .iter()
        .map(|input| {
            let git = input.git_commits as f64;
            let size = input.file_size as f64;
            let rcount = input.ref_count as f64;

            let git_score = normalize(git, min_git, max_git);
            let size_score = normalize(size, min_size, max_size);
            let ref_score = normalize(rcount, min_ref, max_ref);

            // Weights: Git 40%, Size 30%, Refs 30%
            let score = (git_score * 0.4) + (size_score * 0.3) + (ref_score * 0.3);

            ScorerOutput {
                score,
                git_commits_score: git_score,
                file_size_score: size_score,
                ref_count_score: ref_score,
            }
        })
        .collect()
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1024), "1.0KB");
        assert_eq!(format_size(1024 * 1024), "1.0MB");
        assert_eq!(format_size(1500 * 1024), "1.5MB");
    }

    #[test]
    fn test_calculate_scores_empty() {
        let scores = calculate_scores(&[]);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_calculate_scores_single() {
        let inputs = vec![ScorerInput {
            git_commits: 10,
            file_size: 2000,
            ref_count: 5,
        }];
        let scores = calculate_scores(&inputs);
        assert_eq!(scores.len(), 1);
        // Since min == max for all values, normalize returns 100.0 if value > 0.0
        assert_eq!(scores[0].score, 100.0);
    }

    #[test]
    fn test_calculate_scores_multiple() {
        let inputs = vec![
            ScorerInput {
                git_commits: 0,
                file_size: 1000,
                ref_count: 0,
            },
            ScorerInput {
                git_commits: 10,
                file_size: 2000,
                ref_count: 20,
            },
        ];
        let scores = calculate_scores(&inputs);
        assert_eq!(scores.len(), 2);
        // Lowest values get normalized to 0.0
        assert_eq!(scores[0].score, 0.0);
        // Highest values get normalized to 100.0
        assert_eq!(scores[1].score, 100.0);
    }
}

