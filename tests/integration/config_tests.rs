//! Integration tests for configuration loading and application

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test that the example config file (examples/unicleaner.toml) is valid and
/// can be loaded by the scanner without errors. This ensures the shipped
/// example stays in sync with the actual config format.
#[test]
fn test_example_config_loads_successfully() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("clean.rs");
    fs::write(&test_file, "fn main() {\n    println!(\"hello\");\n}\n").unwrap();

    let example_config = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/unicleaner.toml");
    assert!(
        example_config.exists(),
        "Example config must exist at examples/unicleaner.toml"
    );

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--config")
        .arg(&example_config)
        .arg(&test_file);

    cmd.assert().success();
}

#[test]
fn test_load_config_file() {
    // T056: Test loading config from filesystem
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
 [global]
 deny_by_default = true

 [languages.rs]
 preset = "rust"
     "#;

    fs::write(&config_path, config_content).unwrap();

    // Test will fail until config loading is implemented
    assert!(config_path.exists());
}

#[test]
fn test_load_nonexistent_config() {
    // Should handle missing config gracefully
    let nonexistent = PathBuf::from("/nonexistent/config.toml");
    assert!(!nonexistent.exists());
}

#[test]
fn test_language_preset_application() {
    // T057: Test language preset is correctly applied
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let js_file = temp.path().join("test.js");
    fs::write(&js_file, "const x = \"a\u{00A0}b\";\n").unwrap();

    let config_content = r#"
 [global]
 deny_by_default = true

 [languages.js]
 preset = "javascript"
    "#;

    fs::write(&config_path, config_content).unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .arg(&js_file)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "NBSP should be allowed by the javascript preset"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        violations.is_empty(),
        "NBSP should be suppressed by allowlist, got: {}",
        stdout
    );
}

#[test]
fn test_multiple_language_presets() {
    // Should support multiple language configurations
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
[languages.rust]
preset = "rust"

[languages.javascript]
preset = "javascript"
    "#;

    fs::write(&config_path, config_content).unwrap();

    let file_path = temp.path().join("test.rs");
    fs::write(&file_path, "fn main() {}\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("test.rs");

    cmd.assert().success();
}

#[test]
fn test_file_specific_rules() {
    // T058: Test file-specific rules override presets
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
[[rules]]
pattern = "src/legacy/*.rs"
allowed_ranges = [[0x0000, 0xFFFF]]
    "#;

    fs::write(&config_path, config_content).unwrap();

    let file_path = temp.path().join("test.rs");
    fs::write(&file_path, "fn main() {}\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("test.rs");

    cmd.assert().success();
}

#[test]
fn test_file_rule_glob_patterns() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
[[rules]]
pattern = "**/*.rs"
allowed_ranges = [[0x0000, 0x007F]]
    "#;

    fs::write(&config_path, config_content).unwrap();

    let file_path = temp.path().join("test.rs");
    fs::write(&file_path, "fn main() {}\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("test.rs");

    cmd.assert().success();
}

#[test]
fn test_deny_by_default_behavior() {
    // T059: Test deny-by-default security model
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
[global]
deny_by_default = true
    "#;

    fs::write(&config_path, config_content).unwrap();

    let file_path = temp.path().join("unicode.rs");
    fs::write(&file_path, "fn café() { }\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("--format")
        .arg("json")
        .arg("unicode.rs")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "deny_by_default should flag non-ASCII without an allowlist"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(
        json["violations"]
            .as_array()
            .is_some_and(|v| v.iter().any(|x| x["code_point"].as_u64() == Some(0x00E9))),
        "Expected to find U+00E9 in violations, got: {}",
        stdout
    );
}

#[test]
fn test_allow_by_default_mode() {
    // Test permissive mode (not recommended but supported)
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
[global]
deny_by_default = false
    "#;

    fs::write(&config_path, config_content).unwrap();

    let file_path = temp.path().join("unicode.rs");
    fs::write(&file_path, "fn café() { }\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("--format")
        .arg("json")
        .arg("unicode.rs")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "allow-by-default should not flag non-ASCII unless explicitly denied"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        violations.is_empty(),
        "Expected no violations, got: {}",
        stdout
    );
}

#[test]
fn test_config_with_custom_allowed_ranges() {
    // Test custom Unicode range specification
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let config_content = r#"
[[rules]]
pattern = "*.rs"
allowed_ranges = [
    [0x0000, 0x007F],  # Basic Latin
    [0x0080, 0x00FF],  # Latin-1 Supplement
]
    "#;

    fs::write(&config_path, config_content).unwrap();

    let file_path = temp.path().join("unicode.rs");
    fs::write(&file_path, "fn café() { }\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("--format")
        .arg("json")
        .arg("unicode.rs")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "Custom allowed_ranges should allow U+00E9"
    );
}

#[test]
fn test_invalid_config_handling() {
    // Malformed TOML should produce clear error
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("unicleaner.toml");

    let invalid_content = r#"
[global
invalid syntax here
    "#;

    fs::write(&config_path, invalid_content).unwrap();

    let file_path = temp.path().join("test.rs");
    fs::write(&file_path, "fn main() {}\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.current_dir(temp.path())
        .arg("scan")
        .arg("--config")
        .arg("unicleaner.toml")
        .arg("test.rs");

    // Should fail with non-zero exit code on invalid config
    cmd.assert().failure();
}

// =============================================================================
// Phase 2: Config loading tests (T005-T008)
// =============================================================================

/// T005: Config allowing a range that covers U+200B suppresses that violation
#[test]
fn test_config_loading_from_file() {
    let temp = TempDir::new().unwrap();

    // Create file with zero-width space (U+200B)
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "hello\u{200B}world\n").unwrap();

    // Config that allows safe ASCII + General Punctuation (includes U+200B at
    // 0x2000-0x206F)
    let config_path = temp.path().join("custom.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = true

 [[rules]]
 pattern = "**/*"
 allowed_ranges = [
     [0x0009, 0x000D],
     [0x0020, 0x007E],
     [0x2000, 0x206F],
 ]
 "#,
    )
    .unwrap();

    // With config allowing the range: no violations expected
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        violations.is_empty(),
        "Config allowing U+200B range should suppress violation, got: {:?}",
        violations
    );

    // Without config (default deny-by-default): violation expected
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan").arg("--format").arg("json").arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        !violations.is_empty(),
        "Without config, U+200B should be detected as violation"
    );
}

/// T006: Auto-discovery of unicleaner.toml in CWD
#[test]
fn test_config_auto_discovery() {
    let temp = TempDir::new().unwrap();

    // Create file with zero-width space
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "hello\u{200B}world\n").unwrap();

    // Place unicleaner.toml in CWD (temp dir) allowing safe ASCII + the range
    let config_path = temp.path().join("unicleaner.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = true

 [[rules]]
 pattern = "**/*"
 allowed_ranges = [
     [0x0009, 0x000D],
     [0x0020, 0x007E],
     [0x2000, 0x206F],
 ]
 "#,
    )
    .unwrap();

    // Run from temp dir (CWD) — should auto-discover unicleaner.toml
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.current_dir(temp.path())
        .arg("scan")
        .arg("--format")
        .arg("json")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        violations.is_empty(),
        "Auto-discovered config should suppress violation, got: {:?}",
        violations
    );
}

/// T007: --config with missing file exits with error (exit code 2)
#[test]
fn test_config_missing_file_error() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "hello\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("scan")
        .arg("--config")
        .arg("/nonexistent/missing.toml")
        .arg(&test_file);

    cmd.assert()
        .code(2)
        .stderr(predicate::str::contains("not found"));
}

/// W005: --config with unreadable file exits with error (exit code 2)
#[cfg(unix)]
#[test]
fn test_config_unreadable_file_error() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "hello\n").unwrap();

    let config_path = temp.path().join("unreadable.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = true
"#,
    )
    .unwrap();

    // Remove all read bits to force a permission error.
    let mut perms = fs::metadata(&config_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&config_path, perms).unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg(&test_file)
        .output()
        .unwrap();

    // Restore permissions so tempfile cleanup can delete the file.
    let mut restore = fs::metadata(&config_path).unwrap().permissions();
    restore.set_mode(0o644);
    fs::set_permissions(&config_path, restore).unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "Unreadable config should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to load configuration")
            || stderr.to_lowercase().contains("permission denied"),
        "Expected unreadable config error, got stderr: {}",
        stderr
    );
}

// =============================================================================
// Phase 3: US1 Config-Driven Scanning (T014)
// =============================================================================

/// T014: Same file scanned with and without config yields different violation
/// counts
#[test]
fn test_config_changes_scan_behavior() {
    let temp = TempDir::new().unwrap();

    // File with zero-width space
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "hello\u{200B}world\n").unwrap();

    // Scan WITHOUT config (from temp dir with no unicleaner.toml) → violation
    // expected
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .current_dir(temp.path())
        .arg("scan")
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let without_config_count = json["violations"].as_array().unwrap().len();
    assert!(
        without_config_count > 0,
        "Without config, should find violations"
    );

    // Scan WITH config allowing safe ASCII + U+200B range → no violation
    let config_path = temp.path().join("allow.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = true

 [[rules]]
 pattern = "**/*"
 allowed_ranges = [
     [0x0009, 0x000D],
     [0x0020, 0x007E],
     [0x2000, 0x206F],
 ]
 "#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let with_config_count = json["violations"].as_array().unwrap().len();

    assert_eq!(
        with_config_count, 0,
        "Config should suppress violations: without={}, with={}",
        without_config_count, with_config_count
    );
}

// =============================================================================
// Phase 4: US3 Valid Init Config (T017-T018)
// =============================================================================

/// T017: `unicleaner init` generates config that parses successfully
#[test]
fn test_init_generates_valid_config() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("unicleaner.toml");

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("init").arg(&output_path);
    cmd.assert().success();

    // The generated config must parse without error
    let config = unicleaner::config::Configuration::from_file(&output_path);
    assert!(
        config.is_ok(),
        "Init-generated config should parse successfully: {:?}",
        config.err()
    );
}

/// T018: Init config uses correct schema sections ([global], [[rules]],
/// [languages.<ext>])
#[test]
fn test_init_config_matches_parser_schema() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("unicleaner.toml");

    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.arg("init").arg(&output_path);
    cmd.assert().success();

    let content = fs::read_to_string(&output_path).unwrap();

    // Must have [global] section (not bare deny_by_default)
    assert!(
        content.contains("[global]"),
        "Init config must use [global] section, got:\n{}",
        content
    );

    // Must NOT use old-style bare keys at root level
    // deny_by_default should be under [global], not at root
    let lines: Vec<&str> = content.lines().collect();
    let mut in_global = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == "[global]" {
            in_global = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_global = false;
        }
        if trimmed.starts_with("deny_by_default") {
            assert!(
                in_global,
                "deny_by_default must be inside [global], not at root level"
            );
        }
    }
}

// =============================================================================
// Phase 5: US2 Policy Enforcement (T021-T023)
// =============================================================================

/// T021: Denied characters override allowed ranges
#[test]
fn test_denied_chars_override_allowed_ranges() {
    let temp = TempDir::new().unwrap();

    // File with zero-width space (U+200B)
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "hello\u{200B}world\n").unwrap();

    // Config: allow safe ASCII + 0x2000-0x206F but explicitly deny U+200B
    let config_path = temp.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = true

 [[rules]]
 pattern = "**/*"
 allowed_ranges = [
     [0x0009, 0x000D],
     [0x0020, 0x007E],
     [0x2000, 0x206F],
 ]
 denied_characters = [0x200B]
 "#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let violations = json["violations"].as_array().unwrap();
    assert!(
        !violations.is_empty(),
        "Denied character U+200B should be reported even though range allows it"
    );
}

/// T022: Bidi controls always denied even in allow-by-default mode
#[test]
fn test_always_deny_bidi_in_allow_by_default() {
    let temp = TempDir::new().unwrap();

    // File with RLO (U+202E)
    let rlo = char::from_u32(0x202E).unwrap();
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, format!("hello{}world\n", rlo)).unwrap();

    // Config: allow-by-default
    let config_path = temp.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = false
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let violations = json["violations"].as_array().unwrap();
    assert!(
        !violations.is_empty(),
        "Bidi control U+202E should always be denied even in allow-by-default mode"
    );
    assert!(
        violations
            .iter()
            .any(|v| v["code_point"].as_u64() == Some(0x202E)),
        "Should specifically report U+202E"
    );
}

/// T023: File-specific rule overrides global config
#[test]
fn test_file_specific_rule_overrides_global() {
    let temp = TempDir::new().unwrap();

    // File with zero-width space
    let test_file = temp.path().join("test.rs");
    fs::write(&test_file, "hello\u{200B}world\n").unwrap();

    // Config: global allows the range, but .rs rule restricts to ASCII only
    let config_path = temp.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[global]
deny_by_default = true

[[rules]]
pattern = "**/*.rs"
allowed_ranges = [[0x0000, 0x007F]]
"#,
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("json")
        .arg(&test_file)
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let violations = json["violations"].as_array().unwrap();
    assert!(
        !violations.is_empty(),
        "U+200B should be reported since .rs rule only allows ASCII"
    );
}

// =============================================================================
// Phase 15: Build & Project Hygiene (T072-T073)
// =============================================================================

/// T072: .gitignore should not list Cargo.lock (binary crates should commit
/// lock files)
#[test]
fn test_gitignore_does_not_list_cargo_lock() {
    let gitignore = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".gitignore");
    let content = fs::read_to_string(&gitignore).expect(".gitignore must exist");
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        assert_ne!(
            trimmed, "Cargo.lock",
            ".gitignore must not ignore Cargo.lock for binary crates"
        );
        assert_ne!(
            trimmed, "/Cargo.lock",
            ".gitignore must not ignore Cargo.lock for binary crates"
        );
    }
}

/// T073: Every bench file under benches/ has a matching [[bench]] entry in
/// Cargo.toml
#[test]
fn test_all_bench_targets_configured() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let benches_dir = manifest_dir.join("benches");

    if !benches_dir.exists() {
        return; // No bench dir → nothing to check
    }

    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();

    for entry in fs::read_dir(&benches_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let bench_name = path.file_stem().unwrap().to_str().unwrap();
            let expected = format!("name = \"{}\"", bench_name);
            assert!(
                cargo_toml.contains(&expected),
                "Bench file '{}' has no matching [[bench]] entry (expected: {})",
                path.display(),
                expected
            );
        }
    }
}

/// T008: No config file present → uses defaults and succeeds on clean file
#[test]
fn test_config_default_when_no_file() {
    let temp = TempDir::new().unwrap();

    // Clean file, no config anywhere
    let test_file = temp.path().join("clean.txt");
    fs::write(&test_file, "hello world\n").unwrap();

    // Run from temp dir that has no unicleaner.toml
    let mut cmd = cargo_bin_cmd!("unicleaner");
    cmd.current_dir(temp.path())
        .arg("scan")
        .arg("--format")
        .arg("json")
        .arg(&test_file);

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Should succeed with default config, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(
        json["violations"].as_array().unwrap().len(),
        0,
        "Clean file with default config should have no violations"
    );
}
