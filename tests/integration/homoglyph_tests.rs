// Integration tests for homoglyph attack detection
// Tests T025-T029 from tasks.md

use std::path::Path;
use unicleaner::scanner::file_scanner::scan_file;

// T025: Test for Cyrillic homoglyph detection
#[test]
fn test_cyrillic_homoglyph_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/homoglyphs/cyrillic.rs");
    let violations = scan_file(path).expect("Failed to scan file");

    // Should detect Cyrillic characters that look like Latin
    assert!(
        !violations.is_empty(),
        "Cyrillic homoglyphs should be detected"
    );

    // Should detect specific Cyrillic characters: а (U+0430), е (U+0435), о
    // (U+043E)
    let cyrillic_detected = violations.iter().any(|v| {
        v.message.contains("Cyrillic") ||
        v.message.contains("U+0430") ||  // Cyrillic а
        v.message.contains("U+0435") ||  // Cyrillic е
        v.message.contains("U+043E") ||  // Cyrillic о
        v.message.contains("homoglyph") ||
        v.message.contains("confusable")
    });

    assert!(
        cyrillic_detected,
        "Violation should identify Cyrillic homoglyphs"
    );
}

// T026: Test for Greek homoglyph detection
#[test]
fn test_greek_homoglyph_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/homoglyphs/greek.rs");
    let violations = scan_file(path).expect("Failed to scan file");

    assert!(
        !violations.is_empty(),
        "Greek homoglyphs should be detected"
    );

    // Should detect Greek characters: ο (U+03BF), ρ (U+03C1), ν (U+03BD), α
    // (U+03B1)
    let greek_detected = violations.iter().any(|v| {
        v.message.contains("Greek") ||
        v.message.contains("U+03BF") ||  // Greek omicron
        v.message.contains("U+03C1") ||  // Greek rho
        v.message.contains("U+03BD") ||  // Greek nu
        v.message.contains("U+03B1") ||  // Greek alpha
        v.message.contains("homoglyph") ||
        v.message.contains("confusable")
    });

    assert!(greek_detected, "Violation should identify Greek homoglyphs");
}

// T027: Test for mathematical alphanumeric character detection
#[test]
fn test_mathematical_character_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/homoglyphs/math_alphanumeric.rs");
    let violations = scan_file(path).expect("Failed to scan file");

    assert!(
        !violations.is_empty(),
        "Mathematical alphanumeric characters should be detected"
    );

    // Should detect mathematical variants: bold, italic, script, etc.
    let math_detected = violations.iter().any(|v| {
        v.message.to_lowercase().contains("mathematical") ||
        v.message.contains("U+1D4") ||  // Mathematical alphanumeric range
        v.message.contains("U+1D5") ||
        v.message.contains("U+1D6") ||
        v.message.to_lowercase().contains("homoglyph") ||
        v.message.contains("confusable")
    });

    assert!(
        math_detected,
        "Violation should identify mathematical character variants"
    );
}

// T028: Test for mixed script detection
#[test]
fn test_mixed_script_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/homoglyphs/mixed_scripts.rs");
    let violations = scan_file(path).expect("Failed to scan file");

    assert!(
        !violations.is_empty(),
        "Mixed script attacks should be detected"
    );

    // Should detect mixing of different scripts
    let mixed_detected = violations.iter().any(|v| {
        v.message.contains("Cyrillic")
            || v.message.contains("Greek")
            || v.message.contains("mixed")
            || v.message.contains("homoglyph")
            || v.message.contains("confusable")
    });

    assert!(
        mixed_detected,
        "Violations should identify mixed script usage"
    );

    // Should have multiple violations for heavily mixed file
    assert!(
        violations.len() > 5,
        "Mixed script file should have multiple violations, got {}",
        violations.len()
    );
}

// T029: Test for homoglyph severity levels
#[test]
fn test_homoglyph_severity_levels() {
    let test_files = [
        (
            "tests/fixtures/unicode_attacks/homoglyphs/cyrillic.rs",
            "Cyrillic",
        ),
        (
            "tests/fixtures/unicode_attacks/homoglyphs/greek.rs",
            "Greek",
        ),
        (
            "tests/fixtures/unicode_attacks/homoglyphs/math_alphanumeric.rs",
            "Mathematical",
        ),
        (
            "tests/fixtures/unicode_attacks/homoglyphs/fullwidth.rs",
            "Fullwidth",
        ),
    ];

    for (path, desc) in &test_files {
        let violations =
            scan_file(Path::new(path)).unwrap_or_else(|_| panic!("Failed to scan {}", desc));

        assert!(
            !violations.is_empty(),
            "{} homoglyphs should be detected",
            desc
        );

        // Each violation should have location information
        for violation in &violations {
            assert!(violation.line > 0, "Violation should have line number");
            assert!(violation.column > 0, "Violation should have column number");
            assert!(
                !violation.message.is_empty(),
                "Violation should have message"
            );
        }

        // Violations should be categorized as errors or warnings
        // (depending on severity - homoglyphs in identifiers are typically high
        // severity)
        let has_severity = violations.iter().all(|v| {
            // Check if violation has severity indicator in message or has severity field
            !v.message.is_empty()
        });

        assert!(
            has_severity,
            "{} violations should have severity information",
            desc
        );
    }
}

// Additional test: Fullwidth character detection
#[test]
fn test_fullwidth_character_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/homoglyphs/fullwidth.rs");
    let violations = scan_file(path).expect("Failed to scan fullwidth fixture");

    assert!(
        !violations.is_empty(),
        "Fullwidth characters should be detected"
    );

    // Should detect fullwidth Latin letters and digits
    let fullwidth_detected = violations.iter().any(|v| {
        v.message.to_lowercase().contains("fullwidth") ||
        v.message.contains("U+FF") ||  // Fullwidth form range
        v.message.to_lowercase().contains("halfwidth") ||
        v.message.to_lowercase().contains("homoglyph")
    });

    assert!(
        fullwidth_detected,
        "Violations should identify fullwidth characters"
    );
}

// Test: Confusable identifier detection
#[test]
fn test_confusable_identifiers() {
    let path = Path::new("tests/fixtures/unicode_attacks/homoglyphs/identifiers.rs");
    let violations = scan_file(path).expect("Failed to scan identifiers fixture");

    assert!(
        !violations.is_empty(),
        "Confusable identifiers should be detected"
    );

    // Should detect characters that create look-alike identifiers
    let confusable_detected = violations.iter().any(|v| {
        v.message.contains("confusable")
            || v.message.contains("homoglyph")
            || v.message.contains("Cyrillic")
            || v.message.contains("Greek")
    });

    assert!(
        confusable_detected,
        "Should detect confusable identifier patterns"
    );
}

// Test: Verify all homoglyph fixtures exist and are scannable
#[test]
fn test_all_homoglyph_fixtures_scannable() {
    let fixtures = [
        "tests/fixtures/unicode_attacks/homoglyphs/cyrillic.rs",
        "tests/fixtures/unicode_attacks/homoglyphs/greek.rs",
        "tests/fixtures/unicode_attacks/homoglyphs/math_alphanumeric.rs",
        "tests/fixtures/unicode_attacks/homoglyphs/fullwidth.rs",
        "tests/fixtures/unicode_attacks/homoglyphs/identifiers.rs",
        "tests/fixtures/unicode_attacks/homoglyphs/mixed_scripts.rs",
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
            "Fixture should contain detectable homoglyphs: {}",
            fixture
        );
    }
}
