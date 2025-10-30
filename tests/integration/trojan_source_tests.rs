// Integration tests for Trojan Source attack detection (CVE-2021-42574,
// CVE-2021-42694) Tests T015-T018 from tasks.md

use std::path::Path;
use test_case::test_case;
use unicleaner::scanner::file_scanner::scan_file;

// T015: Test for RLO (Right-to-Left Override) detection
#[test]
fn test_rlo_detection() {
    let path = Path::new("tests/fixtures/unicode_attacks/trojan_source/rlo_attack.rs");
    let violations = scan_file(path).expect("Failed to scan file");

    // Should detect RLO character (U+202E)
    assert!(!violations.is_empty(), "RLO attack should be detected");

    assert!(
        violations.iter().any(|v| v.message.contains("RLO")
            || v.message.contains("U+202E")
            || v.message.contains("Right-to-Left Override")),
        "Violation should mention RLO character"
    );
}

// T016: Test for LRI/RLI/FSI detection
#[test_case("tests/fixtures/unicode_attacks/trojan_source/lri_hiding.rs", "LRI" ; "LRI hiding attack")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/rli_reorder.rs", "RLI" ; "RLI reordering attack")]
#[test_case("tests/fixtures/unicode_attacks/trojan_source/fsi_attack.rs", "FSI" ; "FSI isolation attack")]
fn test_bidi_isolate_detection(path: &str, attack_type: &str) {
    let violations = scan_file(Path::new(path)).expect("Failed to scan file");

    assert!(
        !violations.is_empty(),
        "{} attack should be detected",
        attack_type
    );

    assert!(
        violations.iter().any(|v| v.message.contains(attack_type)),
        "Violation should mention {} character",
        attack_type
    );
}

// T017: Test for nested bidi overrides
#[test]
fn test_nested_bidi_overrides() {
    let path = Path::new("tests/fixtures/unicode_attacks/trojan_source/combined_bidi.rs");
    let violations = scan_file(path).expect("Failed to scan file");

    // Combined attacks should detect multiple bidi characters
    assert!(
        !violations.is_empty(),
        "Combined bidi attack should be detected"
    );

    // Should detect multiple different bidi control characters
    let bidi_types: Vec<&str> = violations
        .iter()
        .filter_map(|v| {
            if v.message.contains("RLO") {
                Some("RLO")
            } else if v.message.contains("LRI") {
                Some("LRI")
            } else if v.message.contains("RLI") {
                Some("RLI")
            } else if v.message.contains("PDI") {
                Some("PDI")
            } else {
                None
            }
        })
        .collect();

    assert!(
        !bidi_types.is_empty(),
        "Should detect at least one type of bidi character in combined attack"
    );
}

// T018: Test for bidi in comments vs code
#[test]
fn test_bidi_in_comments_vs_code() {
    // Test that bidi characters are detected regardless of whether they're in
    // comments or code
    let fixtures = [
        (
            "tests/fixtures/unicode_attacks/trojan_source/rlo_attack.rs",
            "code",
        ),
        (
            "tests/fixtures/unicode_attacks/trojan_source/lri_hiding.rs",
            "comment or code",
        ),
    ];

    for (path, context) in &fixtures {
        let violations = scan_file(Path::new(path)).expect("Failed to scan file");

        assert!(
            !violations.is_empty(),
            "Bidi characters in {} should be detected in {}",
            path,
            context
        );

        // Verify that violations include line and column information
        for violation in &violations {
            assert!(violation.line > 0, "Violation should have line number");
            assert!(violation.column > 0, "Violation should have column number");
        }
    }
}

// Additional comprehensive test for all Trojan Source fixtures
#[test]
fn test_all_trojan_source_fixtures_detected() {
    let fixtures = [
        "tests/fixtures/unicode_attacks/trojan_source/rlo_attack.rs",
        "tests/fixtures/unicode_attacks/trojan_source/lri_hiding.rs",
        "tests/fixtures/unicode_attacks/trojan_source/rli_reorder.rs",
        "tests/fixtures/unicode_attacks/trojan_source/fsi_attack.rs",
        "tests/fixtures/unicode_attacks/trojan_source/pdi_attack.rs",
        "tests/fixtures/unicode_attacks/trojan_source/combined_bidi.rs",
    ];

    for fixture in &fixtures {
        let path = Path::new(fixture);
        assert!(path.exists(), "Fixture should exist: {}", fixture);

        let violations = scan_file(path).expect("Failed to scan file");
        assert!(
            !violations.is_empty(),
            "Should detect malicious Unicode in {}",
            fixture
        );
    }
}

// Test that clean files don't trigger false positives
#[test]
fn test_no_false_positives_on_clean_code() {
    let clean_file = Path::new("tests/integration/fixtures/clean/simple.rs");
    if clean_file.exists() {
        let violations = scan_file(clean_file).expect("Failed to scan clean file");

        // Clean file should not have bidi-related violations
        let bidi_violations: Vec<_> = violations
            .iter()
            .filter(|v| {
                v.message.contains("RLO")
                    || v.message.contains("LRI")
                    || v.message.contains("RLI")
                    || v.message.contains("FSI")
                    || v.message.contains("PDI")
            })
            .collect();

        assert!(
            bidi_violations.is_empty(),
            "Clean file should not have bidi violations"
        );
    }
}
