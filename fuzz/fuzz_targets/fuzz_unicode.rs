#![no_main]

use libfuzzer_sys::fuzz_target;
use unicleaner::unicode::malicious::{detect_malicious_unicode, MaliciousCategory};

fuzz_target!(|data: &[u8]| {
    // Try to convert bytes to a valid UTF-8 string
    if let Ok(text) = std::str::from_utf8(data) {
        // Fuzz the Unicode detection logic
        let detections = detect_malicious_unicode(text);

        // Verify invariants:
        // 1. Detection should never panic
        // 2. All detections should have valid indices within the string
        for detection in detections {
            assert!(
                detection.byte_offset < text.len(),
                "Byte offset out of bounds"
            );

            // Verify category is valid
            match detection.category {
                MaliciousCategory::ZeroWidth
                | MaliciousCategory::BidiOverride
                | MaliciousCategory::Homoglyph
                | MaliciousCategory::ControlChar
                | MaliciousCategory::NonStandard => {}
            }

            // Verify code point is valid Unicode (0 to 0x10FFFF)
            assert!(detection.code_point <= 0x10FFFF, "Invalid code point");
        }
    }
});
