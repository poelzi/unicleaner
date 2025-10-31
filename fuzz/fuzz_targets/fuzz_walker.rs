//! Fuzz target for directory walker (path traversal and symlink safety)
//! Tests that directory walking never panics and handles symlinks safely

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;
use unicleaner::scanner::walker::{walk_paths, WalkConfig};

fuzz_target!(|data: &[u8]| {
    if let Ok(paths_str) = std::str::from_utf8(data) {
        // Split on newlines to create multiple paths
        let paths: Vec<PathBuf> = paths_str
            .lines()
            .take(50) // Limit to avoid resource exhaustion
            .map(PathBuf::from)
            .collect();

        if !paths.is_empty() {
            // Test with default config
            let _ = walk_paths(&paths, &WalkConfig::default());

            // Test with various configurations
            let configs = vec![
                WalkConfig {
                    follow_links: true,
                    max_depth: Some(5),
                    respect_gitignore: true,
                },
                WalkConfig {
                    follow_links: false,
                    max_depth: Some(1),
                    respect_gitignore: false,
                },
                WalkConfig {
                    follow_links: true,
                    max_depth: None,
                    respect_gitignore: true,
                },
            ];

            for config in configs {
                let _ = walk_paths(&paths, &config);
            }
        }
    }
});
