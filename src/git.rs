use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn check_git_availability() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn get_git_commits(project_root: &Path, relative_path: &str) -> Result<u32> {
    let output = Command::new("git")
        .args(&["log", "--follow", "--oneline", relative_path])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines().count() as u32;
        Ok(count)
    } else {
        Ok(0)
    }
}
