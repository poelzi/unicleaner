#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;
use unicleaner::config::Configuration;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as TOML config
    if let Ok(text) = std::str::from_utf8(data) {
        // Write to a temporary file
        if let Ok(mut temp_file) = NamedTempFile::new() {
            if temp_file.write_all(text.as_bytes()).is_ok() {
                // Attempt to load config from file - should never panic
                let _result = Configuration::from_file(temp_file.path());

                // We don't care if parsing succeeds or fails, just that it
                // doesn't panic Invalid TOML should return Err,
                // not panic
            }
        }
    }
});
