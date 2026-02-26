//! Integration tests for file scanning

use std::path::Path;
use unicleaner::scanner::file_scanner::scan_file;

#[test]
fn test_scan_zero_width_fixtures() {
    let fixture_path = Path::new("tests/integration/fixtures/zero_width/test1.rs");
    assert!(
        fixture_path.exists(),
        "Test fixture not found: {:?}",
        fixture_path
    );

    let violations = scan_file(fixture_path).unwrap();
    assert!(
        !violations.is_empty(),
        "Should detect zero-width space in fixture"
    );

    // Verify at least one zero-width character was found
    assert!(
        violations.iter().any(|v| v.code_point == 0x200B),
        "Should detect U+200B zero-width space"
    );
}

#[test]
fn test_scan_bidi_fixtures() {
    let fixture_path = Path::new("tests/integration/fixtures/bidi/trojan_source.rs");
    assert!(fixture_path.exists(), "Bidi test fixture should exist");

    let violations = scan_file(fixture_path).unwrap();
    assert!(
        !violations.is_empty(),
        "Should detect bidi override characters in fixture"
    );

    // Verify bidi control characters were detected (U+202E RLO, U+2066 LRI, U+2069
    // PDI)
    let has_bidi = violations
        .iter()
        .any(|v| matches!(v.code_point, 0x202A..=0x202E | 0x2066..=0x2069));
    assert!(has_bidi, "Should detect bidirectional override characters");
}

#[test]
fn test_scan_homoglyph_fixtures() {
    let fixture_path = Path::new("tests/integration/fixtures/homoglyphs/cyrillic_a.py");
    assert!(fixture_path.exists(), "Homoglyph test fixture should exist");

    let violations = scan_file(fixture_path).unwrap();
    assert!(
        !violations.is_empty(),
        "Should detect homoglyph characters in fixture"
    );
}

#[test]
fn test_scan_clean_files() {
    let fixture_path = Path::new("tests/integration/fixtures/clean/simple.rs");
    assert!(fixture_path.exists(), "Clean test fixture should exist");

    let violations = scan_file(fixture_path).unwrap();
    assert!(
        violations.is_empty(),
        "Clean file should have no violations, found: {:?}",
        violations
            .iter()
            .map(|v| format!("U+{:04X} at {}:{}", v.code_point, v.line, v.column))
            .collect::<Vec<_>>()
    );
}
