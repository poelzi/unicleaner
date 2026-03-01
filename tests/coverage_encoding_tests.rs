// Additional coverage tests for scanner/encoding module
use unicleaner::scanner::encoding::{
    DetectedEncoding, decode_utf32_le, detect_decode_with_encoding, is_binary,
};

/// Test that heuristic detection with tiny input (< 4 bytes) falls through.
/// detect_heuristic is private, but we can exercise it through detect_decode_with_encoding
/// which calls detect_heuristic when no BOM is found. With only 2 bytes of valid UTF-8,
/// detect_heuristic returns None and the code falls through to UTF-8 parsing.
#[test]
fn test_detect_heuristic_tiny_input() {
    // 2 bytes, no BOM - detect_heuristic will get called and return None (< 4 bytes)
    let bytes: &[u8] = b"AB";
    let (text, encoding) = detect_decode_with_encoding(bytes).unwrap();
    assert_eq!(text, "AB");
    assert_eq!(encoding, DetectedEncoding::Utf8);
}

/// Test is_binary with empty slice - should return false (check_len == 0 early exit)
#[test]
fn test_is_binary_empty() {
    assert!(!is_binary(&[]));
}

/// Test decode_utf32_le with a length not divisible by 4 - should return Err
#[test]
fn test_decode_utf32_le_bad_length() {
    let bytes: &[u8] = &[0x41, 0x00, 0x00, 0x00, 0x42]; // 5 bytes
    let result = decode_utf32_le(bytes);
    assert!(result.is_err());
}

/// Test decode_utf32_le with bytes encoding U+D800 (surrogate) - should return Err
/// because char::from_u32(0xD800) returns None
#[test]
fn test_decode_utf32_le_surrogate() {
    // U+D800 in little-endian: 0x00, 0xD8, 0x00, 0x00
    let bytes: &[u8] = &[0x00, 0xD8, 0x00, 0x00];
    let result = decode_utf32_le(bytes);
    assert!(result.is_err());
}

/// Test detect_encoding with simple UTF-8 bytes (no BOM) - verifies Utf8 encoding
#[test]
fn test_detect_encoding_basic() {
    let bytes = b"Hello, world!";
    let (text, encoding) = detect_decode_with_encoding(bytes).unwrap();
    assert_eq!(text, "Hello, world!");
    assert_eq!(encoding, DetectedEncoding::Utf8);
}
