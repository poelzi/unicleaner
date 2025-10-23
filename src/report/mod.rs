//! Report module for violation reporting and output formatting

pub mod formatter;
pub mod json;
pub mod violation;

// Re-export main types
pub use violation::{ScanError, Violation};

use std::path::PathBuf;
use std::time::Duration;

/// Aggregate result of scanning one or more files
#[derive(Debug, Clone)]
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
}
