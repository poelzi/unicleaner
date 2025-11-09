// Property-based tests for scanner determinism (T047)
// Ensures same input always produces same output

use proptest::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

// Property: Scanning the same file twice should give identical results
proptest! {
    #[test]
    fn same_file_same_results(s in "\\PC{0,1000}") {
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", s).unwrap();
        temp.flush().unwrap();

        let result1 = unicleaner::scanner::file_scanner::scan_file(temp.path());
        let result2 = unicleaner::scanner::file_scanner::scan_file(temp.path());

        match (result1, result2) {
            (Ok(v1), Ok(v2)) => {
                prop_assert_eq!(v1.len(), v2.len(), "Same file should produce same number of violations");

                // Check that violations match
                for (violation1, violation2) in v1.iter().zip(v2.iter()) {
                    prop_assert_eq!(violation1.line, violation2.line, "Line numbers should match");
                    prop_assert_eq!(violation1.column, violation2.column, "Column numbers should match");
                    prop_assert_eq!(&violation1.message, &violation2.message, "Messages should match");
                }
            },
            (Err(_), Err(_)) => {
                // Both failed - that's deterministic too
            },
            _ => {
                prop_assert!(false, "Scanner produced different result types on same input");
            }
        }
    }
}

// Property: Files with identical content should produce identical results
proptest! {
    #[test]
    fn identical_content_identical_results(s in "\\PC{0,500}") {
        // Create two separate files with same content
        let mut temp1 = NamedTempFile::new().unwrap();
        let mut temp2 = NamedTempFile::new().unwrap();

        write!(temp1, "{}", s).unwrap();
        write!(temp2, "{}", s).unwrap();
        temp1.flush().unwrap();
        temp2.flush().unwrap();

        let result1 = unicleaner::scanner::file_scanner::scan_file(temp1.path());
        let result2 = unicleaner::scanner::file_scanner::scan_file(temp2.path());

        match (result1, result2) {
            (Ok(v1), Ok(v2)) => {
                prop_assert_eq!(v1.len(), v2.len(),
                    "Identical content should produce same number of violations");
            },
            (Err(_), Err(_)) => {},
            _ => {
                prop_assert!(false, "Identical content produced different results");
            }
        }
    }
}

// Property: Order of scanning shouldn't matter for results
proptest! {
    #[test]
    fn scan_order_independent(content in "\\PC{0,300}") {
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        // Scan multiple times
        let results: Vec<_> = (0..3)
            .map(|_| unicleaner::scanner::file_scanner::scan_file(temp.path()))
            .collect();

        // All results should be identical
        if let Some(Ok(first)) = results.first() {
            for violations in results[1..].iter().flatten() {
                prop_assert_eq!(first.len(), violations.len(),
                    "All scans should produce same number of violations");
            }
        }
    }
}
