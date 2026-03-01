// Additional coverage tests for report/formatter module
// Tests private functions indirectly through the public format_human API
use std::path::PathBuf;
use std::time::Duration;

use unicleaner::report::formatter::format_human;
use unicleaner::report::violation::{ErrorType, ScanError};
use unicleaner::report::{ScanResult, Violation};
use unicleaner::unicode::malicious::{MaliciousCategory, Severity};

fn create_test_violation() -> Violation {
    Violation::new(
        PathBuf::from("test.rs"),
        10,
        5,
        42,
        0x200B,
        "zero-width-space".to_string(),
        MaliciousCategory::ZeroWidth,
        Severity::Error,
        "Detected zero-width space".to_string(),
    )
}

/// Test that verbose mode with context shows "Context:" in output
#[test]
fn test_format_violations_verbose_with_context() {
    let violation =
        create_test_violation().with_context("let user\u{200B}name = \"test\"".to_string());
    let result = ScanResult {
        violations: vec![violation],
        files_scanned: 1,
        files_clean: 0,
        files_with_violations: 1,
        errors: Vec::new(),
        duration: Duration::from_secs(0),
        config_used: PathBuf::from("unicleaner.toml"),
    };
    let output = format_human(&result, false, true);
    assert!(
        output.contains("Context:"),
        "Verbose output with context should contain 'Context:'"
    );
}

/// Test that errors in ScanResult show "Errors:" in the summary
#[test]
fn test_format_summary_with_errors() {
    let result = ScanResult {
        violations: Vec::new(),
        files_scanned: 3,
        files_clean: 2,
        files_with_violations: 0,
        errors: vec![ScanError::new(
            PathBuf::from("bad.rs"),
            ErrorType::IoError,
            "permission denied".to_string(),
        )],
        duration: Duration::from_secs(1),
        config_used: PathBuf::from("unicleaner.toml"),
    };
    let output = format_human(&result, false, false);
    assert!(
        output.contains("Errors:"),
        "Summary with errors should contain 'Errors:'"
    );
}

/// Test format_severity with color enabled for all severity levels
/// (exercised through format_human with use_color=true)
#[test]
fn test_format_severity_with_color() {
    // Create violations at each severity level
    let error_violation = Violation::new(
        PathBuf::from("test.rs"),
        1,
        1,
        0,
        0x200B,
        "zws".to_string(),
        MaliciousCategory::ZeroWidth,
        Severity::Error,
        "error".to_string(),
    );
    let warning_violation = Violation::new(
        PathBuf::from("test.rs"),
        2,
        1,
        0,
        0x200C,
        "zwnj".to_string(),
        MaliciousCategory::ZeroWidth,
        Severity::Warning,
        "warning".to_string(),
    );
    let info_violation = Violation::new(
        PathBuf::from("test.rs"),
        3,
        1,
        0,
        0x200D,
        "zwj".to_string(),
        MaliciousCategory::ZeroWidth,
        Severity::Info,
        "info".to_string(),
    );

    let result = ScanResult {
        violations: vec![error_violation, warning_violation, info_violation],
        files_scanned: 1,
        files_clean: 0,
        files_with_violations: 1,
        errors: Vec::new(),
        duration: Duration::from_secs(0),
        config_used: PathBuf::from("unicleaner.toml"),
    };

    // Call with use_color=true to exercise the color formatting paths
    let output = format_human(&result, true, false);
    // With color, the output should still contain the file reference and messages
    assert!(output.contains("test.rs"));
    assert!(output.contains("Scan Result:"));
}
