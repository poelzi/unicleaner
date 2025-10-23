//! Unicode detection logic

use crate::report::Violation;
use crate::unicode::malicious::{
    get_malicious_patterns, is_malicious, MaliciousCategory, Severity,
};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

/// Detect malicious Unicode in a string
pub fn detect_in_string(content: &str, file_path: &Path) -> Vec<Violation> {
    let patterns = get_malicious_patterns();
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for (col_num, grapheme) in line.grapheme_indices(true) {
            for ch in grapheme.chars() {
                let code_point = ch as u32;

                if let Some(pattern_name) = is_malicious(code_point) {
                    // Find the full pattern details
                    if let Some(pattern) = patterns.iter().find(|p| p.name == pattern_name) {
                        let violation = Violation::new(
                            file_path.to_path_buf(),
                            line_num + 1, // 1-indexed
                            col_num + 1,  // 1-indexed
                            code_point,
                            pattern.name.clone(),
                            pattern.category,
                            pattern.severity,
                            pattern.description.clone(),
                        )
                        .with_context(line.to_string());

                        violations.push(violation);
                    }
                }
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_zero_width_space() {
        let content = "let user​name = \"admin\";"; // Contains U+200B
        let violations = detect_in_string(content, &PathBuf::from("test.rs"));

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code_point, 0x200B);
        assert_eq!(violations[0].pattern_name, "zero-width-space");
    }

    #[test]
    fn test_detect_bidi_override() {
        // Construct string with U+202E (right-to-left override) to avoid compiler warning
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
}
