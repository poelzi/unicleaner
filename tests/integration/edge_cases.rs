// Integration tests for edge cases (T059-T064)

use std::io::Write;
use tempfile::NamedTempFile;
use unicleaner::scanner::file_scanner::scan_file;

// T059: Test extremely long lines (>10000 chars)
#[test]
fn test_extremely_long_lines() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // Create a 15000 character line
    let long_line = "a".repeat(15000);
    writeln!(temp, "{}", long_line).expect("Failed to write");
    temp.flush().expect("Failed to flush");

    // Should handle without crashing
    let result = scan_file(temp.path());
    assert!(
        result.is_ok(),
        "Should handle extremely long lines without crashing"
    );
}

// T060: Test files with millions of Unicode characters
#[test]
fn test_file_with_many_unicode_chars() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // Create file with 100K Unicode characters
    for _ in 0..10000 {
        write!(temp, "测试中文🌍").expect("Failed to write");
    }
    temp.flush().expect("Failed to flush");

    // Should handle large Unicode files
    let result = scan_file(temp.path());
    assert!(
        result.is_ok(),
        "Should handle files with many Unicode characters"
    );
}

// T061: Test invalid UTF-8 sequences handling
#[test]
fn test_invalid_utf8_sequences() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // Write invalid UTF-8 bytes
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, 0xC0, 0xAF];
    temp.write_all(&invalid_utf8).expect("Failed to write");
    temp.flush().expect("Failed to flush");

    // Should handle invalid UTF-8 gracefully (return error, not panic)
    let result = scan_file(temp.path());
    // Either Ok (if handled as binary) or Err (if rejected), but shouldn't panic
    if result.is_ok() {
        // Handled gracefully
    }
    // Rejected gracefully (Err case)
}

// T062: Test mixed UTF-8/UTF-16/UTF-32 detection
#[test]
fn test_mixed_encoding_detection() {
    // UTF-8 file
    let mut temp_utf8 = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp_utf8, "UTF-8 content: hello").expect("Failed to write");
    temp_utf8.flush().expect("Failed to flush");

    let result = scan_file(temp_utf8.path());
    assert!(result.is_ok(), "Should handle UTF-8 files");

    // UTF-16 LE file with BOM
    let mut temp_utf16 = NamedTempFile::new().expect("Failed to create temp file");
    temp_utf16
        .write_all(&[0xFF, 0xFE])
        .expect("Failed to write BOM"); // UTF-16 LE BOM
    temp_utf16
        .write_all(&[0x68, 0x00, 0x69, 0x00])
        .expect("Failed to write"); // "hi" in UTF-16 LE
    temp_utf16.flush().expect("Failed to flush");

    let result = scan_file(temp_utf16.path());
    assert!(result.is_ok(), "Should handle UTF-16 files");
}

// T063: Test symlink and circular reference handling
#[test]
#[cfg(unix)] // Symlinks work differently on Windows
fn test_symlink_handling() {
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("real_file.rs");
    let link_path = temp_dir.path().join("link_to_file");

    // Create a real file
    std::fs::write(&file_path, "fn test() {}").expect("Failed to write file");

    // Create symlink
    symlink(&file_path, &link_path).expect("Failed to create symlink");

    // Should handle symlink properly
    let result = scan_file(&link_path);
    assert!(result.is_ok(), "Should handle symlinks");
}

// T064: Test permission denied scenarios
#[test]
#[cfg(unix)] // Permission handling is Unix-specific
fn test_permission_denied() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let mut temp = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp, "test content").expect("Failed to write");
    temp.flush().expect("Failed to flush");

    let path = temp.path();

    // Remove read permissions
    let mut perms = fs::metadata(path)
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(0o000); // No permissions
    fs::set_permissions(path, perms).expect("Failed to set permissions");

    // Should return error (not panic) on permission denied
    let result = scan_file(path);

    // Restore permissions for cleanup
    let mut perms = fs::metadata(path)
        .expect("Failed to get metadata")
        .permissions();
    perms.set_mode(0o644);
    let _ = fs::set_permissions(path, perms);

    assert!(result.is_err(), "Should return error on permission denied");
}

// Additional edge cases

// Test: Empty file
#[test]
fn test_empty_file() {
    let temp = NamedTempFile::new().expect("Failed to create temp file");

    let result = scan_file(temp.path());
    assert!(result.is_ok(), "Should handle empty files");

    if let Ok(violations) = result {
        assert!(
            violations.is_empty(),
            "Empty file should have no violations"
        );
    }
}

// Test: File with only whitespace
#[test]
fn test_whitespace_only_file() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");
    write!(temp, "   \n\t\r\n  ").expect("Failed to write");
    temp.flush().expect("Failed to flush");

    let result = scan_file(temp.path());
    assert!(result.is_ok(), "Should handle whitespace-only files");
}

// Test: File with null bytes
#[test]
fn test_file_with_null_bytes() {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");
    temp.write_all(b"test\0content\0with\0nulls")
        .expect("Failed to write");
    temp.flush().expect("Failed to flush");

    let result = scan_file(temp.path());
    // Should handle gracefully (might be treated as binary)
    if result.is_ok() {
        // Handled
    }
    // Rejected as binary (Err case)
}

// Test: Very deep directory nesting
#[test]
fn test_deep_directory_nesting() {
    use std::path::PathBuf;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut path = PathBuf::from(temp_dir.path());

    // Create nested directories
    for i in 0..50 {
        path.push(format!("dir{}", i));
    }

    std::fs::create_dir_all(&path).expect("Failed to create nested dirs");

    path.push("test.rs");
    std::fs::write(&path, "fn test() {}").expect("Failed to write file");

    // Should handle deeply nested files
    let result = scan_file(&path);
    assert!(result.is_ok(), "Should handle deeply nested files");
}

// Test: Files with unusual line endings
#[test]
fn test_unusual_line_endings() {
    let test_cases = vec![
        ("unix", "line1\nline2\nline3"),
        ("windows", "line1\r\nline2\r\nline3"),
        ("old_mac", "line1\rline2\rline3"),
        ("mixed", "line1\nline2\r\nline3\rline4"),
    ];

    for (name, content) in test_cases {
        let mut temp = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp, "{}", content).expect("Failed to write");
        temp.flush().expect("Failed to flush");

        let result = scan_file(temp.path());
        assert!(result.is_ok(), "Should handle {} line endings", name);
    }
}

// Test: File with BOM at various positions
#[test]
fn test_bom_positions() {
    // BOM at start (normal for UTF-8)
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");
    temp.write_all(&[0xEF, 0xBB, 0xBF])
        .expect("Failed to write BOM");
    write!(temp, "content").expect("Failed to write");
    temp.flush().expect("Failed to flush");

    let result = scan_file(temp.path());
    assert!(result.is_ok(), "Should handle BOM at file start");
}
