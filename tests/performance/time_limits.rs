// Performance tests for time limits (T086)
// Ensures scanner meets speed requirements

#[cfg(test)]
mod time_limit_tests {
    use std::fs;
    use std::io::Write;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use unicleaner::scanner::file_scanner::scan_file;

    // Test: 1000-file repo should scan in < 5 seconds (per plan.md requirement)
    #[test]
    fn test_1000_file_repo_under_5_seconds() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create 1000 files
        for i in 0..1000 {
            let file_path = temp_dir.path().join(format!("file_{}.rs", i));
            let mut file = fs::File::create(&file_path).expect("Failed to create file");

            writeln!(file, "pub fn test_{}() {{", i).expect("Failed to write");
            writeln!(file, "    let x = {};", i).expect("Failed to write");
            writeln!(file, "    println!(\"Value: {{}}\", x);").expect("Failed to write");
            writeln!(file, "}}").expect("Failed to write");
        }

        let files: Vec<_> = fs::read_dir(temp_dir.path())
            .expect("Failed to read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        let start = Instant::now();

        for file in &files {
            let _ = scan_file(file);
        }

        let duration = start.elapsed();

        assert!(
            duration < Duration::from_secs(5),
            "1000-file repo should scan in < 5 seconds per requirements, took {:?}",
            duration
        );

        println!("✓ Scanned 1000 files in {:?}", duration);
    }

    // Test: Single file should scan in < 10ms
    #[test]
    fn test_single_file_under_10ms() {
        let mut temp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp, "fn test() {{ let x = 42; }}").expect("Failed to write");
        temp.flush().expect("Failed to flush");

        let start = Instant::now();
        let _ = scan_file(temp.path());
        let duration = start.elapsed();

        assert!(
            duration < Duration::from_millis(10),
            "Single small file should scan in < 10ms, took {:?}",
            duration
        );
    }

    // Test: 100 files should scan in < 500ms
    #[test]
    fn test_100_files_under_500ms() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        for i in 0..100 {
            let file_path = temp_dir.path().join(format!("file_{}.rs", i));
            let mut file = fs::File::create(&file_path).expect("Failed to create file");
            writeln!(file, "fn test_{}() {{ }}", i).expect("Failed to write");
        }

        let files: Vec<_> = fs::read_dir(temp_dir.path())
            .expect("Failed to read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        let start = Instant::now();
        for file in &files {
            let _ = scan_file(file);
        }
        let duration = start.elapsed();

        assert!(
            duration < Duration::from_millis(500),
            "100 files should scan in < 500ms, took {:?}",
            duration
        );
    }
}
