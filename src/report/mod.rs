//! Report module for violation reporting and output formatting

pub mod formatter;
pub mod json;
pub mod violation;

// Re-export main types
pub use violation::{ScanError, Violation};

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
    pub fn exit_code(&self) -> i32 {
        if self.passed() {
            0
        } else if !self.violations.is_empty() {
            1
        } else {
            2
        }
    }

    /// Total number of violations
    pub fn total_violations(&self) -> usize {
        self.violations.len()
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
