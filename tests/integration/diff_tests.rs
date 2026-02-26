//! Integration tests for --diff mode

use assert_cmd::cargo_bin_cmd;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn run_git(dir: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
fn test_diff_mode_accepts_file_path_and_scans_changed_file() {
    if !git_available() {
        eprintln!("Skipping test: git not available");
        return;
    }

    let temp = TempDir::new().unwrap();
    let repo = temp.path();

    if !run_git(repo, &["init"]) {
        eprintln!("Skipping test: git init failed");
        return;
    }

    // Configure user for commits
    run_git(repo, &["config", "user.name", "Test User"]);
    run_git(repo, &["config", "user.email", "test@example.com"]);

    // Initial clean commit
    let file_path = repo.join("file.txt");
    fs::write(&file_path, "hello world\n").unwrap();
    run_git(repo, &["add", "file.txt"]);
    if !run_git(repo, &["commit", "-m", "init"]) {
        eprintln!("Skipping test: git commit failed");
        return;
    }

    // Modify the file to include a violation
    fs::write(&file_path, "hello\u{200B}world\n").unwrap();

    // First scan path is a file (regression for --diff base path handling)
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .current_dir(repo)
        .arg("scan")
        .arg("--diff")
        .arg("--format")
        .arg("json")
        .arg("file.txt")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["code_point"].as_u64() == Some(0x200B)),
        "Should report ZWSP (U+200B), got: {}",
        stdout
    );
}
