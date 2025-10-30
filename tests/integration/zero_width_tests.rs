// Integration tests for zero-width and invisible character detection
// Tests T036-T040 from tasks.md

use std::path::Path;
use unicleaner::scanner::file_scanner::scan_file;

// T036: Test for ZWSP (Zero-Width Space) detection
#[test]
fn test_zwsp_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/zero_width/zwsp.rs");
    let violations = scan_file(path).expect("Failed to scan ZWSP fixture");

    assert!(
        !violations.is_empty(),
        "Zero-Width Space (ZWSP) should be detected"
    );

    // Should detect U+200B
    let zwsp_detected = violations.iter().any(|v| {
        v.message.to_lowercase().contains("zero-width space")
            || v.message.contains("ZWSP")
            || v.message.contains("U+200B")
            || v.message.contains("zero-width")
            || v.message.contains("invisible")
    });

    assert!(zwsp_detected, "Violations should identify ZWSP characters");
}

// T037: Test for ZWNJ/ZWJ detection
#[test]
fn test_zwnj_zwj_detection() {
    let test_files = [
        (
            "tests/fixtures/unicode_attacks/zero_width/zwnj.rs",
            "ZWNJ",
            "U+200C",
        ),
        (
            "tests/fixtures/unicode_attacks/zero_width/zwj.rs",
            "ZWJ",
            "U+200D",
        ),
    ];

    for (path, name, codepoint) in &test_files {
        let violations = scan_file(Path::new(path))
            .unwrap_or_else(|_| panic!("Failed to scan {} fixture", name));

        assert!(!violations.is_empty(), "{} should be detected", name);

        let detected = violations.iter().any(|v| {
            v.message.contains(name)
                || v.message.contains(codepoint)
                || v.message.to_lowercase().contains("zero-width")
                || v.message.to_lowercase().contains("invisible")
        });

        assert!(
            detected,
            "Violations should identify {} characters ({})",
            name, codepoint
        );
    }
}

// T038: Test for BOM in middle of file
#[test]
fn test_bom_in_middle_of_file() {
    let path = Path::new("tests/fixtures/unicode_attacks/zero_width/bom.rs");
    let violations = scan_file(path).expect("Failed to scan BOM fixture");

    assert!(
        !violations.is_empty(),
        "BOM/ZWNBSP in middle of file should be detected"
    );

    // Should detect U+FEFF when not at start of file
    let bom_detected = violations.iter().any(|v| {
        v.message.contains("BOM")
            || v.message.contains("U+FEFF")
            || v.message
                .to_lowercase()
                .contains("zero-width no-break space")
            || v.message.contains("ZWNBSP")
    });

    assert!(
        bom_detected,
        "Violations should identify BOM/ZWNBSP in middle of file"
    );

    // Violations should not be at line 1, column 1 (which would be legitimate BOM)
    let middle_of_file = violations.iter().any(|v| !(v.line == 1 && v.column == 1));

    assert!(middle_of_file, "Should detect BOMs not at file start");
}

// T039: Test for combining character stacking
#[test]
fn test_combining_character_stacking() {
    let path = Path::new("tests/fixtures/unicode_attacks/zero_width/combining.rs");
    let violations = scan_file(path).expect("Failed to scan combining char fixture");

    assert!(
        !violations.is_empty(),
        "Excessive combining characters should be detected"
    );

    // Should detect combining diacritical marks
    let combining_detected = violations.iter().any(|v| {
        v.message.to_lowercase().contains("combining") ||
        v.message.contains("diacritical") ||
        v.message.contains("U+03") ||  // Combining marks range
        v.message.contains("excessive") ||
        v.message.contains("stacked") ||
        v.message.contains("stacking")
    });

    assert!(
        combining_detected,
        "Violations should identify combining character abuse"
    );

    // Should have multiple violations for heavily stacked characters
    assert!(
        violations.len() > 3,
        "File with many combining marks should have multiple violations"
    );
}

// T040: Test for zero-width characters in identifiers
#[test]
fn test_zero_width_in_identifiers() {
    let fixtures = [
        "tests/fixtures/unicode_attacks/zero_width/zwsp.rs",
        "tests/fixtures/unicode_attacks/zero_width/zwnj.rs",
        "tests/fixtures/unicode_attacks/zero_width/zwj.rs",
        "tests/fixtures/unicode_attacks/zero_width/bom.rs",
    ];

    for fixture in &fixtures {
        let path = Path::new(fixture);
        let violations = scan_file(path).unwrap_or_else(|_| panic!("Failed to scan {}", fixture));

        assert!(
            !violations.is_empty(),
            "Zero-width characters in identifiers should be detected in {}",
            fixture
        );

        // Each violation should have precise location
        for violation in &violations {
            assert!(violation.line > 0, "Violation should have line number");
            assert!(violation.column > 0, "Violation should have column number");
            assert!(
                !violation.message.is_empty(),
                "Violation should have message"
            );

            // Message should identify the invisible character
            let msg_lower = violation.message.to_lowercase();
            let has_char_info = violation.message.contains("U+")
                || msg_lower.contains("zero-width")
                || msg_lower.contains("invisible")
                || violation.message.contains("BOM")
                || msg_lower.contains("combining");

            assert!(
                has_char_info,
                "Violation message should identify the invisible character type. Got message: \
                 '{}' for pattern: {} in {}",
                violation.message, violation.pattern_name, fixture
            );
        }
    }
}

// Additional test: Invisible separator detection
#[test]
fn test_invisible_separator_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/zero_width/separators.rs");
    let violations = scan_file(path).expect("Failed to scan separators fixture");

    assert!(
        !violations.is_empty(),
        "Invisible separator characters should be detected"
    );

    // Should detect various invisible separators
    let separator_detected = violations.iter().any(|v| {
        v.message.to_lowercase().contains("separator") ||
        v.message.contains("U+2028") ||  // Line separator
        v.message.contains("U+2029") ||  // Paragraph separator
        v.message.contains("U+00A0") ||  // Non-breaking space
        v.message.contains("U+3000") ||  // Ideographic space
        v.message.to_lowercase().contains("invisible") ||
        v.message.contains("whitespace")
    });

    assert!(
        separator_detected,
        "Violations should identify invisible separator characters"
    );
}

// Test: Verify all zero-width fixtures exist and are scannable
#[test]
fn test_all_zero_width_fixtures_scannable() {
    let fixtures = [
        "tests/fixtures/unicode_attacks/zero_width/zwsp.rs",
        "tests/fixtures/unicode_attacks/zero_width/zwnj.rs",
        "tests/fixtures/unicode_attacks/zero_width/zwj.rs",
        "tests/fixtures/unicode_attacks/zero_width/bom.rs",
        "tests/fixtures/unicode_attacks/zero_width/combining.rs",
        "tests/fixtures/unicode_attacks/zero_width/separators.rs",
    ];

    for fixture in &fixtures {
        let path = Path::new(fixture);
        assert!(path.exists(), "Fixture should exist: {}", fixture);

        let result = scan_file(path);
        assert!(
            result.is_ok(),
            "Should be able to scan fixture: {}",
            fixture
        );

        let violations = result.unwrap();
        assert!(
            !violations.is_empty(),
            "Fixture should contain detectable invisible characters: {}",
            fixture
        );
    }
}

// Test: No false positives on clean code
#[test]
fn test_no_false_positives_on_clean_code() {
    let clean_file = Path::new("tests/integration/fixtures/clean/simple.rs");
    if clean_file.exists() {
        let violations = scan_file(clean_file).expect("Failed to scan clean file");

        // Clean file should not have zero-width violations
        let zero_width_violations: Vec<_> = violations
            .iter()
            .filter(|v| {
                v.message.contains("zero-width")
                    || v.message.contains("ZWSP")
                    || v.message.contains("ZWNJ")
                    || v.message.contains("ZWJ")
                    || v.message.contains("U+200B")
                    || v.message.contains("U+200C")
                    || v.message.contains("U+200D")
            })
            .collect();

        assert!(
            zero_width_violations.is_empty(),
            "Clean file should not have zero-width violations"
        );
    }
}
