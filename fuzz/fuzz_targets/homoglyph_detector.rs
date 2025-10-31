//! Fuzz target for Unicode detection with arbitrary characters
//! Tests that Unicode detection never panics on arbitrary Unicode strings and
//! byte sequences

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Try to interpret as UTF-8
    if let Ok(text) = std::str::from_utf8(data) {
        // Test Unicode detector directly on the string
        let path = std::path::Path::new("fuzz_input.txt");
        let _ = unicleaner::scanner::unicode_detector::detect_in_string(text, path);

        // Test scanning a file with this content
        if let Ok(mut temp) = NamedTempFile::new() {
            let _ = write!(temp, "{}", text);
            let _ = temp.flush();

            // Scan should never panic, even with arbitrary Unicode
            let _ = unicleaner::scanner::file_scanner::scan_file(temp.path());
        }
    }

    // Also test with raw bytes (may be invalid UTF-8)
    if let Ok(mut temp) = NamedTempFile::new() {
        let _ = temp.write_all(data);
        let _ = temp.flush();

        // Should handle invalid UTF-8 gracefully
        let _ = unicleaner::scanner::file_scanner::scan_file(temp.path());
    }
});
