//! Core file scanning logic

use crate::Result;
use crate::config::Configuration;
use crate::report::Violation;
use crate::scanner::encoding::{detect_decode_with_encoding, is_binary};
use crate::scanner::unicode_detector::detect_in_string;
use std::fs;
use std::path::Path;

/// Scan a single file for malicious Unicode with configuration
pub fn scan_file_with_config(path: &Path, config: &Configuration) -> Result<Vec<Violation>> {
    // Read file bytes
    let bytes = fs::read(path)?;

    // Skip binary files
    if is_binary(&bytes) {
        return Ok(Vec::new());
    }

    // Decode to UTF-8 and detect encoding
    let (content, encoding) = detect_decode_with_encoding(&bytes)?;

    // Detect malicious Unicode with config
    let mut violations = detect_in_string(&content, path);

    // Apply configuration rules to filter violations
    violations = apply_config_rules(violations, path, config);

    // Add encoding information to all violations
    for violation in &mut violations {
        *violation = violation.clone().with_encoding(encoding);
    }

    Ok(violations)
}

/// Scan a single file for malicious Unicode (without config - uses defaults)
pub fn scan_file(path: &Path) -> Result<Vec<Violation>> {
    scan_file_with_config(path, &Configuration::default())
}

/// Apply configuration rules to filter violations
fn apply_config_rules(
    violations: Vec<Violation>,
    path: &Path,
    config: &Configuration,
) -> Vec<Violation> {
    // Find matching file rules for this path
    let matching_rules: Vec<_> = config
        .file_rules
        .iter()
        .filter(|rule| rule.matcher.is_match(path))
        .collect();

    // If deny-by-default and no matching allow rules, keep all violations
    if config.deny_by_default {
        // Keep violations that are NOT explicitly allowed by file rules
        violations
            .into_iter()
            .filter(|v| {
                // Check if any matching rule allows this character
                !matching_rules.iter().any(|rule| {
                    rule.allowed_ranges
                        .iter()
                        .any(|range| range.contains(v.code_point))
                })
            })
            .collect()
    } else {
        // Allow-by-default mode: only report violations explicitly denied
        violations
            .into_iter()
            .filter(|v| {
                // Check if any matching rule explicitly denies this character
                matching_rules
                    .iter()
                    .any(|rule| rule.denied_code_points.contains(&v.code_point))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_scan_file_returns_violations() {
        // This test will fail until we implement scanning
        let path = PathBuf::from("tests/integration/fixtures/zero_width/test1.rs");
        if path.exists() {
            let violations = scan_file(&path).unwrap();
            // Should find zero-width space
            assert!(!violations.is_empty(), "Should detect malicious Unicode");
            assert!(violations.iter().any(|v| v.code_point == 0x200B));
        }
    }

    #[test]
    fn test_scan_clean_file() {
        let path = PathBuf::from("tests/integration/fixtures/clean/simple.rs");
        if path.exists() {
            let violations = scan_file(&path).unwrap();
            assert!(
                violations.is_empty(),
                "Clean file should have no violations"
            );
        }
    }
}
