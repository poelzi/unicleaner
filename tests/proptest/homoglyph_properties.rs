// Property-based tests for homoglyph detection accuracy (T045)

use proptest::prelude::*;
use unicleaner::unicode::categories::is_homoglyph_risk;

// Property: ASCII letters should never be homoglyph risks
proptest! {
    #[test]
    fn ascii_letters_not_risks(c in prop::char::range('A', 'z')) {
        // Only test actual letters (skips punctuation between Z and a)
        if c.is_ascii_alphabetic() {
            prop_assert!(
                !is_homoglyph_risk(c),
                "ASCII letter '{}' should not be homoglyph risk", c
            );
        }
    }
}

// Property: Known Cyrillic homoglyphs are detected
proptest! {
    #[test]
    fn cyrillic_homoglyphs_flagged(text in "\\PC{0,50}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Insert Cyrillic 'а' (U+0430) which looks like Latin 'a'
        let with_cyrillic = format!("test{}аdmin", text);  // Cyrillic а

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", with_cyrillic).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            let has_cyrillic = violations.iter().any(|v| {
                v.message.contains("Cyrillic") ||
                v.message.contains("U+0430") ||
                v.message.contains("homoglyph") ||
                v.message.contains("confusable")
            });

            prop_assert!(
                has_cyrillic,
                "Should detect Cyrillic homoglyph in text"
            );
        }
    }
}

// Property: Greek omicron (ο) vs Latin o detection
proptest! {
    #[test]
    fn greek_omicron_detected(prefix in "\\PC{0,20}", suffix in "\\PC{0,20}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Greek omicron ο (U+03BF) looks like Latin o
        let text = format!("{}prοcess{}", prefix, suffix);  // Greek ο

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", text).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            let has_greek = violations.iter().any(|v| {
                v.message.contains("Greek") ||
                v.message.contains("U+03BF") ||
                v.message.contains("omicron") ||
                v.message.contains("homoglyph")
            });

            prop_assert!(has_greek, "Should detect Greek omicron");
        }
    }
}

// Property: Mathematical alphanumeric variants are detected
proptest! {
    #[test]
    fn mathematical_variants_detected(text in "\\PC{0,30}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Mathematical italic 'a' (U+1D44E)
        let with_math = format!("{}𝑎dmin", text);

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", with_math).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        prop_assert!(result.is_ok(), "Scanner should not panic");

        let violations = result.unwrap();
        let has_math = violations.iter().any(|v| {
            let msg_lower = v.message.to_lowercase();
            let pattern_lower = v.pattern_name.to_lowercase();
            msg_lower.contains("mathematical") ||
            msg_lower.contains("homoglyph") ||
            pattern_lower.contains("mathematical") ||
            (v.code_point >= 0x1D400 && v.code_point <= 0x1D7FF)
        });

        prop_assert!(has_math, "Should detect mathematical variant (U+1D44E), found {} violations", violations.len());
    }
}

// Property: Fullwidth characters are detected
proptest! {
    #[test]
    fn fullwidth_chars_detected(text in "\\PC{0,30}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Fullwidth 'ａ' (U+FF41)
        let with_fullwidth = format!("{}ａdmin", text);

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", with_fullwidth).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        prop_assert!(result.is_ok(), "Scanner should not panic");

        let violations = result.unwrap();
        let has_fullwidth = violations.iter().any(|v| {
            let msg_lower = v.message.to_lowercase();
            let pattern_lower = v.pattern_name.to_lowercase();
            msg_lower.contains("fullwidth") ||
            msg_lower.contains("homoglyph") ||
            pattern_lower.contains("fullwidth") ||
            (v.code_point >= 0xFF01 && v.code_point <= 0xFF5E)
        });

        prop_assert!(has_fullwidth, "Should detect fullwidth character (U+FF41), found {} violations", violations.len());
    }
}

// Property: Mixed script identifiers trigger warnings
proptest! {
    #[test]
    fn mixed_scripts_detected(latin in "\\PC{1,10}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Mix Latin with Cyrillic
        let mixed = format!("{}аuthеnticate", latin);  // Cyrillic а and е

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", mixed).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            let has_detection = violations.iter().any(|v| {
                v.message.contains("Cyrillic") ||
                v.message.contains("homoglyph") ||
                v.message.contains("mixed") ||
                v.message.contains("confusable")
            });

            prop_assert!(
                has_detection,
                "Should detect mixed script usage"
            );
        }
    }
//
}

// Property: Homoglyph detection should not produce false positives on clean
// text
proptest! {
    #[test]
    fn no_false_positives_on_ascii(text in "[a-zA-Z0-9 ]{1,100}") {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", text).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::file_scanner::scan_file(temp.path());

        if let Ok(violations) = result {
            let homoglyph_violations: Vec<_> = violations.iter()
                .filter(|v| {
                    v.message.contains("homoglyph") ||
                    v.message.contains("Cyrillic") ||
                    v.message.contains("Greek") ||
                    v.message.contains("mathematical") ||
                    v.message.contains("fullwidth")
                })
                .collect();

            prop_assert!(
                homoglyph_violations.is_empty(),
                "Pure ASCII text should not trigger homoglyph warnings"
            );
        }
    }
}
