// Performance regression tests (T066)
// Ensures performance doesn't degrade over time

#[cfg(test)]
mod performance_tests {
    use std::io::Write;
    use std::time::{Duration, Instant};
    use tempfile::NamedTempFile;
    use unicleaner::scanner::file_scanner::scan_file;

    // Performance baseline: Small file should scan in < 100ms
    #[test]
    fn test_small_file_performance() {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        // Create a small file (~1KB)
        for _ in 0..50 {
            writeln!(temp, "fn test() {{ let x = 42; }}").expect("Failed to write");
        }
        temp.flush().expect("Failed to flush");

        let start = Instant::now();
        let result = scan_file(temp.path());
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should scan successfully");
        assert!(
            duration < Duration::from_millis(100),
            "Small file should scan in < 100ms, took {:?}",
            duration
        );
    }

    // Performance baseline: Medium file should scan in < 1 second
    #[test]
    fn test_medium_file_performance() {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        // Create a medium file (~50KB)
        for _ in 0..2500 {
            writeln!(temp, "fn test() {{ let x = 42; }}").expect("Failed to write");
        }
        temp.flush().expect("Failed to flush");

        let start = Instant::now();
        let result = scan_file(temp.path());
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should scan successfully");
        assert!(
            duration < Duration::from_secs(1),
            "Medium file should scan in < 1s, took {:?}",
            duration
        );
    }

    // Performance test: Scanner should handle Unicode-heavy files efficiently
    #[test]
    fn test_unicode_heavy_performance() {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");

        // Create file with lots of Unicode
        for _ in 0..1000 {
            writeln!(temp, "测试中文字符🌍🚀✨").expect("Failed to write");
        }
        temp.flush().expect("Failed to flush");

        let start = Instant::now();
        let result = scan_file(temp.path());
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should scan successfully");
        assert!(
            duration < Duration::from_secs(2),
            "Unicode-heavy file should scan in < 2s, took {:?}",
            duration
        );
    }
}
