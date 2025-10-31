//! Fuzz target for glob pattern matching
//! Tests that glob pattern parsing never panics with malformed patterns

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicleaner::config::rules::FileRule;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as glob pattern
    if let Ok(pattern) = std::str::from_utf8(data) {
        // Test FileRule creation with arbitrary patterns
        // Should handle invalid patterns gracefully
        let _ = FileRule::new(pattern);
    }
});
