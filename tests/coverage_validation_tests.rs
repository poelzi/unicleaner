// Additional coverage tests for config/validation module
use unicleaner::config::Configuration;
use unicleaner::config::rules::FileRule;
use unicleaner::config::validation::validate_config;

/// Test that a rule with range end > 0x10FFFF is rejected
#[test]
fn test_validate_range_exceeds_max() {
    let mut config = Configuration::new();
    let rule = FileRule::new("*.rs")
        .unwrap()
        .with_allowed_range(0x0000, 0x110000, None); // end > 0x10FFFF
    config.file_rules.push(rule);

    let result = validate_config(&config);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("exceeds maximum") || err_msg.contains("0x10FFFF"),
        "Error should mention exceeding maximum Unicode code point, got: {}",
        err_msg
    );
}
