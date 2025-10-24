//! Malicious Unicode pattern definitions

/// Category of malicious Unicode pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MaliciousCategory {
    ZeroWidth,
    BidiOverride,
    Homoglyph,
    ControlChar,
    NonStandard,
}

/// Severity level for violations
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Defines a specific Unicode pattern considered malicious
#[derive(Debug, Clone)]
pub struct MaliciousPattern {
    pub name: String,
    pub category: MaliciousCategory,
    pub code_points: Vec<u32>,
    pub severity: Severity,
    pub description: String,
}

impl MaliciousPattern {
    /// Check if a code point matches this pattern
    pub fn matches(&self, code_point: u32) -> bool {
        self.code_points.contains(&code_point)
    }
}

/// Get all built-in malicious patterns
pub fn get_malicious_patterns() -> Vec<MaliciousPattern> {
    vec![
        // Zero-width characters
        MaliciousPattern {
            name: "zero-width-space".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: vec![0x200B],
            severity: Severity::Error,
            description: "Zero-width space can hide malicious code".to_string(),
        },
        MaliciousPattern {
            name: "zero-width-non-joiner".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: vec![0x200C],
            severity: Severity::Error,
            description: "Zero-width non-joiner can hide malicious code".to_string(),
        },
        MaliciousPattern {
            name: "zero-width-joiner".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: vec![0x200D],
            severity: Severity::Error,
            description: "Zero-width joiner can hide malicious code".to_string(),
        },
        MaliciousPattern {
            name: "zero-width-no-break-space".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: vec![0xFEFF],
            severity: Severity::Error,
            description: "Zero-width no-break space (BOM) can hide malicious code".to_string(),
        },
        // Bidirectional override characters (Trojan Source)
        MaliciousPattern {
            name: "left-to-right-embedding".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x202A],
            severity: Severity::Error,
            description: "Bidirectional text control can alter code meaning".to_string(),
        },
        MaliciousPattern {
            name: "right-to-left-embedding".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x202B],
            severity: Severity::Error,
            description: "Bidirectional text control can alter code meaning".to_string(),
        },
        MaliciousPattern {
            name: "pop-directional-formatting".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x202C],
            severity: Severity::Error,
            description:
                "PDF character detected - bidirectional text control can alter code meaning"
                    .to_string(),
        },
        MaliciousPattern {
            name: "left-to-right-override".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x202D],
            severity: Severity::Error,
            description: "LRO character detected - can reverse code visually (Trojan Source)"
                .to_string(),
        },
        MaliciousPattern {
            name: "right-to-left-override".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x202E],
            severity: Severity::Error,
            description: "RLO character detected".to_string(),
        },
        // Unicode isolate characters (Trojan Source CVE-2021-42574)
        MaliciousPattern {
            name: "left-to-right-isolate".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x2066],
            severity: Severity::Error,
            description: "LRI character detected".to_string(),
        },
        MaliciousPattern {
            name: "right-to-left-isolate".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x2067],
            severity: Severity::Error,
            description: "RLI character detected".to_string(),
        },
        MaliciousPattern {
            name: "first-strong-isolate".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x2068],
            severity: Severity::Error,
            description: "FSI character detected".to_string(),
        },
        MaliciousPattern {
            name: "pop-directional-isolate".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x2069],
            severity: Severity::Error,
            description: "PDI character detected".to_string(),
        },
    ]
}

/// Check if a code point is in any malicious pattern
pub fn is_malicious(code_point: u32) -> Option<&'static str> {
    match code_point {
        0x200B => Some("zero-width-space"),
        0x200C => Some("zero-width-non-joiner"),
        0x200D => Some("zero-width-joiner"),
        0xFEFF => Some("zero-width-no-break-space"),
        0x202A => Some("left-to-right-embedding"),
        0x202B => Some("right-to-left-embedding"),
        0x202C => Some("pop-directional-formatting"),
        0x202D => Some("left-to-right-override"),
        0x202E => Some("right-to-left-override"),
        0x2066 => Some("left-to-right-isolate"),
        0x2067 => Some("right-to-left-isolate"),
        0x2068 => Some("first-strong-isolate"),
        0x2069 => Some("pop-directional-isolate"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_zero_width_space() {
        assert_eq!(is_malicious(0x200B), Some("zero-width-space"));
        assert_eq!(is_malicious(0x0041), None); // Normal 'A'
    }

    #[test]
    fn test_detect_bidi_override() {
        assert_eq!(is_malicious(0x202E), Some("right-to-left-override"));
        assert_eq!(is_malicious(0x202D), Some("left-to-right-override"));
    }

    #[test]
    fn test_malicious_patterns_coverage() {
        let patterns = get_malicious_patterns();

        // Should have zero-width patterns
        assert!(patterns.iter().any(|p| p.name == "zero-width-space"));
        assert!(patterns.iter().any(|p| p.name == "zero-width-non-joiner"));

        // Should have bidi patterns
        assert!(patterns.iter().any(|p| p.name == "right-to-left-override"));
        assert!(patterns.iter().any(|p| p.name == "left-to-right-override"));

        // All should be error severity for now
        assert!(patterns.iter().all(|p| p.severity == Severity::Error));
    }

    #[test]
    fn test_pattern_matches() {
        let pattern = MaliciousPattern {
            name: "test".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: vec![0x200B, 0x200C],
            severity: Severity::Error,
            description: "test".to_string(),
        };

        assert!(pattern.matches(0x200B));
        assert!(pattern.matches(0x200C));
        assert!(!pattern.matches(0x200D));
    }
}
