//! JSON output formatting

use crate::report::ScanResult;
use serde_json;

/// Format scan results as JSON
pub fn format_json(result: &ScanResult) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(result)
}

/// Format scan results as compact JSON (one line)
pub fn format_json_compact(result: &ScanResult) -> Result<String, serde_json::Error> {
    serde_json::to_string(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Violation;
    use crate::unicode::malicious::{MaliciousCategory, Severity};
    use std::path::PathBuf;
    use std::time::Duration;

    fn create_test_violation() -> Violation {
        Violation::new(
            PathBuf::from("test.rs"),
            10,
            5,
            0,
            0x200B,
            "zero-width-space".to_string(),
            MaliciousCategory::ZeroWidth,
            Severity::Error,
            "Zero-width space detected".to_string(),
        )
    }

    #[test]
    fn test_format_json() {
        let violation = create_test_violation();
        let result = ScanResult {
            violations: vec![violation],
            files_scanned: 10,
            files_clean: 9,
            files_with_violations: 1,
            errors: vec![],
            duration: Duration::from_secs(1),
            config_used: PathBuf::from("unicleaner.toml"),
        };

        let json = format_json(&result).unwrap();

        // Verify JSON contains expected fields
        assert!(json.contains("\"violations\""));
        assert!(json.contains("\"files_scanned\""));
        assert!(json.contains("\"test.rs\""));
        assert!(json.contains("\"code_point\""));
        assert!(json.contains("8203")); // 0x200B in decimal
    }

    #[test]
    fn test_format_json_compact() {
        let result = ScanResult {
            violations: vec![],
            files_scanned: 5,
            files_clean: 5,
            files_with_violations: 0,
            errors: vec![],
            duration: Duration::from_millis(500),
            config_used: PathBuf::from("unicleaner.toml"),
        };

        let json = format_json_compact(&result).unwrap();

        // Compact JSON should not have newlines
        assert!(!json.contains("\n"));
        assert!(json.contains("\"files_scanned\":5"));
    }

    #[test]
    fn test_json_deserialize() {
        let violation = create_test_violation();
        let result = ScanResult {
            violations: vec![violation],
            files_scanned: 1,
            files_clean: 0,
            files_with_violations: 1,
            errors: vec![],
            duration: Duration::from_secs(1),
            config_used: PathBuf::from("test.toml"),
        };

        // Serialize then deserialize
        let json = format_json(&result).unwrap();
        let parsed: ScanResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.files_scanned, 1);
        assert_eq!(parsed.violations.len(), 1);
        assert_eq!(parsed.violations[0].code_point, 0x200B);
    }

    #[test]
    fn test_json_with_errors() {
        use crate::report::violation::{ErrorType, ScanError};

        let error = ScanError::new(
            PathBuf::from("binary.dat"),
            ErrorType::EncodingError,
            "Invalid encoding".to_string(),
        );

        let result = ScanResult {
            violations: vec![],
            files_scanned: 2,
            files_clean: 1,
            files_with_violations: 0,
            errors: vec![error],
            duration: Duration::from_secs(1),
            config_used: PathBuf::from("unicleaner.toml"),
        };

        let json = format_json(&result).unwrap();

        assert!(json.contains("\"errors\""));
        assert!(json.contains("\"binary.dat\""));
        assert!(json.contains("\"EncodingError\""));
    }
}
