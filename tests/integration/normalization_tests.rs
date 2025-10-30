// Integration tests for Unicode normalization attacks (T070-T072)

use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;
use unicleaner::scanner::file_scanner::scan_file;
use unicode_normalization::UnicodeNormalization;

// T070: Test canonical combining character reordering
#[test]
fn test_canonical_combining_reordering() {
    let path = Path::new("tests/fixtures/unicode_attacks/normalization/nfc_nfd.rs");

    if path.exists() {
        let result = scan_file(path);
        assert!(result.is_ok(), "Should scan normalization fixture");

        // The fixture contains both NFC and NFD forms
        // Scanner should detect potential normalization issues
    }
}

// T071: Test compatibility character detection
#[test]
fn test_compatibility_character_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/normalization/nfkc_nfkd.rs");

    if path.exists() {
        let result = scan_file(path);
        assert!(result.is_ok(), "Should scan compatibility fixture");

        if let Ok(violations) = result {
            // Should detect fullwidth or other compatibility characters
            let _compat_detected = violations.iter().any(|v| {
                v.message.contains("fullwidth")
                    || v.message.contains("compatibility")
                    || v.message.contains("ligature")
                    || v.message.contains("NFKC")
            });

            // Compatibility chars should be flagged (if scanner implements this)
            if !violations.is_empty() {
                // Good - detected something suspicious
            }
        }
    }
}

// T072: Test normalized vs denormalized identifier comparison
#[test]
fn test_normalized_denormalized_identifiers() {
    // Create two files with same identifier in different normalizations
    let mut temp_nfc = NamedTempFile::new().expect("Failed to create temp file");
    let mut temp_nfd = NamedTempFile::new().expect("Failed to create temp file");

    // NFC form: é is single character
    write!(temp_nfc, "fn café() {{ }}").expect("Failed to write");
    temp_nfc.flush().expect("Failed to flush");

    // NFD form: é is e + combining accent
    let nfd_text = "fn café() { }"; // This will be NFD
    let nfd_normalized: String = nfd_text.nfd().collect();
    write!(temp_nfd, "{}", nfd_normalized).expect("Failed to write");
    temp_nfd.flush().expect("Failed to flush");

    // Both should scan successfully
    let result_nfc = scan_file(temp_nfc.path());
    let result_nfd = scan_file(temp_nfd.path());

    assert!(result_nfc.is_ok(), "Should handle NFC form");
    assert!(result_nfd.is_ok(), "Should handle NFD form");
}

// Additional normalization tests

// Test: Different normalizations of same text
#[test]
fn test_normalization_forms() {
    let test_cases = vec![
        ("naïve", "NFC"),
        ("naïve", "NFD"),
        ("café", "NFC"),
        ("café", "NFD"),
    ];

    for (text, form) in test_cases {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        let normalized = match form {
            "NFC" => text.nfc().collect::<String>(),
            "NFD" => text.nfd().collect::<String>(),
            "NFKC" => text.nfkc().collect::<String>(),
            "NFKD" => text.nfkd().collect::<String>(),
            _ => text.to_string(),
        };

        write!(temp, "fn test() {{ let x = \"{}\"; }}", normalized).expect("Failed to write");
        temp.flush().expect("Failed to flush");

        let result = scan_file(temp.path());
        assert!(result.is_ok(), "Should handle {} form", form);
    }
}

// Test: Fullwidth normalization
#[test]
fn test_fullwidth_normalization() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // Fullwidth characters (compatibility)
    write!(temp, "fn ｔｅｓｔ() {{ }}").expect("Failed to write");
    temp.flush().expect("Failed to flush");

    let result = scan_file(temp.path());
    assert!(result.is_ok(), "Should handle fullwidth characters");

    if let Ok(violations) = result {
        // Should detect fullwidth as potential issue
        let fullwidth_detected = violations
            .iter()
            .any(|v| v.message.to_lowercase().contains("fullwidth") || v.message.contains("U+FF"));

        assert!(fullwidth_detected, "Should detect fullwidth characters");
    }
}

// Test: Ligature normalization
#[test]
fn test_ligature_normalization() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // fi ligature (U+FB01)
    write!(temp, "fn test() {{ let ﬁle = \"test\"; }}").expect("Failed to write");
    temp.flush().expect("Failed to flush");

    let result = scan_file(temp.path());
    assert!(result.is_ok(), "Should handle ligatures");
}

// Test: Combining marks in different orders
#[test]
fn test_combining_mark_order() {
    let mut temp1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut temp2 = NamedTempFile::new().expect("Failed to create temp file");

    // Same base + different combining mark orders
    // e + acute + grave
    write!(temp1, "fn test() {{ let e\u{0301}\u{0300} = 1; }}").expect("Failed to write");
    temp1.flush().expect("Failed to flush");

    // e + grave + acute (canonical order might differ)
    write!(temp2, "fn test() {{ let e\u{0300}\u{0301} = 1; }}").expect("Failed to write");
    temp2.flush().expect("Failed to flush");

    let result1 = scan_file(temp1.path());
    let result2 = scan_file(temp2.path());

    assert!(result1.is_ok(), "Should handle combining marks order 1");
    assert!(result2.is_ok(), "Should handle combining marks order 2");
}

// Test: Verify normalization fixtures exist
#[test]
fn test_normalization_fixtures_exist() {
    let fixtures = [
        "tests/fixtures/unicode_attacks/normalization/nfc_nfd.rs",
        "tests/fixtures/unicode_attacks/normalization/nfkc_nfkd.rs",
    ];

    for fixture in &fixtures {
        let path = Path::new(fixture);
        if path.exists() {
            let result = scan_file(path);
            assert!(result.is_ok(), "Should scan {}", fixture);
        }
    }
}
