// Additional coverage tests for report module - ScanResult::format()
use std::path::PathBuf;
use std::time::Duration;

use unicleaner::cli::args::OutputFormat;
use unicleaner::report::violation::{ErrorType, ScanError};
use unicleaner::report::{ScanResult, Violation};
use unicleaner::unicode::malicious::{MaliciousCategory, Severity};

fn create_test_violation(severity: Severity) -> Violation {
    Violation::new(
        PathBuf::from("test.rs"),
        10,
        5,
        42,
        0x200B,
        "zero-width-space".to_string(),
        MaliciousCategory::ZeroWidth,
        severity,
        "Detected zero-width space".to_string(),
    )
}

fn create_empty_scan_result() -> ScanResult {
    ScanResult {
        violations: Vec::new(),
        files_scanned: 5,
        files_clean: 5,
        files_with_violations: 0,
        errors: Vec::new(),
        duration: Duration::from_millis(100),
        config_used: PathBuf::from("test.toml"),
    }
}

fn create_scan_result_with_violation() -> ScanResult {
    ScanResult {
        violations: vec![create_test_violation(Severity::Error)],
        files_scanned: 5,
        files_clean: 4,
        files_with_violations: 1,
        errors: Vec::new(),
        duration: Duration::from_millis(100),
        config_used: PathBuf::from("test.toml"),
    }
}

// --- ScanResult::format() tests ---

#[test]
fn test_format_json() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Json, false, false).unwrap();
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_format_json_quiet() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Json, false, true).unwrap();
    // Compact JSON should not contain pretty-print newlines (it may have newlines inside strings)
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
    // Compact format typically has no indentation
    assert!(!output.contains("    "));
}

#[test]
fn test_format_markdown() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Markdown, false, false).unwrap();
    assert!(output.contains("# Unicleaner Scan:") || output.contains("# Unicleaner"));
}

#[test]
fn test_format_markdown_quiet() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Markdown, false, true).unwrap();
    assert!(output.starts_with("## Summary"));
}

#[test]
fn test_format_github() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Github, false, false).unwrap();
    assert!(output.contains("::error file="));
}

#[test]
fn test_format_github_no_violations() {
    let result = create_empty_scan_result();
    let output = result.format(OutputFormat::Github, false, false).unwrap();
    assert!(output.is_empty());
}

#[test]
fn test_format_gitlab() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Gitlab, false, false).unwrap();
    // GitLab format outputs JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_format_human() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Human, false, false).unwrap();
    assert!(output.contains("Scan Result:"));
}

#[test]
fn test_format_human_quiet() {
    let result = create_scan_result_with_violation();
    let output = result.format(OutputFormat::Human, false, true).unwrap();
    // quiet mode trims to "\nScan Result:" or "Scan Result:"
    assert!(output.contains("Scan Result:"));
    // The output should be shorter than the full output
    let full = result.format(OutputFormat::Human, false, false).unwrap();
    assert!(output.len() <= full.len());
}

#[test]
fn test_passed_with_errors_only() {
    let mut result = create_empty_scan_result();
    result.errors.push(ScanError::new(
        PathBuf::from("missing.rs"),
        ErrorType::IoError,
        "File not found".to_string(),
    ));
    assert!(!result.passed());
    assert_eq!(result.exit_code(), 3);
}
