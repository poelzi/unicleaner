//! File encoding detection and handling

use crate::Result;
use encoding_rs::{Encoding, UTF_16BE, UTF_16LE};
use serde::{Deserialize, Serialize};

/// Detected encoding information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectedEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    Utf32Le,
    Utf32Be,
}

impl DetectedEncoding {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16Le => "UTF-16 LE",
            Self::Utf16Be => "UTF-16 BE",
            Self::Utf32Le => "UTF-32 LE",
            Self::Utf32Be => "UTF-32 BE",
        }
    }
}

/// Detect BOM (Byte Order Mark) at the start of file
fn detect_bom(bytes: &[u8]) -> Option<DetectedEncoding> {
    // UTF-32 BOMs must be checked first: UTF-32 LE BOM (FF FE 00 00) starts
    // with the UTF-16 LE BOM (FF FE), so encoding_rs would misidentify it.
    // encoding_rs follows the Encoding Standard which does not define UTF-32.
    if bytes.len() >= 4 {
        if bytes[..4] == [0xFF, 0xFE, 0x00, 0x00] {
            return Some(DetectedEncoding::Utf32Le);
        }
        if bytes[..4] == [0x00, 0x00, 0xFE, 0xFF] {
            return Some(DetectedEncoding::Utf32Be);
        }
    }

    // UTF-8 and UTF-16 BOMs via encoding_rs
    if let Some((encoding, _bom_len)) = Encoding::for_bom(bytes) {
        if encoding == encoding_rs::UTF_8 {
            return Some(DetectedEncoding::Utf8);
        }
        if encoding == encoding_rs::UTF_16LE {
            return Some(DetectedEncoding::Utf16Le);
        }
        if encoding == encoding_rs::UTF_16BE {
            return Some(DetectedEncoding::Utf16Be);
        }
    }

    None
}

/// Heuristic detection for files without BOM
fn detect_heuristic(bytes: &[u8]) -> Option<DetectedEncoding> {
    if bytes.len() < 4 {
        return None;
    }

    // Check for null byte patterns that indicate UTF-16 or UTF-32
    let null_count = bytes
        .iter()
        .take(100.min(bytes.len()))
        .filter(|&&b| b == 0)
        .count();

    // UTF-32 has many nulls (3 out of 4 bytes for ASCII)
    if null_count >= 4 {
        // Check pattern: ASCII in UTF-32 LE has pattern: XX 00 00 00
        if bytes.len() >= 8
            && bytes[1] == 0x00
            && bytes[2] == 0x00
            && bytes[3] == 0x00
            && bytes[5] == 0x00
            && bytes[6] == 0x00
            && bytes[7] == 0x00
        {
            return Some(DetectedEncoding::Utf32Le);
        }

        // Check pattern: ASCII in UTF-32 BE has pattern: 00 00 00 XX
        if bytes.len() >= 8
            && bytes[0] == 0x00
            && bytes[1] == 0x00
            && bytes[2] == 0x00
            && bytes[4] == 0x00
            && bytes[5] == 0x00
            && bytes[6] == 0x00
        {
            return Some(DetectedEncoding::Utf32Be);
        }
    }

    // UTF-16 has alternating nulls for ASCII text
    if null_count >= 2 {
        // Check for UTF-16 LE pattern: XX 00 XX 00
        let mut le_matches = 0;
        for i in (0..bytes.len().min(100)).step_by(2) {
            if i + 1 < bytes.len() && bytes[i] != 0 && bytes[i + 1] == 0 {
                le_matches += 1;
            }
        }

        if le_matches >= 2 {
            return Some(DetectedEncoding::Utf16Le);
        }

        // Check for UTF-16 BE pattern: 00 XX 00 XX
        let mut be_matches = 0;
        for i in (0..bytes.len().min(100)).step_by(2) {
            if i + 1 < bytes.len() && bytes[i] == 0 && bytes[i + 1] != 0 {
                be_matches += 1;
            }
        }

        if be_matches >= 2 {
            return Some(DetectedEncoding::Utf16Be);
        }
    }

    None
}

/// Best-effort hint for UTF-16/UTF-32, without decoding.
///
/// This is intended to be used to avoid misclassifying UTF-16/UTF-32 text as
/// binary solely due to embedded NUL bytes.
pub fn detect_utf16_or_utf32(bytes: &[u8]) -> Option<DetectedEncoding> {
    match detect_bom(bytes) {
        Some(
            enc @ (DetectedEncoding::Utf16Le
            | DetectedEncoding::Utf16Be
            | DetectedEncoding::Utf32Le
            | DetectedEncoding::Utf32Be),
        ) => Some(enc),
        _ => match detect_heuristic(bytes) {
            Some(
                enc @ (DetectedEncoding::Utf16Le
                | DetectedEncoding::Utf16Be
                | DetectedEncoding::Utf32Le
                | DetectedEncoding::Utf32Be),
            ) => Some(enc),
            _ => None,
        },
    }
}

/// Decode UTF-16 LE bytes to String
pub fn decode_utf16_le(bytes: &[u8]) -> Result<String> {
    let (cow, _encoding, had_errors) = UTF_16LE.decode(bytes);
    if had_errors {
        return Err(crate::Error::Encoding(
            "Invalid UTF-16 LE sequence".to_string(),
        ));
    }
    Ok(cow.into_owned())
}

/// Decode UTF-16 BE bytes to String
pub fn decode_utf16_be(bytes: &[u8]) -> Result<String> {
    let (cow, _encoding, had_errors) = UTF_16BE.decode(bytes);
    if had_errors {
        return Err(crate::Error::Encoding(
            "Invalid UTF-16 BE sequence".to_string(),
        ));
    }
    Ok(cow.into_owned())
}

/// Decode UTF-32 LE bytes to String
pub fn decode_utf32_le(bytes: &[u8]) -> Result<String> {
    if bytes.len() % 4 != 0 {
        return Err(crate::Error::Encoding(
            "Invalid UTF-32 LE length (not multiple of 4)".to_string(),
        ));
    }

    let mut chars = Vec::new();
    for chunk in bytes.chunks_exact(4) {
        let code_point = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let ch = char::from_u32(code_point).ok_or_else(|| {
            crate::Error::Encoding(format!("Invalid UTF-32 code point: U+{:04X}", code_point))
        })?;
        chars.push(ch);
    }

    Ok(chars.into_iter().collect())
}

/// Decode UTF-32 BE bytes to String
pub fn decode_utf32_be(bytes: &[u8]) -> Result<String> {
    if bytes.len() % 4 != 0 {
        return Err(crate::Error::Encoding(
            "Invalid UTF-32 BE length (not multiple of 4)".to_string(),
        ));
    }

    let mut chars = Vec::new();
    for chunk in bytes.chunks_exact(4) {
        let code_point = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let ch = char::from_u32(code_point).ok_or_else(|| {
            crate::Error::Encoding(format!("Invalid UTF-32 code point: U+{:04X}", code_point))
        })?;
        chars.push(ch);
    }

    Ok(chars.into_iter().collect())
}

/// Detect encoding and convert to UTF-8
pub fn detect_and_decode(bytes: &[u8]) -> Result<String> {
    // Check for BOM first (highest priority)
    if let Some(encoding) = detect_bom(bytes) {
        let data = match encoding {
            DetectedEncoding::Utf8 => &bytes[3..], // Skip UTF-8 BOM
            DetectedEncoding::Utf16Le | DetectedEncoding::Utf16Be => &bytes[2..], // Skip UTF-16 BOM
            DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => &bytes[4..], // Skip UTF-32 BOM
        };

        return match encoding {
            DetectedEncoding::Utf8 => std::str::from_utf8(data)
                .map(|s| s.to_string())
                .map_err(|e| crate::Error::Encoding(format!("Invalid UTF-8 after BOM: {}", e))),
            DetectedEncoding::Utf16Le => decode_utf16_le(data),
            DetectedEncoding::Utf16Be => decode_utf16_be(data),
            DetectedEncoding::Utf32Le => decode_utf32_le(data),
            DetectedEncoding::Utf32Be => decode_utf32_be(data),
        };
    }

    // Try heuristic detection for UTF-16/32
    if let Some(encoding) = detect_heuristic(bytes) {
        return match encoding {
            DetectedEncoding::Utf16Le => decode_utf16_le(bytes),
            DetectedEncoding::Utf16Be => decode_utf16_be(bytes),
            DetectedEncoding::Utf32Le => decode_utf32_le(bytes),
            DetectedEncoding::Utf32Be => decode_utf32_be(bytes),
            DetectedEncoding::Utf8 => unreachable!("UTF-8 not returned by heuristic"),
        };
    }

    // Try UTF-8 (after checking for UTF-16/32 to avoid false positives with nulls)
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }

    Err(crate::Error::Encoding(
        "Could not detect encoding (not UTF-8, UTF-16, or UTF-32)".to_string(),
    ))
}

/// Detect encoding and decode to UTF-8, returning both the string and detected
/// encoding
pub fn detect_decode_with_encoding(bytes: &[u8]) -> Result<(String, DetectedEncoding)> {
    // Check for BOM first (highest priority)
    if let Some(encoding) = detect_bom(bytes) {
        let data = match encoding {
            DetectedEncoding::Utf8 => &bytes[3..], // Skip UTF-8 BOM
            DetectedEncoding::Utf16Le | DetectedEncoding::Utf16Be => &bytes[2..], // Skip UTF-16 BOM
            DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => &bytes[4..], // Skip UTF-32 BOM
        };
        let decoded = match encoding {
            DetectedEncoding::Utf8 => std::str::from_utf8(data)
                .map(|s| s.to_string())
                .map_err(|e| crate::Error::Encoding(format!("Invalid UTF-8 after BOM: {}", e)))?,
            DetectedEncoding::Utf16Le => decode_utf16_le(data)?,
            DetectedEncoding::Utf16Be => decode_utf16_be(data)?,
            DetectedEncoding::Utf32Le => decode_utf32_le(data)?,
            DetectedEncoding::Utf32Be => decode_utf32_be(data)?,
        };
        return Ok((decoded, encoding));
    }

    // Try heuristic detection for UTF-16/32
    if let Some(encoding) = detect_heuristic(bytes) {
        let decoded = match encoding {
            DetectedEncoding::Utf16Le => decode_utf16_le(bytes)?,
            DetectedEncoding::Utf16Be => decode_utf16_be(bytes)?,
            DetectedEncoding::Utf32Le => decode_utf32_le(bytes)?,
            DetectedEncoding::Utf32Be => decode_utf32_be(bytes)?,
            DetectedEncoding::Utf8 => unreachable!("UTF-8 not returned by heuristic"),
        };
        return Ok((decoded, encoding));
    }

    // Try UTF-8 (after checking for UTF-16/32 to avoid false positives with nulls)
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok((s.to_string(), DetectedEncoding::Utf8));
    }

    // Fall back to error
    Err(crate::Error::Encoding(
        "Could not detect encoding (not UTF-8, UTF-16, or UTF-32)".to_string(),
    ))
}

/// Detect the encoding of a file by path
///
/// This is a convenience function for property tests that reads the file
/// and returns the detected encoding name as a string.
pub fn detect_encoding(path: &std::path::Path) -> Result<String> {
    use std::fs;

    let bytes = fs::read(path)?;
    let (_content, encoding) = detect_decode_with_encoding(&bytes)?;
    Ok(encoding.name().to_string())
}

/// Check if file appears to be binary (null bytes in first 8KB)
pub fn is_binary(bytes: &[u8]) -> bool {
    let check_len = bytes.len().min(8192);
    if check_len == 0 {
        return false;
    }

    let sample = &bytes[..check_len];

    // Heuristic 1: Consecutive nulls (binary files tend to have many)
    // UTF-16/32 text will have nulls but not many consecutive ones
    let mut max_consecutive_nulls: usize = 0;
    let mut current_nulls: usize = 0;

    // Heuristic 2: Control byte ratio
    // Count bytes in 0x00-0x08, 0x0E-0x1F (excluding TAB 0x09, LF 0x0A, CR 0x0D)
    let mut control_count: usize = 0;

    for &byte in sample {
        if byte == 0 {
            current_nulls += 1;
            max_consecutive_nulls = max_consecutive_nulls.max(current_nulls);
        } else {
            current_nulls = 0;
        }

        if matches!(byte, 0x00..=0x08 | 0x0E..=0x1F) {
            control_count += 1;
        }
    }

    // Binary if many consecutive nulls
    if max_consecutive_nulls > 10 {
        return true;
    }

    // Binary if control byte ratio > 30%
    let control_ratio = (control_count as f64) / (check_len as f64);
    if control_ratio > 0.30 {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_utf8() {
        let text = "Hello, world!";
        let result = detect_and_decode(text.as_bytes()).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_detect_utf8_with_unicode() {
        let text = "Hello, 世界!";
        let result = detect_and_decode(text.as_bytes()).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_is_binary_with_null_bytes() {
        let binary = b"Hello\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00World";
        assert!(is_binary(binary));
    }

    #[test]
    fn test_is_binary_with_text() {
        let text = b"Hello World";
        assert!(!is_binary(text));
    }

    // T064: Binary detection via control byte ratio
    #[test]
    fn test_binary_detection_control_byte_ratio() {
        // 40% control bytes, zero consecutive nulls
        let mut data = Vec::new();
        for i in 0..100u8 {
            if i < 40 {
                data.push(0x01); // control byte (SOH)
            } else {
                data.push(0x41); // 'A'
            }
        }
        assert!(is_binary(&data), "High control byte ratio should be binary");
    }

    // T065: Normal text with occasional controls is NOT binary
    #[test]
    fn test_text_file_not_falsely_binary() {
        let mut data = Vec::new();
        // Normal text with tabs and newlines
        for _ in 0..100 {
            data.extend_from_slice(b"Hello\tWorld\n");
        }
        assert!(
            !is_binary(&data),
            "Normal text with tabs/newlines should not be binary"
        );
    }

    // T121: UTF-16 LE BOM detection
    #[test]
    fn test_detect_utf16_le_bom() {
        // UTF-16 LE BOM: FF FE
        let text = "Hello";
        let mut bytes = vec![0xFF, 0xFE]; // UTF-16 LE BOM
        // Encode "Hello" in UTF-16 LE
        for ch in text.encode_utf16() {
            bytes.push(ch as u8);
            bytes.push((ch >> 8) as u8);
        }

        let result = detect_and_decode(&bytes);
        assert!(result.is_ok(), "UTF-16 LE with BOM should be detected");
        assert_eq!(result.unwrap(), text);
    }

    // T122: UTF-16 BE BOM detection
    #[test]
    fn test_detect_utf16_be_bom() {
        // UTF-16 BE BOM: FE FF
        let text = "Hello";
        let mut bytes = vec![0xFE, 0xFF]; // UTF-16 BE BOM
        // Encode "Hello" in UTF-16 BE
        for ch in text.encode_utf16() {
            bytes.push((ch >> 8) as u8);
            bytes.push(ch as u8);
        }

        let result = detect_and_decode(&bytes);
        assert!(result.is_ok(), "UTF-16 BE with BOM should be detected");
        assert_eq!(result.unwrap(), text);
    }

    // T123: UTF-32 LE BOM detection
    #[test]
    fn test_detect_utf32_le_bom() {
        // UTF-32 LE BOM: FF FE 00 00
        let text = "Hi";
        let mut bytes = vec![0xFF, 0xFE, 0x00, 0x00]; // UTF-32 LE BOM
        // Encode "Hi" in UTF-32 LE
        for ch in text.chars() {
            let code = ch as u32;
            bytes.push(code as u8);
            bytes.push((code >> 8) as u8);
            bytes.push((code >> 16) as u8);
            bytes.push((code >> 24) as u8);
        }

        let result = detect_and_decode(&bytes);
        assert!(result.is_ok(), "UTF-32 LE with BOM should be detected");
        assert_eq!(result.unwrap(), text);
    }

    // T124: UTF-32 BE BOM detection
    #[test]
    fn test_detect_utf32_be_bom() {
        // UTF-32 BE BOM: 00 00 FE FF
        let text = "Hi";
        let mut bytes = vec![0x00, 0x00, 0xFE, 0xFF]; // UTF-32 BE BOM
        // Encode "Hi" in UTF-32 BE
        for ch in text.chars() {
            let code = ch as u32;
            bytes.push((code >> 24) as u8);
            bytes.push((code >> 16) as u8);
            bytes.push((code >> 8) as u8);
            bytes.push(code as u8);
        }

        let result = detect_and_decode(&bytes);
        assert!(result.is_ok(), "UTF-32 BE with BOM should be detected");
        assert_eq!(result.unwrap(), text);
    }

    // T125: UTF-16 LE decoding without BOM
    #[test]
    fn test_decode_utf16_le_no_bom() {
        let text = "Test";
        let mut bytes = Vec::new();
        // Encode "Test" in UTF-16 LE without BOM
        for ch in text.encode_utf16() {
            bytes.push(ch as u8);
            bytes.push((ch >> 8) as u8);
        }

        // Should detect UTF-16 LE heuristically
        let result = detect_and_decode(&bytes);
        assert!(
            result.is_ok(),
            "UTF-16 LE without BOM should be detected heuristically"
        );
        assert_eq!(result.unwrap(), text);
    }

    // T126: UTF-16 BE decoding without BOM
    #[test]
    fn test_decode_utf16_be_no_bom() {
        let text = "Test";
        let mut bytes = Vec::new();
        // Encode "Test" in UTF-16 BE without BOM
        for ch in text.encode_utf16() {
            bytes.push((ch >> 8) as u8);
            bytes.push(ch as u8);
        }

        // Should detect UTF-16 BE heuristically
        let result = detect_and_decode(&bytes);
        assert!(
            result.is_ok(),
            "UTF-16 BE without BOM should be detected heuristically"
        );
        assert_eq!(result.unwrap(), text);
    }

    // T127: UTF-32 LE decoding without BOM
    #[test]
    fn test_decode_utf32_le_no_bom() {
        let text = "AB";
        let mut bytes = Vec::new();
        // Encode "AB" in UTF-32 LE without BOM
        for ch in text.chars() {
            let code = ch as u32;
            bytes.push(code as u8);
            bytes.push((code >> 8) as u8);
            bytes.push((code >> 16) as u8);
            bytes.push((code >> 24) as u8);
        }

        // Should detect UTF-32 LE heuristically
        let result = detect_and_decode(&bytes);
        assert!(
            result.is_ok(),
            "UTF-32 LE without BOM should be detected heuristically"
        );
        assert_eq!(result.unwrap(), text);
    }

    // T128: UTF-32 BE decoding without BOM
    #[test]
    fn test_decode_utf32_be_no_bom() {
        let text = "AB";
        let mut bytes = Vec::new();
        // Encode "AB" in UTF-32 BE without BOM
        for ch in text.chars() {
            let code = ch as u32;
            bytes.push((code >> 24) as u8);
            bytes.push((code >> 16) as u8);
            bytes.push((code >> 8) as u8);
            bytes.push(code as u8);
        }

        // Should detect UTF-32 BE heuristically
        let result = detect_and_decode(&bytes);
        assert!(
            result.is_ok(),
            "UTF-32 BE without BOM should be detected heuristically"
        );
        assert_eq!(result.unwrap(), text);
    }

    #[test]
    fn test_detect_heuristic_tiny_input() {
        // Less than 4 bytes should return None
        assert!(detect_heuristic(&[0x41]).is_none());
        assert!(detect_heuristic(&[0x41, 0x42]).is_none());
        assert!(detect_heuristic(&[0x41, 0x42, 0x43]).is_none());
    }

    #[test]
    fn test_detect_and_decode_unrecognizable() {
        // Bytes that are not valid UTF-8, UTF-16, or UTF-32
        let bytes = [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87];
        let result = detect_and_decode(&bytes);
        assert!(result.is_err(), "Unrecognizable bytes should error");
    }

    #[test]
    fn test_detect_decode_with_encoding_unrecognizable() {
        let bytes = [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87];
        let result = detect_decode_with_encoding(&bytes);
        assert!(result.is_err(), "Unrecognizable bytes should error");
    }

    #[test]
    fn test_is_binary_empty() {
        assert!(!is_binary(&[]), "Empty input should not be binary");
    }

    #[test]
    fn test_detect_encoding_convenience() {
        let temp = tempfile::TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, "Hello world").unwrap();
        let result = detect_encoding(&file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "UTF-8");
    }

    #[test]
    fn test_decode_utf32_le_bad_length() {
        // Not a multiple of 4
        let bytes = [0x41, 0x00, 0x00, 0x00, 0x42];
        let result = decode_utf32_le(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_utf32_be_bad_length() {
        let bytes = [0x00, 0x00, 0x00, 0x41, 0x42];
        let result = decode_utf32_be(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_utf32_le_surrogate() {
        // U+D800 is a surrogate — invalid as a code point
        let bytes: [u8; 4] = [0x00, 0xD8, 0x00, 0x00];
        let result = decode_utf32_le(&bytes);
        assert!(result.is_err(), "Surrogate code point should error");
    }

    #[test]
    fn test_decode_utf32_be_surrogate() {
        // U+D800 in big-endian
        let bytes: [u8; 4] = [0x00, 0x00, 0xD8, 0x00];
        let result = decode_utf32_be(&bytes);
        assert!(result.is_err(), "Surrogate code point should error");
    }
}
