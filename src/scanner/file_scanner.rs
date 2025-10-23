//! Core file scanning logic

use crate::report::Violation;
use crate::Result;
use std::path::Path;

/// Scan a single file for malicious Unicode
pub fn scan_file(path: &Path) -> Result<Vec<Violation>> {
    // TODO: Implement file scanning
    Ok(Vec::new())
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
            // Should find zero-width space - this will fail initially
            assert!(!violations.is_empty(), "Should detect malicious Unicode");
        }
    }
}
