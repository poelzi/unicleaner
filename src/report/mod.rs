//! Report module for violation reporting and output formatting

pub mod formatter;
pub mod json;
pub mod markdown;
pub mod violation;

// Re-export main types
pub use violation::{ScanError, Violation};

use crate::cli::args::OutputFormat;
use crate::unicode::malicious::Severity;
use std::path::PathBuf;
use std::time::Duration;

/// Aggregate result of scanning one or more files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanResult {
    pub violations: Vec<Violation>,
    pub files_scanned: usize,
    pub files_clean: usize,
    pub files_with_violations: usize,
    pub errors: Vec<ScanError>,
    pub duration: Duration,
    pub config_used: PathBuf,
}

impl ScanResult {
    /// Check if the scan passed (no violations or errors)
    pub fn passed(&self) -> bool {
        self.violations.is_empty() && self.errors.is_empty()
    }

    /// Get appropriate exit code
    /// 0 = success (no violations, no errors)
    /// 1 = violations found (regardless of errors)
    /// 3 = errors only (no violations, but some files couldn't be scanned)
    pub fn exit_code(&self) -> i32 {
        if !self.violations.is_empty() {
            1
        } else if !self.errors.is_empty() {
            3
        } else {
            0
        }
    }

    /// Total number of violations
    pub fn total_violations(&self) -> usize {
        self.violations.len()
    }

    /// Format this scan result in the given output format.
    ///
    /// This is the central dispatch for all output formats; both `scan` and
    /// `format-report` funnel through here so the logic is in one place.
    pub fn format(
        &self,
        format: OutputFormat,
        verbose: bool,
        quiet: bool,
    ) -> Result<String, String> {
        match format {
            OutputFormat::Human => {
                let full = formatter::format_human(self, false, verbose);
                if quiet {
                    // Only show from "Scan Result:" onwards
                    if let Some(pos) = full
                        .find("\nScan Result:")
                        .or_else(|| full.find("Scan Result:"))
                    {
                        Ok(full[pos..].to_string())
                    } else {
                        Ok(full)
                    }
                } else {
                    Ok(full)
                }
            }
            OutputFormat::Json => {
                let json = if quiet {
                    json::format_json_compact(self)
                } else {
                    json::format_json(self)
                };
                json.map_err(|e| format!("Error formatting JSON: {}", e))
            }
            OutputFormat::Markdown => {
                let full = markdown::format_markdown(self, verbose);
                if quiet {
                    if let Some(pos) = full.find("## Summary") {
                        Ok(full[pos..].to_string())
                    } else {
                        Ok(full)
                    }
                } else {
                    Ok(full)
                }
            }
            OutputFormat::Github => {
                let mut out = String::new();
                for v in &self.violations {
                    out.push_str(&format!(
                        "::error file={},line={},col={}::{}\n",
                        v.file_path.display(),
                        v.line,
                        v.column,
                        v.message,
                    ));
                }
                Ok(out)
            }
            OutputFormat::Gitlab => {
                // GitLab CI uses JSON with specific schema; for now use standard JSON
                json::format_json(self)
                    .map_err(|e| format!("Error formatting GitLab output: {}", e))
            }
        }
    }

    /// Filter violations by minimum severity level
    /// Returns a new ScanResult with only violations >= min_severity
    pub fn filter_by_severity(mut self, min_severity: Severity) -> Self {
        // Filter violations
        self.violations.retain(|v| v.severity >= min_severity);

        // Recalculate statistics
        let files_with_violations = self
            .violations
            .iter()
            .map(|v| &v.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let files_clean = self.files_scanned - files_with_violations - self.errors.len();

        self.files_with_violations = files_with_violations;
        self.files_clean = files_clean;

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::malicious::{MaliciousCategory, Severity};
    use std::path::PathBuf;

    fn create_test_violation(severity: Severity) -> Violation {
        Violation::new(
            PathBuf::from("test.rs"),
            1,
            1,
            0,
            0x200B,
            "test".to_string(),
            MaliciousCategory::ZeroWidth,
            severity,
            "Test violation".to_string(),
        )
    }

    fn create_test_scan_result() -> ScanResult {
        ScanResult {
            violations: Vec::new(),
            files_scanned: 0,
            files_clean: 0,
            files_with_violations: 0,
            errors: Vec::new(),
            duration: Duration::from_secs(0),
            config_used: PathBuf::from("test.toml"),
        }
    }

    #[test]
    fn test_scan_result_passed() {
        let result = create_test_scan_result();
        assert_eq!(result.files_scanned, 0);
        assert_eq!(result.files_clean, 0);
        assert_eq!(result.files_with_violations, 0);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_total_violations() {
        let mut result = create_test_scan_result();
        result
            .violations
            .push(create_test_violation(Severity::Error));
        result
            .violations
            .push(create_test_violation(Severity::Warning));

        assert_eq!(result.total_violations(), 2);
    }

    #[test]
    fn test_filter_by_severity() {
        let mut result = create_test_scan_result();
        result.files_scanned = 1;
        result
            .violations
            .push(create_test_violation(Severity::Error));
        result
            .violations
            .push(create_test_violation(Severity::Warning));
        result
            .violations
            .push(create_test_violation(Severity::Info));

        let filtered = result.filter_by_severity(Severity::Warning);
        assert_eq!(filtered.violations.len(), 2); // Error and Warning only
    }

    // T045: Exit code 3 for errors only
    #[test]
    fn test_exit_code_3_errors_only() {
        let mut result = create_test_scan_result();
        result.errors.push(ScanError::new(
            PathBuf::from("missing.rs"),
            crate::report::violation::ErrorType::IoError,
            "File not found".to_string(),
        ));
        assert_eq!(result.exit_code(), 3);
    }

    // T046: Exit code 1 for violations with errors
    #[test]
    fn test_exit_code_1_violations_with_errors() {
        let mut result = create_test_scan_result();
        result
            .violations
            .push(create_test_violation(Severity::Error));
        result.errors.push(ScanError::new(
            PathBuf::from("missing.rs"),
            crate::report::violation::ErrorType::IoError,
            "File not found".to_string(),
        ));
        assert_eq!(result.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_0_success() {
        let result = create_test_scan_result();
        assert_eq!(result.exit_code(), 0);
    }

    #[test]
    fn test_filter_by_severity_error_only() {
        let mut result = create_test_scan_result();
        result.files_scanned = 1;
        result
            .violations
            .push(create_test_violation(Severity::Error));
        result
            .violations
            .push(create_test_violation(Severity::Warning));

        let filtered = result.filter_by_severity(Severity::Error);
        assert_eq!(filtered.violations.len(), 1); // Error only
    }
}
