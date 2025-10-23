//! Core file scanning logic

use crate::report::Violation;
use crate::scanner::encoding::{detect_and_decode, is_binary};
use crate::scanner::unicode_detector::detect_in_string;
use crate::Result;
use std::fs;
use std::path::Path;

/// Scan a single file for malicious Unicode
pub fn scan_file(path: &Path) -> Result<Vec<Violation>> {
    // Read file bytes
    let bytes = fs::read(path)?;

    // Skip binary files
    if is_binary(&bytes) {
        return Ok(Vec::new());
    }

    // Decode to UTF-8
    let content = detect_and_decode(&bytes)?;

    // Detect malicious Unicode
    let violations = detect_in_string(&content, path);

    Ok(violations)
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
