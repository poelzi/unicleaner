//! Fuzz target for homoglyph detection (T054)
//! Tests that homoglyph detection never panics on arbitrary Unicode strings

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Try to interpret as UTF-8
    if let Ok(text) = std::str::from_utf8(data) {
        // Test homoglyph detection on individual characters
        for ch in text.chars() {
            let _ = unicleaner::unicode::categories::is_homoglyph_risk(ch);
            let _ = unicleaner::unicode::categories::get_script(ch);
        }

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
