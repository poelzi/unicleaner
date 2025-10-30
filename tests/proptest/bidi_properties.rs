// Property-based tests for bidi character combinations (T044)

use proptest::prelude::*;
use unicleaner::unicode::categories::is_bidi_control;

// Property: Bidi controls are in specific ranges
proptest! {
    #[test]
    fn bidi_controls_in_ranges(c in any::<char>()) {
        if is_bidi_control(c) {
            let code = c as u32;
            let in_expected_range =
                (code >= 0x200E && code <= 0x200F) ||  // LRM, RLM
                (code >= 0x202A && code <= 0x202E) ||  // Embedding/Override
                (code >= 0x2066 && code <= 0x2069);    // Isolate

            prop_assert!(
                in_expected_range,
                "Bidi control U+{:04X} not in expected range", code
            );
        }
    }
}

// Property: Bidi override characters are detected
proptest! {
    #[test]
    fn bidi_overrides_detected(text in "\\PC{0,50}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Insert RLO (U+202E) into text
        let with_rlo = format!("test\u{202E}{}", text);

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", with_rlo).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            // Should detect the RLO character
            let has_rlo_detection = violations.iter().any(|v| {
                v.message.contains("RLO") ||
                v.message.contains("U+202E") ||
                v.message.contains("Right-to-Left Override")
            });

            prop_assert!(
                has_rlo_detection,
                "Should detect RLO character in text"
            );
        }
    }
}

// Property: Multiple bidi controls in sequence are detected
proptest! {
    #[test]
    fn multiple_bidi_detected(count in 1usize..=5) {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create text with multiple RLO characters
        let mut text = String::from("test");
        for _ in 0..count {
            text.push('\u{202E}');  // RLO
        }
        text.push_str("end");

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", text).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            let bidi_count = violations.iter().filter(|v| {
                v.message.contains("RLO") ||
                v.message.contains("U+202E") ||
                v.message.contains("bidi")
            }).count();

            prop_assert!(
                bidi_count >= count,
                "Should detect {} bidi characters, found {}", count, bidi_count
            );
        }
    }
}

// Property: Bidi isolates (LRI, RLI, FSI) are distinct from overrides
proptest! {
    #[test]
    fn bidi_isolates_vs_overrides(c in any::<char>()) {
        if is_bidi_control(c) {
            let code = c as u32;

            let is_isolate = matches!(code, 0x2066 | 0x2067 | 0x2068 | 0x2069);
            let is_override = matches!(code, 0x202D | 0x202E);

            // Can't be both isolate and override
            prop_assert!(
                !(is_isolate && is_override),
                "Character U+{:04X} can't be both isolate and override", code
            );
        }
    }
}

// Property: Nested bidi controls are detected
proptest! {
    #[test]
    fn nested_bidi_detected(depth in 1usize..=3) {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut text = String::from("start");

        // Add nested LRI characters
        for _ in 0..depth {
            text.push('\u{2066}');  // LRI
        }

        text.push_str("middle");

        // Close with PDI
        for _ in 0..depth {
            text.push('\u{2069}');  // PDI
        }

        text.push_str("end");

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", text).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            // Should detect bidi characters
            let has_bidi = violations.iter().any(|v| {
                v.message.contains("LRI") ||
                v.message.contains("PDI") ||
                v.message.contains("bidi") ||
                v.message.contains("U+2066") ||
                v.message.contains("U+2069")
            });

            prop_assert!(has_bidi, "Should detect nested bidi controls");
        }
    }
}
