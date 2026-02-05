//! Integration tests for named Unicode block configuration

use std::path::Path;
use unicleaner::config::parser::parse_config;

#[test]
fn test_allowed_blocks_basic_latin_accepts_ascii() {
    // T009: Config with allowed_blocks = ["Basic Latin"] should accept ASCII
    let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_blocks = ["Basic Latin"]
    "#;

    let config = parse_config(toml, Path::new("test.toml")).unwrap();
    assert_eq!(config.file_rules.len(), 1);

    let rule = &config.file_rules[0];
    // ASCII 'A' (U+0041) should be in allowed ranges
    assert!(
        rule.is_code_point_allowed(0x0041),
        "ASCII 'A' should be allowed by Basic Latin block"
    );
    // Greek alpha (U+03B1) should NOT be in allowed ranges
    assert!(
        !rule.is_code_point_allowed(0x03B1),
        "Greek alpha should not be allowed by Basic Latin block"
    );
}

#[test]
fn test_allowed_blocks_rejects_unknown_name() {
    // T010: Config with unrecognized block name should produce error with suggestions
    let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_blocks = ["Nonexistent Block"]
    "#;

    let result = parse_config(toml, Path::new("test.toml"));
    assert!(result.is_err(), "Should fail for unknown block name");

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Nonexistent Block"),
        "Error should contain the invalid name, got: {}",
        err_msg
    );
}

#[test]
fn test_multiple_allowed_blocks() {
    // T014: Config with multiple blocks accepts chars from both, rejects others
    let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_blocks = ["Basic Latin", "Hebrew"]
    "#;

    let config = parse_config(toml, Path::new("test.toml")).unwrap();
    let rule = &config.file_rules[0];

    // ASCII 'A' (U+0041) should be allowed
    assert!(
        rule.is_code_point_allowed(0x0041),
        "ASCII 'A' should be allowed"
    );
    // Hebrew Alef (U+05D0) should be allowed
    assert!(
        rule.is_code_point_allowed(0x05D0),
        "Hebrew Alef should be allowed"
    );
    // Cyrillic A (U+0410) should NOT be allowed
    assert!(
        !rule.is_code_point_allowed(0x0410),
        "Cyrillic A should not be allowed"
    );
}

#[test]
fn test_allowed_blocks_and_ranges_union() {
    // T015: Config with both allowed_blocks and allowed_ranges uses union semantics
    let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_blocks = ["Basic Latin"]
allowed_ranges = [[0x0400, 0x04FF]]
    "#;

    let config = parse_config(toml, Path::new("test.toml")).unwrap();
    let rule = &config.file_rules[0];

    // ASCII 'A' (U+0041) from named block should be allowed
    assert!(
        rule.is_code_point_allowed(0x0041),
        "ASCII 'A' from named block should be allowed"
    );
    // Cyrillic A (U+0410) from numeric range should be allowed
    assert!(
        rule.is_code_point_allowed(0x0410),
        "Cyrillic A from numeric range should be allowed"
    );
    // Greek alpha (U+03B1) from neither should NOT be allowed
    assert!(
        !rule.is_code_point_allowed(0x03B1),
        "Greek alpha should not be allowed (not in either)"
    );
}

#[test]
fn test_allowed_blocks_alias_ascii() {
    // T018: Config with alias "ascii" should work like "Basic Latin"
    let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_blocks = ["ascii"]
    "#;

    let config = parse_config(toml, Path::new("test.toml")).unwrap();
    let rule = &config.file_rules[0];

    // ASCII 'A' (U+0041) should be allowed
    assert!(
        rule.is_code_point_allowed(0x0041),
        "ASCII 'A' should be allowed by 'ascii' alias"
    );
    // Latin-1 'e-acute' (U+00E9) should NOT be allowed
    assert!(
        !rule.is_code_point_allowed(0x00E9),
        "Latin-1 char should not be allowed by 'ascii' alias"
    );
}

#[test]
fn test_backward_compat_allowed_ranges_only() {
    // T028: Existing configs with only allowed_ranges (no allowed_blocks) continue to work
    let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_ranges = [[0x0000, 0x007F], [0x0400, 0x04FF]]
    "#;

    let config = parse_config(toml, Path::new("test.toml")).unwrap();
    let rule = &config.file_rules[0];

    // ASCII 'A' (U+0041) should be allowed
    assert!(
        rule.is_code_point_allowed(0x0041),
        "ASCII 'A' should be allowed by numeric range"
    );
    // Cyrillic A (U+0410) should be allowed
    assert!(
        rule.is_code_point_allowed(0x0410),
        "Cyrillic A should be allowed by numeric range"
    );
    // Greek alpha (U+03B1) should NOT be allowed
    assert!(
        !rule.is_code_point_allowed(0x03B1),
        "Greek alpha should not be allowed"
    );
}
