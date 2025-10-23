//! File encoding detection and handling

use crate::Result;

/// Detect encoding and convert to UTF-8
pub fn detect_and_decode(bytes: &[u8]) -> Result<String> {
    // Try UTF-8 first (fast path)
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }

    // TODO: Implement chardetng for non-UTF-8 files
    Err(crate::Error::Encoding(
        "Non-UTF-8 encoding not yet implemented".to_string(),
    ))
}

/// Check if file appears to be binary (null bytes in first 8KB)
pub fn is_binary(bytes: &[u8]) -> bool {
    let check_len = bytes.len().min(8192);
    bytes[..check_len].contains(&0)
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
        let binary = b"Hello\x00World";
        assert!(is_binary(binary));
    }

    #[test]
    fn test_is_binary_with_text() {
        let text = b"Hello World";
        assert!(!is_binary(text));
    }
}
