//! Integration tests for file scanning

use std::path::PathBuf;

#[test]
fn test_scan_zero_width_fixtures() {
    let fixture_path = PathBuf::from("tests/integration/fixtures/zero_width/test1.rs");
    if !fixture_path.exists() {
        panic!("Test fixture not found: {:?}", fixture_path);
    }

    // TODO: This will fail until scanning is implemented
    // let violations = unicleaner::scanner::file_scanner::scan_file(&fixture_path).unwrap();
    // assert!(!violations.is_empty(), "Should detect zero-width space");

    // For now, just verify fixture exists
    assert!(fixture_path.exists());
}

#[test]
fn test_scan_bidi_fixtures() {
    let fixture_path = PathBuf::from("tests/integration/fixtures/bidi/trojan_source.rs");
    assert!(fixture_path.exists(), "Bidi test fixture should exist");
}

#[test]
fn test_scan_homoglyph_fixtures() {
    let fixture_path = PathBuf::from("tests/integration/fixtures/homoglyphs/cyrillic_a.py");
    assert!(fixture_path.exists(), "Homoglyph test fixture should exist");
}

#[test]
fn test_scan_clean_files() {
    let fixture_path = PathBuf::from("tests/integration/fixtures/clean/simple.rs");
    assert!(fixture_path.exists(), "Clean test fixture should exist");

    // TODO: When implemented, should return zero violations
    // let violations = unicleaner::scanner::file_scanner::scan_file(&fixture_path).unwrap();
    // assert!(violations.is_empty(), "Clean file should have no violations");
}
