// Property-based tests for scanner stability (T046)
// Ensures scanner never panics on any input

use proptest::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

// Property: Scanner should never panic on arbitrary UTF-8 strings
proptest! {
    #[test]
    fn scanner_never_panics_on_utf8(s in "\\PC*") {
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", s).unwrap();

        // Should not panic - may return Ok or Err, but shouldn't crash
        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp.path())
        });

        prop_assert!(result.is_ok(), "Scanner panicked on valid UTF-8 input");
    }
}

// Property: Scanner should handle arbitrary byte sequences gracefully
proptest! {
    #[test]
    fn scanner_handles_arbitrary_bytes(bytes in prop::collection::vec(any::<u8>(), 0..1000)) {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(&bytes).unwrap();

        // Should not panic, even on invalid UTF-8
        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp.path())
        });

        prop_assert!(result.is_ok(), "Scanner panicked on arbitrary bytes");
    }
}

// Property: Scanner should handle very long lines
proptest! {
    #[test]
    fn scanner_handles_long_lines(len in 1usize..100000) {
        let line = "a".repeat(len);
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", line).unwrap();

        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp.path())
        });

        prop_assert!(result.is_ok(), "Scanner panicked on long line of length {}", len);
    }
}

// Property: Scanner should handle files with many lines
proptest! {
    #[test]
    fn scanner_handles_many_lines(num_lines in 1usize..10000) {
        let mut temp = NamedTempFile::new().unwrap();
        for i in 0..num_lines {
            writeln!(temp, "line {}", i).unwrap();
        }

        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp.path())
        });

        prop_assert!(result.is_ok(), "Scanner panicked on {} lines", num_lines);
    }
}

// Property: Scanner should handle mixed Unicode content
proptest! {
    #[test]
    fn scanner_handles_mixed_unicode(chars in prop::collection::vec(any::<char>(), 0..1000)) {
        let text: String = chars.into_iter().collect();
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", text).unwrap();

        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp.path())
        });

        prop_assert!(result.is_ok(), "Scanner panicked on mixed Unicode");
    }
}
