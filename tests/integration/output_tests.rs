//! Integration tests for output formats and CLI flags

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test file with malicious Unicode
fn create_test_file_with_violation(dir: &TempDir) -> std::path::PathBuf {
    let file_path = dir.path().join("test.rs");
    // Contains a zero-width space (U+200B) which should be detected
    fs::write(
        &file_path,
        "fn main() {\u{200B}\n    println!(\"hello\");\n}\n",
    )
    .unwrap();
    file_path
}

/// Helper to create a clean test file
fn create_clean_test_file(dir: &TempDir) -> std::path::PathBuf {
    let file_path = dir.path().join("clean.rs");
    fs::write(&file_path, "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
    file_path
}

#[test]
fn test_json_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--format").arg("json").arg(&test_file);

    // Exit code 1 is expected when violations are found
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("violations"))
        .stdout(predicate::str::contains("files_scanned"))
        .stdout(predicate::str::contains("duration"));
}

#[test]
fn test_json_output_is_valid() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--format").arg("json").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify JSON is valid by parsing it
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        json_result.is_ok(),
        "Output should be valid JSON: {}",
        stdout
    );
}

#[test]
fn test_json_compact_with_quiet() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_clean_test_file(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan")
        .arg("--format")
        .arg("json")
        .arg("--quiet")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Compact JSON should be on a single line (no newlines except at end)
    let line_count = stdout.lines().count();
    assert!(
        line_count <= 2,
        "Compact JSON should be on one line, got {} lines",
        line_count
    );
}

#[test]
fn test_color_flag_never() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--color").arg("never").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // With --color=never, output should not contain ANSI escape codes
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[test]
fn test_color_flag_always() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--color").arg("always").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // With --color=always, output should contain ANSI escape codes for colors
    assert!(
        stdout.contains("\x1b["),
        "Output should contain ANSI color codes"
    );
}

#[test]
fn test_no_color_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--no-color").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // With --no-color, output should not contain ANSI escape codes
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes with --no-color"
    );
}

#[test]
fn test_no_color_env_variable() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan")
        .arg("--color")
        .arg("auto")
        .arg(&test_file)
        .env("NO_COLOR", "1");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // With NO_COLOR=1 and --color=auto, should not have colors
    assert!(
        !stdout.contains("\x1b["),
        "Output should respect NO_COLOR environment variable"
    );
}

#[test]
fn test_github_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan")
        .arg("--format")
        .arg("github")
        .arg(&test_file);

    // Exit code 1 is expected when violations are found
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("::error file="));
}

#[test]
fn test_gitlab_output_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan")
        .arg("--format")
        .arg("gitlab")
        .arg(&test_file);

    // GitLab format uses JSON, exit code 1 is expected when violations are found
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("violations"));
}

#[test]
fn test_quiet_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_clean_test_file(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--quiet").arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan Result:"));
}

#[test]
fn test_verbose_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--verbose").arg(&test_file);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Verbose mode should show progress messages on stderr
    assert!(stderr.contains("Collecting files") || stderr.contains("Scanning files"));
}

#[test]
fn test_human_output_default() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_clean_test_file(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scan Result:"))
        .stdout(predicate::str::contains("Files scanned:"));
}

#[test]
fn test_json_schema_contract() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--format").arg("json").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Parse JSON and verify schema contract
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Verify top-level fields exist and have correct types
    assert!(
        json.get("violations").is_some(),
        "Should have 'violations' field"
    );
    assert!(
        json["violations"].is_array(),
        "'violations' should be an array"
    );

    assert!(
        json.get("files_scanned").is_some(),
        "Should have 'files_scanned' field"
    );
    assert!(
        json["files_scanned"].is_number(),
        "'files_scanned' should be a number"
    );

    assert!(
        json.get("files_clean").is_some(),
        "Should have 'files_clean' field"
    );
    assert!(
        json["files_clean"].is_number(),
        "'files_clean' should be a number"
    );

    assert!(
        json.get("files_with_violations").is_some(),
        "Should have 'files_with_violations' field"
    );
    assert!(
        json["files_with_violations"].is_number(),
        "'files_with_violations' should be a number"
    );

    assert!(json.get("errors").is_some(), "Should have 'errors' field");
    assert!(json["errors"].is_array(), "'errors' should be an array");

    assert!(
        json.get("duration").is_some(),
        "Should have 'duration' field"
    );
    assert!(
        json["duration"].is_object(),
        "'duration' should be an object"
    );

    assert!(
        json.get("config_used").is_some(),
        "Should have 'config_used' field"
    );
    assert!(
        json["config_used"].is_string(),
        "'config_used' should be a string"
    );

    // Verify violation schema if violations exist
    if let Some(violations) = json["violations"].as_array() {
        if !violations.is_empty() {
            let violation = &violations[0];

            // Required fields in each violation
            assert!(
                violation.get("file_path").is_some(),
                "Violation should have 'file_path'"
            );
            assert!(
                violation["file_path"].is_string(),
                "'file_path' should be a string"
            );

            assert!(
                violation.get("line").is_some(),
                "Violation should have 'line'"
            );
            assert!(violation["line"].is_number(), "'line' should be a number");

            assert!(
                violation.get("column").is_some(),
                "Violation should have 'column'"
            );
            assert!(
                violation["column"].is_number(),
                "'column' should be a number"
            );

            assert!(
                violation.get("code_point").is_some(),
                "Violation should have 'code_point'"
            );
            assert!(
                violation["code_point"].is_number(),
                "'code_point' should be a number"
            );

            assert!(
                violation.get("character").is_some(),
                "Violation should have 'character'"
            );
            assert!(
                violation["character"].is_string(),
                "'character' should be a string"
            );

            assert!(
                violation.get("pattern_name").is_some(),
                "Violation should have 'pattern_name'"
            );
            assert!(
                violation["pattern_name"].is_string(),
                "'pattern_name' should be a string"
            );

            assert!(
                violation.get("category").is_some(),
                "Violation should have 'category'"
            );
            assert!(
                violation["category"].is_string(),
                "'category' should be a string"
            );

            assert!(
                violation.get("severity").is_some(),
                "Violation should have 'severity'"
            );
            assert!(
                violation["severity"].is_string(),
                "'severity' should be a string"
            );

            assert!(
                violation.get("message").is_some(),
                "Violation should have 'message'"
            );
            assert!(
                violation["message"].is_string(),
                "'message' should be a string"
            );

            assert!(
                violation.get("encoding").is_some(),
                "Violation should have 'encoding'"
            );
            assert!(
                violation["encoding"].is_string(),
                "'encoding' should be a string"
            );
        }
    }

    // Verify duration schema
    let duration = &json["duration"];
    assert!(
        duration.get("secs").is_some(),
        "Duration should have 'secs'"
    );
    assert!(duration["secs"].is_number(), "'secs' should be a number");
    assert!(
        duration.get("nanos").is_some(),
        "Duration should have 'nanos'"
    );
    assert!(duration["nanos"].is_number(), "'nanos' should be a number");
}

#[test]
fn test_severity_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    // First, check that violations exist without filtering
    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan").arg("--format").arg("json").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let total_violations = json["violations"].as_array().unwrap().len();

    assert!(
        total_violations > 0,
        "Should have violations without filtering"
    );

    // Now filter by severity=error (should still show the zero-width space which is
    // an error)
    let mut cmd = Command::cargo_bin("unicleaner").unwrap();
    cmd.arg("scan")
        .arg("--format")
        .arg("json")
        .arg("--severity")
        .arg("error")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify that violations still exist (zero-width space is an error)
    let filtered_violations = json["violations"].as_array().unwrap().len();
    assert!(
        filtered_violations > 0,
        "Should have error-level violations"
    );
}
