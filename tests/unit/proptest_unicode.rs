//! Property-based tests for Unicode detection using proptest

use proptest::prelude::*;
use unicleaner::unicode::malicious::{detect_malicious_unicode, is_malicious_code_point};
use unicleaner::unicode::ranges::UnicodeRange;

proptest! {
    #[test]
    fn test_unicode_detection_never_panics(text in "\\PC*") {
        // Property: Unicode detection should never panic on any string
        let _ = detect_malicious_unicode(&text);
    }

    #[test]
    fn test_valid_code_points_only(text in "\\PC*") {
        // Property: All detected code points should be valid Unicode (≤ 0x10FFFF)
        let detections = detect_malicious_unicode(&text);
        for detection in detections {
            prop_assert!(detection.code_point <= 0x10FFFF);
        }
    }

    #[test]
    fn test_detection_indices_within_bounds(text in "\\PC*") {
        // Property: All byte offsets should be within string bounds
        let detections = detect_malicious_unicode(&text);
        for detection in detections {
            prop_assert!(detection.byte_offset < text.len());
        }
    }

    #[test]
    fn test_clean_ascii_has_no_violations(text in "[a-zA-Z0-9 \\n\\t]*") {
        // Property: Pure ASCII text should have no malicious Unicode
        let detections = detect_malicious_unicode(&text);
        prop_assert_eq!(detections.len(), 0);
    }

    #[test]
    fn test_unicode_range_contains_consistency(
        start in 0u32..0x10FFFF,
        end in 0u32..0x10FFFF,
        point in 0u32..0x10FFFF
    ) {
        // Property: contains() should be consistent with start/end bounds
        if start <= end {
            let range = UnicodeRange {
                start,
                end,
                name: "test".to_string(),
            };

            let should_contain = point >= start && point <= end;
            prop_assert_eq!(range.contains(point), should_contain);
        }
    }

    #[test]
    fn test_zero_width_detection(
        prefix in "[a-zA-Z]*",
        suffix in "[a-zA-Z]*"
    ) {
        // Property: Zero-width space should always be detected
        let text = format!("{}\u{200B}{}", prefix, suffix);
        let detections = detect_malicious_unicode(&text);

        // Should find at least the zero-width space
        prop_assert!(detections.len() > 0);
        prop_assert!(detections.iter().any(|d| d.code_point == 0x200B));
    }

    #[test]
    fn test_bidi_override_detection(
        prefix in "[a-zA-Z]*",
        suffix in "[a-zA-Z]*"
    ) {
        // Property: RLO (Right-to-Left Override) should always be detected
        let text = format!("{}\u{202E}{}", prefix, suffix);
        let detections = detect_malicious_unicode(&text);

        // Should find at least the RLO character
        prop_assert!(detections.len() > 0);
        prop_assert!(detections.iter().any(|d| d.code_point == 0x202E));
    }

    #[test]
    fn test_multiple_detections_ordered(text in "\\PC*") {
        // Property: Detections should be ordered by byte offset
        let detections = detect_malicious_unicode(&text);

        for i in 1..detections.len() {
            prop_assert!(detections[i - 1].byte_offset <= detections[i].byte_offset);
        }
    }

    #[test]
    fn test_is_malicious_consistent_with_detect(text in "\\PC{0,100}") {
        // Property: is_malicious_code_point should agree with detect_malicious_unicode
        let detections = detect_malicious_unicode(&text);

        for ch in text.chars() {
            let code_point = ch as u32;
            let detected = detections.iter().any(|d| d.code_point == code_point);
            let is_malicious = is_malicious_code_point(code_point);

            // If detected, must be malicious
            if detected {
                prop_assert!(is_malicious);
            }
        }
    }

    #[test]
    fn test_repeated_scanning_stable(text in "\\PC{0,100}") {
        // Property: Scanning same text multiple times should give same results
        let detections1 = detect_malicious_unicode(&text);
        let detections2 = detect_malicious_unicode(&text);

        prop_assert_eq!(detections1.len(), detections2.len());

        for (d1, d2) in detections1.iter().zip(detections2.iter()) {
            prop_assert_eq!(d1.code_point, d2.code_point);
            prop_assert_eq!(d1.byte_offset, d2.byte_offset);
            prop_assert_eq!(d1.category, d2.category);
        }
    }
}

#[cfg(test)]
mod manual_tests {
    use super::*;

    #[test]
    fn test_known_malicious_patterns() {
        // Test with known malicious patterns
        let test_cases = vec![
            ("\u{200B}", 1),                 // Zero-width space
            ("\u{200C}", 1),                 // Zero-width non-joiner
            ("\u{200D}", 1),                 // Zero-width joiner
            ("\u{202E}", 1),                 // Right-to-left override
            ("\u{202D}", 1),                 // Left-to-right override
            ("hello\u{200B}world", 1),       // Embedded zero-width
            ("\u{200B}\u{200C}\u{200D}", 3), // Multiple zero-width
            ("clean text", 0),               // No malicious characters
        ];

        for (text, expected_count) in test_cases {
            let detections = detect_malicious_unicode(text);
            assert_eq!(
                detections.len(),
                expected_count,
                "Failed for text: {:?}",
                text
            );
        }
    }
}
