//! Core file scanning logic

use crate::Result;
use crate::config::Configuration;
use crate::report::Violation;
use crate::scanner::encoding::{
    DetectedEncoding, detect_decode_with_encoding, detect_utf16_or_utf32, is_binary,
};
use crate::scanner::unicode_detector::detect_in_string_with_policy;
use std::fs;
use std::path::Path;

/// Decode bytes using a specific encoding override
fn decode_with_override(bytes: &[u8], encoding: DetectedEncoding) -> Result<String> {
    use crate::scanner::encoding::{
        decode_utf16_be, decode_utf16_le, decode_utf32_be, decode_utf32_le,
    };
    match encoding {
        DetectedEncoding::Utf8 => std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|e| crate::Error::Encoding(format!("Invalid UTF-8: {}", e))),
        DetectedEncoding::Utf16Le => decode_utf16_le(bytes),
        DetectedEncoding::Utf16Be => decode_utf16_be(bytes),
        DetectedEncoding::Utf32Le => decode_utf32_le(bytes),
        DetectedEncoding::Utf32Be => decode_utf32_be(bytes),
    }
}

/// Scan a single file for malicious Unicode with configuration
pub fn scan_file_with_config(
    path: &Path,
    config: &Configuration,
    encoding_override: Option<DetectedEncoding>,
) -> Result<Vec<Violation>> {
    use crate::config::rules::find_matching_rule;

    // Read file bytes
    let bytes = fs::read(path)?;

    // Decode to UTF-8: use override or auto-detect.
    //
    // Important: UTF-16/UTF-32 text contains many NUL bytes and can trip the
    // binary heuristic; detect likely UTF-16/UTF-32 before skipping as binary.
    let (content, encoding) = if let Some(enc) = encoding_override {
        let content = decode_with_override(&bytes, enc)?;
        (content, enc)
    } else {
        let utf16_or_utf32_hint = detect_utf16_or_utf32(&bytes);

        if utf16_or_utf32_hint.is_none() && is_binary(&bytes) {
            return Ok(Vec::new());
        }

        match detect_decode_with_encoding(&bytes) {
            Ok(v) => v,
            Err(e) => {
                // If we only had a heuristic hint and decoding failed, fall
                // back to treating the file as binary (if it still looks like
                // binary) to avoid spurious errors on arbitrary bytes.
                if utf16_or_utf32_hint.is_some() && is_binary(&bytes) {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        }
    };

    let matching_rule = find_matching_rule(&config.file_rules, path);
    let denied_code_points = matching_rule
        .map(|r| r.denied_code_points.as_slice())
        .unwrap_or(&[]);
    let allowed_ranges = config.get_allowed_ranges(path);

    // Detect malicious Unicode and policy violations.
    let mut violations = detect_in_string_with_policy(
        &content,
        path,
        config.deny_by_default,
        allowed_ranges.as_deref(),
        denied_code_points,
    );

    // Apply allowlist suppression / always-deny enforcement.
    violations = apply_config_rules(violations, allowed_ranges.as_deref(), denied_code_points);

    // Add encoding information to all violations
    for violation in &mut violations {
        violation.encoding = encoding;
    }

    Ok(violations)
}

/// Scan a single file for malicious Unicode (without config - uses defaults)
pub fn scan_file(path: &Path) -> Result<Vec<Violation>> {
    scan_file_with_config(path, &Configuration::default(), None)
}

/// Check if a code point is an always-deny pattern (bidi controls)
fn is_always_deny(code_point: u32) -> bool {
    // Bidi marks: LRM, RLM, ALM
    // Bidi override controls U+202A-202E
    // Bidi isolate controls U+2066-2069
    matches!(code_point, 0x061C | 0x200E..=0x200F | 0x202A..=0x202E | 0x2066..=0x2069)
}

/// Apply configuration rules to filter violations
fn apply_config_rules(
    violations: Vec<Violation>,
    allowed_ranges: Option<&[crate::unicode::ranges::UnicodeRange]>,
    denied_code_points: &[u32],
) -> Vec<Violation> {
    violations
        .into_iter()
        .filter(|v| {
            // Always-deny patterns are never allowed regardless of config
            if is_always_deny(v.code_point) {
                return true;
            }

            // Explicit denies override allowlists.
            if denied_code_points.contains(&v.code_point) {
                return true;
            }

            // If allowlisted (rule or preset), suppress the violation.
            if allowed_ranges.is_some_and(|ranges| ranges.iter().any(|r| r.contains(v.code_point)))
            {
                return false;
            }

            // Otherwise, keep the violation.
            true
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_scan_file_returns_violations() {
        // This test will fail until we implement scanning
        let path = PathBuf::from("tests/integration/fixtures/zero_width/test1.rs");
        if path.exists() {
            let violations = scan_file(&path).unwrap();
            // Should find zero-width space
            assert!(!violations.is_empty(), "Should detect malicious Unicode");
            assert!(violations.iter().any(|v| v.code_point == 0x200B));
        }
    }

    #[test]
    fn test_scan_clean_file() {
        let path = PathBuf::from("tests/integration/fixtures/clean/simple.rs");
        if path.exists() {
            let violations = scan_file(&path).unwrap();
            assert!(
                violations.is_empty(),
                "Clean file should have no violations"
            );
        }
    }
}
