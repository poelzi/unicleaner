//! Fuzz target for parallel scanner (race conditions and thread safety)
//! Tests that parallel scanning never panics with arbitrary file lists and
//! thread counts

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;
use unicleaner::config::Configuration;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Use first byte to determine thread count
    let thread_count = (data[0] as usize) % 256;

    // Use remaining data for paths
    if let Ok(paths_str) = std::str::from_utf8(&data[1..]) {
        let paths: Vec<PathBuf> = paths_str
            .lines()
            .take(100) // Limit to avoid resource exhaustion
            .map(PathBuf::from)
            .collect();

        if !paths.is_empty() {
            let config = Configuration::default();

            // Test with None (default thread count)
            let _ = unicleaner::scanner::parallel::scan_files_parallel(
                paths.clone(),
                None,
                &config,
                None,
            );

            // Test with specific thread count from fuzz input
            if thread_count > 0 {
                let _ = unicleaner::scanner::parallel::scan_files_parallel(
                    paths.clone(),
                    Some(thread_count),
                    &config,
                    None,
                );
            }

            // Test edge cases
            let _ = unicleaner::scanner::parallel::scan_files_parallel(
                paths.clone(),
                Some(1),
                &config,
                None,
            );
            let _ = unicleaner::scanner::parallel::scan_files_parallel(
                paths.clone(),
                Some(100),
                &config,
                None,
            );
        }
    }
});
