//! Contract tests for exit codes

use unicleaner::cli::exit_codes::*;

#[test]
fn test_exit_code_constants() {
    assert_eq!(SUCCESS, 0, "Success exit code must be 0");
    assert_eq!(VIOLATIONS_FOUND, 1, "Violations exit code must be 1");
    assert_eq!(ERROR, 2, "Error exit code must be 2");
    assert_eq!(PARTIAL_SUCCESS, 3, "Partial success exit code must be 3");
}

#[test]
fn test_exit_codes_are_unique() {
    let codes = vec![SUCCESS, VIOLATIONS_FOUND, ERROR, PARTIAL_SUCCESS];
    let unique: std::collections::HashSet<_> = codes.iter().collect();
    assert_eq!(codes.len(), unique.len(), "Exit codes must be unique");
}

#[test]
fn test_scan_result_exit_code() {
    use std::path::PathBuf;
    use std::time::Duration;
    use unicleaner::report::ScanResult;

    // Clean scan - should return 0
    let clean_result = ScanResult {
        violations: vec![],
        files_scanned: 10,
        files_clean: 10,
        files_with_violations: 0,
        errors: vec![],
        duration: Duration::from_secs(1),
        config_used: PathBuf::from("test.toml"),
    };
    assert_eq!(clean_result.exit_code(), SUCCESS);
    assert!(clean_result.passed());
}
