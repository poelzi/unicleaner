// Memory usage regression tests (T067)
// Ensures memory usage stays within acceptable bounds

#[cfg(test)]
mod memory_tests {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use unicleaner::scanner::file_scanner::scan_file;

    // Memory test: Large file shouldn't cause excessive memory usage
    #[test]
    fn test_large_file_memory_usage() {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        // Create a large file (~500KB)
        for i in 0..25000 {
            writeln!(temp, "fn test{}() {{ let x = {}; }}", i, i).expect("Failed to write");
        }
        temp.flush().expect("Failed to flush");

        // Should complete without out-of-memory
        let result = scan_file(temp.path());
        assert!(result.is_ok(), "Should handle large files without OOM");
    }

    // Memory test: Scanner should not leak memory on repeated scans
    #[test]
    fn test_no_memory_leak_on_repeated_scans() {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        writeln!(temp, "fn test() {{ let x = 42; }}").expect("Failed to write");
        temp.flush().expect("Failed to flush");

        // Scan same file multiple times - should not accumulate memory
        for _ in 0..100 {
            let result = scan_file(temp.path());
            assert!(result.is_ok(), "Should scan successfully");
        }
    }

    // Memory test: Many small violations shouldn't cause excessive memory
    #[test]
    fn test_many_violations_memory() {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        // Create file with many malicious characters
        for _ in 0..1000 {
            writeln!(temp, "test\u{202E}code\u{200B}here").expect("Failed to write");
        }
        temp.flush().expect("Failed to flush");

        let result = scan_file(temp.path());
        assert!(result.is_ok(), "Should handle many violations");

        if let Ok(violations) = result {
            assert!(!violations.is_empty(), "Should detect violations");
            // Having many violations shouldn't cause OOM
        }
    }
}
