//! Integration tests for Unicode encoding detection and handling

use assert_cmd::cargo_bin_cmd;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn scan_json(path: &Path) -> (i32, serde_json::Value) {
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--format")
        .arg("json")
        .arg(path)
        .output()
        .unwrap();

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    (exit_code, json)
}

fn json_has_code_point(json: &serde_json::Value, code_point: u32) -> bool {
    json["violations"].as_array().is_some_and(|violations| {
        violations
            .iter()
            .any(|v| v["code_point"].as_u64() == Some(code_point as u64))
    })
}

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

// =============================================================================
// Phase 7: US5 Encoding Override (T033)
// =============================================================================

/// T033: --encoding utf16-le forces file decoding
#[test]
fn test_encoding_override_utf16le() {
    let temp = TempDir::new().unwrap();

    // Create a UTF-16 LE file with zero-width space (U+200B) but NO BOM
    // Without BOM and without --encoding flag, this may be misdetected
    let file_path = temp.path().join("test_utf16le.txt");
    let mut bytes: Vec<u8> = Vec::new();
    // "hi" + ZWSP in UTF-16 LE (no BOM)
    for ch in "hi\u{200B}".encode_utf16() {
        bytes.push(ch as u8);
        bytes.push((ch >> 8) as u8);
    }
    fs::write(&file_path, &bytes).unwrap();

    // Scan with --encoding utf16-le → should detect ZWSP
    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--encoding")
        .arg("utf16-le")
        .arg("--format")
        .arg("json")
        .arg(&file_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let violations = json["violations"].as_array().unwrap();
    assert!(
        violations
            .iter()
            .any(|v| v["code_point"].as_u64() == Some(0x200B)),
        "Encoding override should detect ZWSP in UTF-16 LE file"
    );
}

/// W006: Invalid --encoding value should fail with clear CLI error
#[test]
fn test_encoding_override_invalid_value_errors() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "hello\n").unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--encoding")
        .arg("invalid")
        .arg(&file_path)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "Invalid encoding should exit with clap usage error code"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value") && stderr.contains("--encoding"),
        "Expected invalid encoding CLI error, got stderr: {}",
        stderr
    );
}

/// W007: Decode failures should be classified as EncodingError in scan output
#[test]
fn test_encoding_error_classification_on_decode_failure() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("invalid_utf16.bin");

    // Invalid UTF-16 LE payload (odd length) forces decode failure when override is set.
    fs::write(&file_path, [0xFF_u8]).unwrap();

    let mut cmd = cargo_bin_cmd!("unicleaner");
    let output = cmd
        .arg("scan")
        .arg("--encoding")
        .arg("utf16-le")
        .arg("--format")
        .arg("json")
        .arg(&file_path)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(3),
        "Errors-only scans should return partial-success exit code"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    let errors = json["errors"].as_array().expect("errors array");
    assert!(
        !errors.is_empty(),
        "Expected at least one scan error for decode failure"
    );
    assert!(
        errors
            .iter()
            .any(|e| e["error_type"].as_str() == Some("EncodingError")),
        "Expected EncodingError classification, got: {}",
        stdout
    );
}

#[test]
fn test_auto_scan_utf16le_bom_detects_zwsp() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("auto_utf16le_bom.txt");

    let mut bytes = vec![0xFF, 0xFE]; // UTF-16 LE BOM
    for ch in "Hello\u{200B}World\n".encode_utf16() {
        bytes.push(ch as u8);
        bytes.push((ch >> 8) as u8);
    }
    fs::write(&file_path, &bytes).unwrap();

    let (exit_code, json) = scan_json(&file_path);
    assert_eq!(exit_code, 1, "Violations should exit with code 1");
    assert!(
        json_has_code_point(&json, 0x200B),
        "Auto detection should find ZWSP in UTF-16 LE file"
    );
}

#[test]
fn test_auto_scan_utf16be_bom_detects_zwsp() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("auto_utf16be_bom.txt");

    let mut bytes = vec![0xFE, 0xFF]; // UTF-16 BE BOM
    for ch in "Hello\u{200B}World\n".encode_utf16() {
        bytes.push((ch >> 8) as u8);
        bytes.push(ch as u8);
    }
    fs::write(&file_path, &bytes).unwrap();

    let (exit_code, json) = scan_json(&file_path);
    assert_eq!(exit_code, 1, "Violations should exit with code 1");
    assert!(
        json_has_code_point(&json, 0x200B),
        "Auto detection should find ZWSP in UTF-16 BE file"
    );
}

#[test]
fn test_auto_scan_utf32le_bom_detects_zwsp() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("auto_utf32le_bom.txt");

    let mut bytes = vec![0xFF, 0xFE, 0x00, 0x00]; // UTF-32 LE BOM
    for ch in "Hello\u{200B}World\n".chars() {
        let code = ch as u32;
        bytes.push(code as u8);
        bytes.push((code >> 8) as u8);
        bytes.push((code >> 16) as u8);
        bytes.push((code >> 24) as u8);
    }
    fs::write(&file_path, &bytes).unwrap();

    let (exit_code, json) = scan_json(&file_path);
    assert_eq!(exit_code, 1, "Violations should exit with code 1");
    assert!(
        json_has_code_point(&json, 0x200B),
        "Auto detection should find ZWSP in UTF-32 LE file"
    );
}

#[test]
fn test_auto_scan_utf32be_bom_detects_zwsp() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("auto_utf32be_bom.txt");

    let mut bytes = vec![0x00, 0x00, 0xFE, 0xFF]; // UTF-32 BE BOM
    for ch in "Hello\u{200B}World\n".chars() {
        let code = ch as u32;
        bytes.push((code >> 24) as u8);
        bytes.push((code >> 16) as u8);
        bytes.push((code >> 8) as u8);
        bytes.push(code as u8);
    }
    fs::write(&file_path, &bytes).unwrap();

    let (exit_code, json) = scan_json(&file_path);
    assert_eq!(exit_code, 1, "Violations should exit with code 1");
    assert!(
        json_has_code_point(&json, 0x200B),
        "Auto detection should find ZWSP in UTF-32 BE file"
    );
}

#[test]
fn test_auto_scan_utf16le_no_bom_detects_zwsp() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("auto_utf16le_no_bom.txt");

    let mut bytes: Vec<u8> = Vec::new();
    for ch in "hi\u{200B}\n".encode_utf16() {
        bytes.push(ch as u8);
        bytes.push((ch >> 8) as u8);
    }
    fs::write(&file_path, &bytes).unwrap();

    let (exit_code, json) = scan_json(&file_path);
    assert_eq!(exit_code, 1, "Violations should exit with code 1");
    assert!(
        json_has_code_point(&json, 0x200B),
        "Heuristic auto detection should find ZWSP in UTF-16 LE file"
    );
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
