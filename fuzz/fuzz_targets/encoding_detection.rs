//! Fuzz target for encoding detection (T053)
//! Tests that encoding detection never panics on arbitrary byte sequences

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Create a temp file with the fuzz data
    if let Ok(mut temp) = NamedTempFile::new() {
        // Write arbitrary bytes
        let _ = temp.write_all(data);
        let _ = temp.flush();

        // Try to detect encoding - should never panic
        let _ = unicleaner::scanner::encoding::detect_encoding(temp.path());

        // Try to decode the file - should handle any encoding gracefully
        let _ = unicleaner::scanner::encoding::detect_and_decode(temp.path());
    }
});
