//! Violation and error reporting structures

use crate::unicode::malicious::{MaliciousCategory, MaliciousPattern, Severity};
use std::path::PathBuf;

/// Represents a detected malicious Unicode character
#[derive(Debug, Clone)]
pub struct Violation {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub code_point: u32,
    pub character: char,
    pub context: String,
    pub pattern_name: String,
    pub category: MaliciousCategory,
    pub severity: Severity,
    pub message: String,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        file_path: PathBuf,
        line: usize,
        column: usize,
        code_point: u32,
        pattern_name: String,
        category: MaliciousCategory,
        severity: Severity,
        message: String,
    ) -> Self {
        let character = char::from_u32(code_point).unwrap_or('\u{FFFD}');

        Self {
            file_path,
            line,
            column,
            code_point,
            character,
            context: String::new(),
            pattern_name,
            category,
            severity,
            message,
        }
    }

    /// Add context to the violation
    pub fn with_context(mut self, context: String) -> Self {
        self.context = context;
        self
    }

    /// Get Unicode code point in U+XXXX format
    pub fn code_point_string(&self) -> String {
        format!("U+{:04X}", self.code_point)
    }
}

/// Error type for files that couldn't be scanned
#[derive(Debug, Clone)]
pub struct ScanError {
    pub file_path: PathBuf,
    pub error_type: ErrorType,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    IoError,
    EncodingError,
    ParseError,
    PermissionDenied,
}

impl ScanError {
    pub fn new(file_path: PathBuf, error_type: ErrorType, message: String) -> Self {
        Self {
            file_path,
            error_type,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let violation = Violation::new(
            PathBuf::from("test.rs"),
            42,
            15,
            0x200B,
            "zero-width-space".to_string(),
            MaliciousCategory::ZeroWidth,
            Severity::Error,
            "Zero-width space detected".to_string(),
        );

        assert_eq!(violation.line, 42);
        assert_eq!(violation.column, 15);
        assert_eq!(violation.code_point, 0x200B);
        assert_eq!(violation.code_point_string(), "U+200B");
        assert_eq!(violation.severity, Severity::Error);
    }

    #[test]
    fn test_violation_with_context() {
        let violation = Violation::new(
            PathBuf::from("test.rs"),
            1,
            1,
            0x200B,
            "test".to_string(),
            MaliciousCategory::ZeroWidth,
            Severity::Error,
            "test".to_string(),
        )
        .with_context("let user​name".to_string());

        assert!(!violation.context.is_empty());
        assert!(violation.context.contains("user"));
    }

    #[test]
    fn test_scan_error_creation() {
        let error = ScanError::new(
            PathBuf::from("binary.dat"),
            ErrorType::ParseError,
            "Binary file detected".to_string(),
        );

        assert_eq!(error.error_type, ErrorType::ParseError);
        assert!(error.message.contains("Binary"));
    }
}
