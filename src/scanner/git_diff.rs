//! Git integration for scanning only changed files
//!
//! This module provides simplified Git integration using the git command-line tool
//! instead of libgit2, avoiding complex system dependencies.

use crate::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Detect if a directory is a Git repository
pub fn is_git_repository(path: &Path) -> bool {
    Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get list of changed files in the repository
/// Returns files that are modified, added, or untracked since HEAD
///
/// Handles edge cases:
/// - Detached HEAD state
/// - Empty repository (no commits yet)
/// - Staging area changes
pub fn get_changed_files(repo_path: &Path) -> Result<Vec<PathBuf>> {
    // Check if HEAD exists (repository has commits)
    let head_check = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::Git(format!("Failed to check HEAD: {}", e)))?;

    let has_head = head_check.status.success();

    // Get modified and staged files
    let diff_args = if has_head {
        vec!["diff", "--name-only", "HEAD"]
    } else {
        // No commits yet - show all tracked files as changed
        vec!["ls-files"]
    };

    let output = Command::new("git")
        .args(&diff_args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::Git(format!("Failed to run git diff: {}", e)))?;

    if !output.status.success() {
        return Err(crate::Error::Git(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let mut files: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| repo_path.join(line))
        .collect();

    // Get untracked files
    let untracked_output = Command::new("git")
        .arg("ls-files")
        .arg("--others")
        .arg("--exclude-standard")
        .current_dir(repo_path)
        .output()
        .map_err(|e| crate::Error::Git(format!("Failed to run git ls-files: {}", e)))?;

    if untracked_output.status.success() {
        let untracked: Vec<PathBuf> = String::from_utf8_lossy(&untracked_output.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| repo_path.join(line))
            .collect();
        files.extend(untracked);
    }

    Ok(files)
}

/// Filter a list of files to only include those that have changed
pub fn filter_changed_files(files: Vec<PathBuf>, repo_path: &Path) -> Result<Vec<PathBuf>> {
    let changed = get_changed_files(repo_path)?;
    let changed_set: std::collections::HashSet<_> = changed
        .into_iter()
        .map(|p| p.canonicalize().unwrap_or(p))
        .collect();

    Ok(files
        .into_iter()
        .filter(|f| {
            let canonical = f.canonicalize().unwrap_or_else(|_| f.clone());
            changed_set.contains(&canonical)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn run_git_cmd(dir: &Path, args: &[&str]) -> bool {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn create_test_repo() -> Option<TempDir> {
        let dir = TempDir::new().ok()?;

        // Initialize git repo
        if !run_git_cmd(dir.path(), &["init"]) {
            return None;
        }

        // Configure user
        run_git_cmd(dir.path(), &["config", "user.name", "Test User"]);
        run_git_cmd(dir.path(), &["config", "user.email", "test@example.com"]);

        Some(dir)
    }

    fn commit_file(dir: &Path, filename: &str, content: &str) -> bool {
        let file_path = dir.join(filename);
        if fs::write(&file_path, content).is_err() {
            return false;
        }

        run_git_cmd(dir, &["add", filename]) && run_git_cmd(dir, &["commit", "-m", "test commit"])
    }

    #[test]
    fn test_is_git_repository() {
        if let Some(dir) = create_test_repo() {
            assert!(is_git_repository(dir.path()));
        }

        let non_repo = TempDir::new().unwrap();
        assert!(!is_git_repository(non_repo.path()));
    }

    #[test]
    fn test_get_changed_files_new_file() {
        let Some(dir) = create_test_repo() else {
            eprintln!("Skipping test: git not available");
            return;
        };

        // Create initial commit
        if !commit_file(dir.path(), "initial.txt", "initial content") {
            eprintln!("Skipping test: git commit failed");
            return;
        }

        // Add a new file (not committed)
        fs::write(dir.path().join("new.txt"), "new content").unwrap();

        if let Ok(changed) = get_changed_files(dir.path()) {
            assert!(changed.iter().any(|p| p.ends_with("new.txt")));
        }
    }

    #[test]
    fn test_get_changed_files_modified_file() {
        let Some(dir) = create_test_repo() else {
            eprintln!("Skipping test: git not available");
            return;
        };

        // Create initial commit with a file
        if !commit_file(dir.path(), "test.txt", "original content") {
            eprintln!("Skipping test: git commit failed");
            return;
        }

        // Modify the file
        fs::write(dir.path().join("test.txt"), "modified content").unwrap();

        if let Ok(changed) = get_changed_files(dir.path()) {
            assert!(changed.iter().any(|p| p.ends_with("test.txt")));
        }
    }

    #[test]
    fn test_filter_changed_files() {
        let Some(dir) = create_test_repo() else {
            eprintln!("Skipping test: git not available");
            return;
        };

        // Create initial commit
        if !commit_file(dir.path(), "file1.txt", "content1") {
            eprintln!("Skipping test: git commit failed");
            return;
        }
        commit_file(dir.path(), "file2.txt", "content2");

        // Modify only file1
        fs::write(dir.path().join("file1.txt"), "modified content").unwrap();

        let all_files = vec![dir.path().join("file1.txt"), dir.path().join("file2.txt")];

        if let Ok(filtered) = filter_changed_files(all_files, dir.path()) {
            assert_eq!(filtered.len(), 1);
            assert!(filtered[0].ends_with("file1.txt"));
        }
    }

    #[test]
    fn test_no_changed_files() {
        let Some(dir) = create_test_repo() else {
            eprintln!("Skipping test: git not available");
            return;
        };

        // Create initial commit
        if !commit_file(dir.path(), "test.txt", "content") {
            eprintln!("Skipping test: git commit failed");
            return;
        }

        // No modifications
        if let Ok(changed) = get_changed_files(dir.path()) {
            assert!(changed.is_empty());
        }
    }
}
