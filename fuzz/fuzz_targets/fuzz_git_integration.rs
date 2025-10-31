//! Fuzz target for git integration (command execution safety)
//! Tests that git operations never panic and handle malicious paths safely

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;
use tempfile::TempDir;

fuzz_target!(|data: &[u8]| {
    // Test with arbitrary path strings
    if let Ok(path_str) = std::str::from_utf8(data) {
        let path = PathBuf::from(path_str);

        // Test is_git_repository with arbitrary paths - should never panic
        let _ = unicleaner::scanner::git_diff::is_git_repository(&path);

        // Test get_changed_files - should handle non-repos gracefully
        let _ = unicleaner::scanner::git_diff::get_changed_files(&path);

        // Create temp dir and test with valid directory
        if let Ok(temp_dir) = TempDir::new() {
            let _ = unicleaner::scanner::git_diff::is_git_repository(temp_dir.path());
            let _ = unicleaner::scanner::git_diff::get_changed_files(temp_dir.path());

            // Test filter_changed_files with arbitrary paths
            let paths: Vec<PathBuf> = path_str
                .lines()
                .take(10) // Limit to avoid resource exhaustion
                .map(PathBuf::from)
                .collect();

            if !paths.is_empty() {
                let _ = unicleaner::scanner::git_diff::filter_changed_files(paths, temp_dir.path());
            }
        }
    }
});
