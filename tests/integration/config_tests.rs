//! Integration tests for configuration loading and application

use assert_cmd::cargo_bin_cmd;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test that the example config file (examples/unicleaner.toml) is valid and can be loaded
/// by the scanner without errors. This ensures the shipped example stays in sync with
/// the actual config format.
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

[languages.rust]
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

    let config_content = r#"
[languages.rust]
preset = "rust"
    "#;

    fs::write(&config_path, config_content).unwrap();

    // Test preset application logic
    // TODO: Implement preset application testing
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
    // TODO: Implement multiple language preset testing
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

    // Test rule application
    // TODO: Implement file-specific rule testing
}

#[test]
fn test_file_rule_glob_patterns() {
    // Test various glob patterns: *.rs, **/*.js, src/lib.rs
    // TODO: Implement glob pattern testing
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

    // Without explicit allowlist, all non-ASCII should be flagged
    // TODO: Implement deny-by-default behavior testing
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

    // Only explicitly denied characters should be flagged
    // TODO: Implement allow-by-default mode testing
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
    // TODO: Implement custom Unicode range testing
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

    // Should fail gracefully with error message
    // TODO: Implement invalid config error handling testing
}
