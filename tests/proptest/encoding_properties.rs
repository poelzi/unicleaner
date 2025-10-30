// Property-based tests for encoding detection consistency (T049)

use proptest::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

// Property: UTF-8 detection should be consistent
proptest! {
    #[test]
    fn utf8_detection_consistent(s in "\\PC{0,100}") {
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", s).unwrap();
        temp.flush().unwrap();

        // Should successfully detect as UTF-8 or gracefully handle
        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::file_scanner::scan_file(temp.path())
        });

        prop_assert!(result.is_ok(), "UTF-8 detection should not panic");
    }
}

// Property: Same bytes should always detect same encoding
proptest! {
    #[test]
    fn encoding_detection_deterministic(bytes in prop::collection::vec(any::<u8>(), 0..200)) {
        let mut temp1 = NamedTempFile::new().unwrap();
        let mut temp2 = NamedTempFile::new().unwrap();

        temp1.write_all(&bytes).unwrap();
        temp2.write_all(&bytes).unwrap();

        temp1.flush().unwrap();
        temp2.flush().unwrap();

        let result1 = unicleaner::scanner::encoding::detect_encoding(temp1.path());
        let result2 = unicleaner::scanner::encoding::detect_encoding(temp2.path());

        // Both should give the same result
        match (result1, result2) {
            (Ok(enc1), Ok(enc2)) => {
                prop_assert_eq!(enc1, enc2, "Same bytes should detect same encoding");
            },
            (Err(_), Err(_)) => {
                // Both failed - that's consistent too
            },
            _ => {
                prop_assert!(false, "Inconsistent encoding detection results");
            }
        }
    }
}

// Property: Valid UTF-8 should always be detected as UTF-8
proptest! {
    #[test]
    fn valid_utf8_detected(s in "\\PC{1,100}") {
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", s).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::encoding::detect_encoding(temp.path());

        if let Ok(encoding) = result {
            prop_assert!(
                encoding.to_lowercase().contains("utf") || encoding.to_lowercase().contains("8"),
                "Valid UTF-8 string should be detected as UTF-8, got: {}", encoding
            );
        }
    }
}

// Property: BOM detection should be consistent
proptest! {
    #[test]
    fn bom_detection_consistent(content in "\\PC{0,50}") {
        // Create file with UTF-8 BOM
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();  // UTF-8 BOM
        write!(temp, "{}", content).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::encoding::detect_encoding(temp.path());

        if let Ok(encoding) = result {
            // Should detect as UTF-8 (BOM indicates UTF-8)
            prop_assert!(
                encoding.to_lowercase().contains("utf"),
                "File with UTF-8 BOM should be detected as UTF-8"
            );
        }
    }
}

// Property: Empty files should be handled gracefully
proptest! {
    #[test]
    fn empty_file_handled(_unit in prop::bool::ANY) {
        let temp = NamedTempFile::new().unwrap();

        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::encoding::detect_encoding(temp.path())
        });

        prop_assert!(result.is_ok(), "Empty file should not panic encoding detection");
    }
}

// Property: Binary data should be handled without panic
proptest! {
    #[test]
    fn binary_data_handled(bytes in prop::collection::vec(any::<u8>(), 0..500)) {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(&bytes).unwrap();
        temp.flush().unwrap();

        let result = std::panic::catch_unwind(|| {
            unicleaner::scanner::encoding::detect_encoding(temp.path())
        });

        prop_assert!(result.is_ok(), "Binary data should not panic encoding detection");
    }
}

// Property: Encoding detection on same file multiple times gives same result
proptest! {
    #[test]
    fn repeated_detection_consistent(s in "\\PC{0,100}") {
        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", s).unwrap();
        temp.flush().unwrap();

        let results: Vec<_> = (0..3)
            .map(|_| unicleaner::scanner::encoding::detect_encoding(temp.path()))
            .collect();

        // All results should be identical
        for i in 1..results.len() {
            match (&results[0], &results[i]) {
                (Ok(enc0), Ok(enci)) => {
                    prop_assert_eq!(enc0, enci, "Repeated detection should be consistent");
                },
                (Err(_), Err(_)) => {},
                _ => {
                    prop_assert!(false, "Inconsistent results on repeated detection");
                }
            }
        }
    }
}

// Property: Mixed Unicode should be detected as UTF-8
proptest! {
    #[test]
    fn mixed_unicode_detected_as_utf8(chars in prop::collection::vec(any::<char>(), 0..100)) {
        let text: String = chars.into_iter().collect();

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", text).unwrap();
        temp.flush().unwrap();

        let result = unicleaner::scanner::encoding::detect_encoding(temp.path());

        if let Ok(encoding) = result {
            prop_assert!(
                encoding.to_lowercase().contains("utf"),
                "Mixed Unicode should be detected as UTF-8"
            );
        }
    }
}
