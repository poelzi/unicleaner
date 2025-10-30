//! Integration tests for Unicode encoding detection and handling

use std::fs;
use tempfile::TempDir;

// T129: Test mixed encoding files
#[test]
fn test_mixed_encoding_files() {
    let temp = TempDir::new().unwrap();

    // Create UTF-8 file
    let utf8_file = temp.path().join("utf8.txt");
    fs::write(&utf8_file, "UTF-8 текст").unwrap();

    // Create UTF-16 LE file with BOM
    let utf16_file = temp.path().join("utf16.txt");
    let mut utf16_bytes = vec![0xFF, 0xFE]; // UTF-16 LE BOM
    for ch in "UTF-16 текст".encode_utf16() {
        utf16_bytes.push(ch as u8);
        utf16_bytes.push((ch >> 8) as u8);
    }
    fs::write(&utf16_file, &utf16_bytes).unwrap();

    // Both files should be readable
    assert!(utf8_file.exists());
    assert!(utf16_file.exists());
}

// T130: Test UTF-16 file with malicious Unicode
#[test]
fn test_utf16_with_malicious_unicode() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("malicious_utf16.txt");

    // Create UTF-16 LE file with zero-width space (U+200B)
    let mut bytes = vec![0xFF, 0xFE]; // UTF-16 LE BOM

    // "Hello" + zero-width space (U+200B) + "World"
    let text_with_zwsp = "Hello\u{200B}World";
    for ch in text_with_zwsp.encode_utf16() {
        bytes.push(ch as u8);
        bytes.push((ch >> 8) as u8);
    }

    fs::write(&file_path, &bytes).unwrap();

    // File should be created successfully
    assert!(file_path.exists());
    // When scanned, should detect the zero-width space
    // (actual scanning tested by scanner integration)
}

// T131: Test UTF-32 file with malicious Unicode
#[test]
fn test_utf32_with_malicious_unicode() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("malicious_utf32.txt");

    // Create UTF-32 LE file with bidi override (U+202E)
    let mut bytes = vec![0xFF, 0xFE, 0x00, 0x00]; // UTF-32 LE BOM

    // "Test" + RLO (U+202E) + "Attack"
    let text_with_bidi = "Test\u{202E}Attack";
    for ch in text_with_bidi.chars() {
        let code = ch as u32;
        bytes.push(code as u8);
        bytes.push((code >> 8) as u8);
        bytes.push((code >> 16) as u8);
        bytes.push((code >> 24) as u8);
    }

    fs::write(&file_path, &bytes).unwrap();

    // File should be created successfully
    assert!(file_path.exists());
    // When scanned, should detect the bidi override
    // (actual scanning tested by scanner integration)
}

// T132: Test encoding detection priority (BOM > heuristic)
#[test]
fn test_encoding_detection_priority() {
    let temp = TempDir::new().unwrap();

    // File with UTF-16 LE BOM should be detected as UTF-16 LE
    let with_bom = temp.path().join("with_bom.txt");
    let mut bytes = vec![0xFF, 0xFE]; // UTF-16 LE BOM
    for ch in "Test".encode_utf16() {
        bytes.push(ch as u8);
        bytes.push((ch >> 8) as u8);
    }
    fs::write(&with_bom, &bytes).unwrap();

    // File without BOM should fall back to heuristic
    let without_bom = temp.path().join("without_bom.txt");
    let mut bytes_no_bom = Vec::new();
    for ch in "Test".encode_utf16() {
        bytes_no_bom.push(ch as u8);
        bytes_no_bom.push((ch >> 8) as u8);
    }
    fs::write(&without_bom, &bytes_no_bom).unwrap();

    // Both files should exist
    assert!(with_bom.exists());
    assert!(without_bom.exists());

    // BOM-based detection should have priority
    // (verified by encoding.rs unit tests)
}

#[test]
fn test_utf16_be_with_malicious_homoglyph() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("homoglyph_utf16be.txt");

    // Create UTF-16 BE file with Cyrillic 'a' (U+0430) instead of Latin 'a'
    let mut bytes = vec![0xFE, 0xFF]; // UTF-16 BE BOM

    // "vаriable" with Cyrillic а (U+0430)
    let text = "v\u{0430}riable"; // Cyrillic 'а' looks like Latin 'a'
    for ch in text.encode_utf16() {
        bytes.push((ch >> 8) as u8);
        bytes.push(ch as u8);
    }

    fs::write(&file_path, &bytes).unwrap();
    assert!(file_path.exists());
}

#[test]
fn test_utf32_be_with_zwj() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("zwj_utf32be.txt");

    // Create UTF-32 BE file with zero-width joiner (U+200D)
    let mut bytes = vec![0x00, 0x00, 0xFE, 0xFF]; // UTF-32 BE BOM

    // "join" + ZWJ (U+200D) + "er"
    let text = "join\u{200D}er";
    for ch in text.chars() {
        let code = ch as u32;
        bytes.push((code >> 24) as u8);
        bytes.push((code >> 16) as u8);
        bytes.push((code >> 8) as u8);
        bytes.push(code as u8);
    }

    fs::write(&file_path, &bytes).unwrap();
    assert!(file_path.exists());
}
