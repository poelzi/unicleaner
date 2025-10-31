#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;
use unicleaner::scanner::unicode_detector::detect_in_string;
use unicleaner::unicode::malicious::MaliciousCategory;

fuzz_target!(|data: &[u8]| {
    // Try to convert bytes to a valid UTF-8 string
    if let Ok(text) = std::str::from_utf8(data) {
        // Fuzz the Unicode detection logic
        let path = Path::new("fuzz_input.txt");
        let violations = detect_in_string(text, path);

        // Verify invariants:
        // 1. Detection should never panic
        // 2. All violations should have valid properties
        for violation in violations {
            // Verify line and column are at least 1
            assert!(violation.line >= 1, "Line number should be at least 1");
            assert!(violation.column >= 1, "Column number should be at least 1");

            // Verify category is valid
            match violation.category {
                MaliciousCategory::ZeroWidth
                | MaliciousCategory::BidiOverride
                | MaliciousCategory::Homoglyph
                | MaliciousCategory::ControlChar
                | MaliciousCategory::NonStandard => {}
            }

            // Verify code point is valid Unicode (0 to 0x10FFFF)
            assert!(violation.code_point <= 0x10FFFF, "Invalid code point");
        }
    }
});
