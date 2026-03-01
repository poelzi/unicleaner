//! Unicode detection logic

use crate::report::Violation;
use crate::unicode::malicious::{MaliciousCategory, Severity, pattern_for};
use crate::unicode::ranges::UnicodeRange;
use std::path::Path;

/// Detect malicious Unicode in a string
pub fn detect_in_string(content: &str, file_path: &Path) -> Vec<Violation> {
    detect_in_string_with_policy(content, file_path, false, None, &[])
}

pub fn detect_in_string_with_policy(
    content: &str,
    file_path: &Path,
    deny_by_default: bool,
    allowed_ranges: Option<&[UnicodeRange]>,
    denied_code_points: &[u32],
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let mut column: usize = 1;
        for (byte_offset, ch) in line.char_indices() {
            let code_point = ch as u32;

            if let Some(pattern) = pattern_for(code_point) {
                let violation = Violation::new(
                    file_path.to_path_buf(),
                    line_num + 1, // 1-indexed
                    column,       // 1-indexed, character-based
                    byte_offset,  // byte offset within line
                    code_point,
                    pattern.name.clone(),
                    pattern.category,
                    pattern.severity,
                    pattern.description.clone(),
                )
                .with_context(context_snippet(line, byte_offset));

                violations.push(violation);
                column += 1;
                continue;
            }

            if denied_code_points.contains(&code_point) {
                let violation = Violation::new(
                    file_path.to_path_buf(),
                    line_num + 1,
                    column,
                    byte_offset,
                    code_point,
                    "explicitly-denied".to_string(),
                    MaliciousCategory::NonStandard,
                    Severity::Error,
                    "Code point is explicitly denied by configuration".to_string(),
                )
                .with_context(context_snippet(line, byte_offset));
                violations.push(violation);
                column += 1;
                continue;
            }

            if deny_by_default && !is_allowed_by_policy(code_point, allowed_ranges) {
                let violation = Violation::new(
                    file_path.to_path_buf(),
                    line_num + 1,
                    column,
                    byte_offset,
                    code_point,
                    "disallowed-code-point".to_string(),
                    MaliciousCategory::NonStandard,
                    Severity::Error,
                    "Code point is outside the configured allowlist (deny-by-default)".to_string(),
                )
                .with_context(context_snippet(line, byte_offset));
                violations.push(violation);
            }

            column += 1;
        }
    }

    violations
}

fn is_allowed_by_policy(code_point: u32, allowed_ranges: Option<&[UnicodeRange]>) -> bool {
    if let Some(ranges) = allowed_ranges {
        return ranges.iter().any(|r| r.contains(code_point));
    }

    // Default allowlist for deny-by-default: safe ASCII only.
    matches!(code_point, 0x0009..=0x000D | 0x0020..=0x007E)
}

fn context_snippet(line: &str, byte_offset: usize) -> String {
    const CONTEXT_BYTES: usize = 60;

    if line.len() <= CONTEXT_BYTES * 2 {
        return line.to_string();
    }

    let mut start = byte_offset.saturating_sub(CONTEXT_BYTES);
    while start > 0 && !line.is_char_boundary(start) {
        start -= 1;
    }

    let mut end = (byte_offset + CONTEXT_BYTES).min(line.len());
    while end < line.len() && !line.is_char_boundary(end) {
        end += 1;
    }

    let mut out = String::new();
    if start > 0 {
        out.push_str("...");
    }
    out.push_str(&line[start..end]);
    if end < line.len() {
        out.push_str("...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_zero_width_space() {
        let content = "let user\u{200B}name = \"admin\";"; // Contains U+200B
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code_point, 0x200B);
        assert_eq!(violations[0].pattern_name, "zero-width-space");
    }

    #[test]
    fn test_detect_bidi_override() {
        // Construct string with U+202E (right-to-left override) to avoid compiler
        // warning
        let bidi_char = char::from_u32(0x202E).unwrap();
        let content = format!("/* {} }}", bidi_char);
        let violations = detect_in_string(&content, &PathBuf::from("test.rs"));

        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.code_point == 0x202E));
    }

    #[test]
    fn test_clean_content() {
        let content = "let username = \"admin\";";
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        assert!(violations.is_empty());
    }

    // T039: Column is character-based, not byte-based
    #[test]
    fn test_column_is_char_based_not_byte() {
        // "é" is 2 bytes in UTF-8, then a zero-width space at char position 2
        // Byte layout: é(2 bytes) + U+200B(3 bytes) = byte offset 2 for ZWSP
        // Char layout: é(1 char) + U+200B(1 char) = char column 2 for ZWSP
        let content = "é\u{200B}x";
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code_point, 0x200B);
        // Column should be char-based: é is char 1, ZWSP is char 2
        assert_eq!(
            violations[0].column, 2,
            "Column should be char-based (1-indexed)"
        );
        // Byte offset should be the raw byte position: é takes 2 bytes
        assert_eq!(
            violations[0].byte_offset, 2,
            "Byte offset should be byte position"
        );
    }

    // T040: byte_offset field is populated correctly
    #[test]
    fn test_byte_offset_field_populated() {
        // "abc" + ZWSP: byte offset should be 3
        let content = "abc\u{200B}";
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].byte_offset, 3);
        assert_eq!(violations[0].column, 4); // 1-indexed char position
    }

    #[test]
    fn test_zwj_position_inside_grapheme_cluster() {
        // Woman technologist emoji: U+1F469 U+200D U+1F4BB
        // Ensure we report the ZWJ scalar position, not the grapheme start.
        let content = "a\u{1F469}\u{200D}\u{1F4BB}b";
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        let zwj = violations
            .iter()
            .find(|v| v.code_point == 0x200D)
            .expect("Should detect ZWJ");

        assert_eq!(zwj.column, 3, "ZWJ should be at scalar column 3");
        assert_eq!(zwj.byte_offset, 5, "ZWJ should start at byte offset 5");
    }

    #[test]
    fn test_combining_mark_position() {
        // 'e' + COMBINING ACUTE ACCENT (U+0301)
        let content = "e\u{0301}";
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        let combining = violations
            .iter()
            .find(|v| v.code_point == 0x0301)
            .expect("Should detect combining mark");

        assert_eq!(combining.column, 2, "Combining mark should be at column 2");
        assert_eq!(
            combining.byte_offset, 1,
            "Combining mark should start at byte offset 1"
        );
    }

    #[test]
    fn test_denied_code_points() {
        // 'é' (U+00E9) is not malicious but we deny it explicitly
        let content = "caf\u{00E9}";
        let violations = detect_in_string_with_policy(
            content,
            &PathBuf::from("test.rs"),
            false,
            None,
            &[0x00E9],
        );

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code_point, 0x00E9);
        assert_eq!(violations[0].pattern_name, "explicitly-denied");
    }

    #[test]
    fn test_deny_by_default_with_no_allowlist() {
        // 'é' (U+00E9) is outside ASCII range; deny-by-default with no custom ranges
        let content = "caf\u{00E9}";
        let violations = detect_in_string_with_policy(
            content,
            &PathBuf::from("test.rs"),
            true,
            None, // Falls back to default safe ASCII
            &[],
        );

        // 'é' should be flagged as disallowed
        assert!(violations.iter().any(|v| v.code_point == 0x00E9));
        let v = violations.iter().find(|v| v.code_point == 0x00E9).unwrap();
        assert_eq!(v.pattern_name, "disallowed-code-point");
    }

    #[test]
    fn test_deny_by_default_with_allowlist() {
        // 'é' (U+00E9) is explicitly allowed via range
        let ranges = vec![UnicodeRange::new(0x0000, 0x00FF)]; // Basic Latin + Latin-1 Supplement
        let content = "caf\u{00E9}";
        let violations = detect_in_string_with_policy(
            content,
            &PathBuf::from("test.rs"),
            true,
            Some(&ranges),
            &[],
        );

        // 'é' should NOT be flagged
        assert!(violations.iter().all(|v| v.code_point != 0x00E9));
    }

    #[test]
    fn test_context_snippet_long_line() {
        // Create a line longer than 120 chars with a violation in the middle
        let prefix = "a".repeat(80);
        let suffix = "b".repeat(80);
        let content = format!("{}\u{200B}{}", prefix, suffix);
        let violations = detect_in_string(&content, &PathBuf::from("test.rs"));

        assert_eq!(violations.len(), 1);
        // Context should be truncated with "..." markers
        assert!(violations[0].context.contains("..."));
    }

    #[test]
    fn test_is_allowed_by_policy_with_ranges() {
        let ranges = vec![UnicodeRange::new(0x0041, 0x005A)]; // A-Z only
        assert!(is_allowed_by_policy(0x0041, Some(&ranges))); // 'A'
        assert!(!is_allowed_by_policy(0x0061, Some(&ranges))); // 'a' not in range
    }

    #[test]
    fn test_is_allowed_by_policy_default() {
        // Default: safe ASCII (0x0009-0x000D, 0x0020-0x007E)
        assert!(is_allowed_by_policy(0x0020, None)); // space
        assert!(is_allowed_by_policy(0x007E, None)); // tilde
        assert!(is_allowed_by_policy(0x0009, None)); // tab
        assert!(!is_allowed_by_policy(0x0080, None)); // outside ASCII
        assert!(!is_allowed_by_policy(0x0000, None)); // NUL
    }

    #[test]
    fn test_context_snippet_short_line() {
        let line = "short";
        let snippet = context_snippet(line, 2);
        assert_eq!(snippet, "short"); // No truncation needed
    }
}
