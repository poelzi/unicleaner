//! Additional integration tests for output formats and CLI flags - coverage boost
//! This is a standalone integration test binary.

use assert_cmd::cargo_bin_cmd;
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

/// Minimal valid JSON scan result for format-report tests
fn sample_scan_json() -> &'static str {
    r#"{"violations":[{"file_path":"test.rs","line":1,"column":5,"byte_offset":4,"code_point":8203,"character":"\u200b","context":"fn m\u200bain","pattern_name":"zero-width-space","category":"ZeroWidth","severity":"Error","message":"Zero-width space","encoding":"Utf8"}],"files_scanned":1,"files_clean":0,"files_with_violations":1,"errors":[],"duration":{"secs":0,"nanos":42000000},"config_used":"unicleaner.toml"}"#
}

/// Write the sample JSON to a file in the given temp dir and return its path
fn write_sample_json(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("report.json");
    fs::write(&p, sample_scan_json()).unwrap();
    p
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("init").current_dir(temp_dir.path());

    cmd.assert().code(0);

    assert!(
        temp_dir.path().join("unicleaner.toml").exists(),
        "init should create unicleaner.toml"
    );
}

#[test]
fn test_init_command_custom_path() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("init")
        .arg("custom.toml")
        .current_dir(temp_dir.path());

    cmd.assert().code(0);

    assert!(
        temp_dir.path().join("custom.toml").exists(),
        "init should create custom.toml"
    );
}

#[test]
fn test_init_command_no_overwrite() {
    let temp_dir = TempDir::new().unwrap();

    // First init - should succeed
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("init").current_dir(temp_dir.path());
    cmd.assert().code(0);

    // Second init without --force - should fail
    let mut cmd2 = cargo_bin_cmd!("unicleaner");
    cmd2.arg("init").current_dir(temp_dir.path());
    cmd2.assert()
        .code(1)
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_command_force() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("init").current_dir(temp_dir.path());
    cmd.assert().code(0);

    // Second init with --force - should succeed
    let mut cmd2 = cargo_bin_cmd!("unicleaner");
    cmd2.arg("init").arg("--force").current_dir(temp_dir.path());
    cmd2.assert().code(0);
}

#[test]
fn test_list_presets() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("list-presets");

    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("rust"))
        .stdout(predicate::str::contains("javascript"));
}

#[test]
fn test_list_blocks_no_filter() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("list-blocks");

    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("Basic Latin"));
}

#[test]
fn test_list_blocks_with_filter() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("list-blocks").arg("latin");

    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("Latin"));
}

#[test]
fn test_list_blocks_no_match() {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("list-blocks").arg("zzzzzzzzz");

    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("No Unicode blocks"));
}

#[test]
fn test_format_report_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let json_path = write_sample_json(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("format-report")
        .arg("-f")
        .arg("markdown")
        .arg(&json_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("# Unicleaner Scan:"))
        .stdout(predicate::str::contains("## Summary"));
}

#[test]
fn test_format_report_from_stdin() {
    use std::io::Write;
    use std::process::Stdio;

    let bin_path = assert_cmd::cargo::cargo_bin!("unicleaner");
    let mut child = std::process::Command::new(bin_path)
        .arg("format-report")
        .arg("-f")
        .arg("markdown")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(sample_scan_json().as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "Should exit with 1 (has violations)"
    );
    assert!(
        stdout.contains("# Unicleaner Scan:"),
        "Should contain markdown header"
    );
    assert!(stdout.contains("## Summary"), "Should contain summary");
}

#[test]
fn test_format_report_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let bad_path = temp_dir.path().join("bad.json");
    fs::write(&bad_path, "this is not json").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("format-report")
        .arg("-f")
        .arg("markdown")
        .arg(&bad_path);

    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("Failed to parse JSON"));
}

#[test]
fn test_format_report_to_human() {
    let temp_dir = TempDir::new().unwrap();
    let json_path = write_sample_json(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("format-report")
        .arg("-f")
        .arg("human")
        .arg(&json_path);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("Scan Result"));
}

#[test]
fn test_scan_output_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);
    let report_path = temp_dir.path().join("report.md");

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--format")
        .arg("json")
        .arg("-o")
        .arg(format!("markdown:{}", report_path.display()))
        .arg(&test_file);

    // JSON on stdout, markdown written to file
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("violations"));

    let md_content = fs::read_to_string(&report_path).unwrap();
    assert!(
        md_content.contains("# Unicleaner Scan:"),
        "Markdown report should contain header"
    );
    assert!(
        md_content.contains("## Summary"),
        "Markdown report should contain summary"
    );
}

#[test]
fn test_scan_output_multiple() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);
    let md_path = temp_dir.path().join("report.md");
    let txt_path = temp_dir.path().join("report.txt");

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("-f")
        .arg("json")
        .arg("-o")
        .arg(format!("markdown:{}", md_path.display()))
        .arg("-o")
        .arg(format!("human:{}", txt_path.display()))
        .arg(&test_file);

    cmd.assert().code(1);

    assert!(md_path.exists(), "Markdown report file should exist");
    assert!(txt_path.exists(), "Human report file should exist");

    let md_content = fs::read_to_string(&md_path).unwrap();
    assert!(md_content.contains("## Summary"));

    let txt_content = fs::read_to_string(&txt_path).unwrap();
    assert!(txt_content.contains("Scan Result"));
}

#[test]
fn test_scan_severity_filter() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("-f")
        .arg("json")
        .arg("--severity")
        .arg("error")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // All reported violations should have severity "Error"
    if let Some(violations) = json["violations"].as_array() {
        for v in violations {
            assert_eq!(
                v["severity"].as_str().unwrap(),
                "Error",
                "Only Error severity violations should be present"
            );
        }
    }
}

#[test]
fn test_scan_github_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--format")
        .arg("github")
        .arg(&test_file);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("::error file="));
}

#[test]
fn test_scan_gitlab_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--format")
        .arg("gitlab")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // GitLab format should produce valid JSON
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        json_result.is_ok(),
        "GitLab output should be valid JSON, got: {}",
        stdout
    );
}

#[test]
fn test_scan_markdown_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--format")
        .arg("markdown")
        .arg(&test_file);

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("# Unicleaner Scan:"));
}

#[test]
fn test_scan_quiet_json() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("-f")
        .arg("json")
        .arg("--quiet")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Compact JSON should be on a single line
    let line_count = stdout.trim().lines().count();
    assert!(
        line_count <= 1,
        "Quiet JSON should be a single line, got {} lines",
        line_count
    );
}

#[test]
fn test_scan_quiet_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("-f")
        .arg("markdown")
        .arg("--quiet")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("## Summary"),
        "Quiet markdown should contain summary"
    );
    assert!(
        !stdout.contains("# Unicleaner Scan:"),
        "Quiet markdown should NOT contain the full header"
    );
}

#[test]
fn test_scan_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("-f")
        .arg("markdown")
        .arg("--verbose")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("`U+200B`"),
        "Verbose markdown should contain code point info, got:\n{}",
        stdout
    );
}

#[test]
fn test_scan_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file_with_violation(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--config")
        .arg("nonexistent.toml")
        .arg(&test_file);

    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Error")));
}

#[test]
fn test_scan_clean_file() {
    let temp_dir = TempDir::new().unwrap();
    let clean_file = create_clean_test_file(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan").arg("-f").arg("json").arg(&clean_file);

    let output = cmd.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "Clean file should exit with 0"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 0, "Clean file should have 0 violations");
}

#[test]
fn test_format_report_json_to_json() {
    let temp_dir = TempDir::new().unwrap();
    let json_path = write_sample_json(&temp_dir);

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("format-report")
        .arg("-f")
        .arg("json")
        .arg(&json_path);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Output should be valid JSON (roundtrip)
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "format-report JSON roundtrip should produce valid JSON: {}\nOutput: {}",
            e, stdout
        )
    });
    assert!(
        json["violations"].is_array(),
        "Roundtripped JSON should have violations array"
    );
    assert_eq!(
        json["files_scanned"].as_u64().unwrap(),
        1,
        "files_scanned should be preserved"
    );
}
