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

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Static cache of malicious patterns (initialized once)
static MALICIOUS_PATTERNS: Lazy<Vec<MaliciousPattern>> = Lazy::new(build_malicious_patterns);

static MALICIOUS_LOOKUP: Lazy<HashMap<u32, usize>> = Lazy::new(|| {
    let mut map = HashMap::new();

    for (idx, pattern) in MALICIOUS_PATTERNS.iter().enumerate() {
        for &code_point in &pattern.code_points {
            // First match wins; patterns should be non-overlapping.
            map.entry(code_point).or_insert(idx);
        }
    }

    map
});

/// Get all built-in malicious patterns (cached, zero allocation after first call)
pub fn get_malicious_patterns() -> &'static Vec<MaliciousPattern> {
    &MALICIOUS_PATTERNS
}

/// Get the malicious pattern for a code point, if any.
pub fn pattern_for(code_point: u32) -> Option<&'static MaliciousPattern> {
    let idx = *MALICIOUS_LOOKUP.get(&code_point)?;
    MALICIOUS_PATTERNS.get(idx)
}

fn build_malicious_patterns() -> Vec<MaliciousPattern> {
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
            description: "PDF character detected - bidirectional text control can alter code \
                          meaning"
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
        // Bidirectional marks (Trojan Source adjacent)
        MaliciousPattern {
            name: "bidi-marks".to_string(),
            category: MaliciousCategory::BidiOverride,
            code_points: vec![0x061C, 0x200E, 0x200F],
            severity: Severity::Error,
            description: "Bidirectional mark detected - can manipulate source display".to_string(),
        },
        // Homoglyph patterns - Cyrillic lookalikes
        MaliciousPattern {
            name: "cyrillic-homoglyphs".to_string(),
            category: MaliciousCategory::Homoglyph,
            code_points: vec![
                0x0430, // а - Cyrillic Small Letter A (looks like Latin a)
                0x0435, // е - Cyrillic Small Letter Ie (looks like Latin e)
                0x043E, // о - Cyrillic Small Letter O (looks like Latin o)
                0x0440, // р - Cyrillic Small Letter Er (looks like Latin p)
                0x0441, // с - Cyrillic Small Letter Es (looks like Latin c)
                0x0445, // х - Cyrillic Small Letter Ha (looks like Latin x)
                0x0443, // у - Cyrillic Small Letter U (looks like Latin y)
                0x0410, // А - Cyrillic Capital Letter A
                0x0415, // Е - Cyrillic Capital Letter Ie
                0x041E, // О - Cyrillic Capital Letter O
                0x0420, // Р - Cyrillic Capital Letter Er
                0x0421, // С - Cyrillic Capital Letter Es
                0x0425, // Х - Cyrillic Capital Letter Ha
            ],
            severity: Severity::Error,
            description: "Cyrillic homoglyph detected - confusable with Latin characters"
                .to_string(),
        },
        // Greek homoglyphs
        MaliciousPattern {
            name: "greek-homoglyphs".to_string(),
            category: MaliciousCategory::Homoglyph,
            code_points: vec![
                0x03B1, // α - Greek Small Letter Alpha (looks like Latin a)
                0x03BF, // ο - Greek Small Letter Omicron (looks like Latin o)
                0x03C1, // ρ - Greek Small Letter Rho (looks like Latin p)
                0x03BD, // ν - Greek Small Letter Nu (looks like Latin v)
                0x0391, // Α - Greek Capital Letter Alpha
                0x039F, // Ο - Greek Capital Letter Omicron
                0x03A1, // Ρ - Greek Capital Letter Rho
            ],
            severity: Severity::Error,
            description: "Greek homoglyph detected - confusable with Latin characters".to_string(),
        },
        // Fullwidth forms
        MaliciousPattern {
            name: "fullwidth-forms".to_string(),
            category: MaliciousCategory::Homoglyph,
            code_points: (0xFF01..=0xFF5E).collect(), // Fullwidth ASCII variants
            severity: Severity::Warning,
            description: "Fullwidth character detected - confusable with ASCII".to_string(),
        },
        // Mathematical Alphanumeric Symbols
        MaliciousPattern {
            name: "mathematical-alphanumeric".to_string(),
            category: MaliciousCategory::Homoglyph,
            code_points: {
                let mut points = Vec::new();
                // Mathematical Bold (U+1D400–U+1D433)
                points.extend(0x1D400..=0x1D433);
                // Mathematical Italic (U+1D434–U+1D467)
                points.extend(0x1D434..=0x1D467);
                // Mathematical Bold Italic (U+1D468–U+1D49B)
                points.extend(0x1D468..=0x1D49B);
                // Mathematical Script (U+1D49C–U+1D4CF)
                points.extend(0x1D49C..=0x1D4CF);
                // Mathematical Bold Script (U+1D4D0–U+1D503)
                points.extend(0x1D4D0..=0x1D503);
                // Mathematical Fraktur (U+1D504–U+1D537)
                points.extend(0x1D504..=0x1D537);
                // Mathematical Double-Struck (U+1D538–U+1D56B)
                points.extend(0x1D538..=0x1D56B);
                // Mathematical Bold Fraktur (U+1D56C–U+1D59F)
                points.extend(0x1D56C..=0x1D59F);
                // Mathematical Sans-Serif (U+1D5A0–U+1D5D3)
                points.extend(0x1D5A0..=0x1D5D3);
                // Mathematical Sans-Serif Bold (U+1D5D4–U+1D607)
                points.extend(0x1D5D4..=0x1D607);
                // Mathematical Sans-Serif Italic (U+1D608–U+1D63B)
                points.extend(0x1D608..=0x1D63B);
                // Mathematical Sans-Serif Bold Italic (U+1D63C–U+1D66F)
                points.extend(0x1D63C..=0x1D66F);
                // Mathematical Monospace (U+1D670–U+1D6A3)
                points.extend(0x1D670..=0x1D6A3);
                // Additional mathematical alphanumerics (U+1D6A4–U+1D7FF)
                points.extend(0x1D6A4..=0x1D7FF);
                // Also include Letterlike Symbols that look like regular letters
                points.push(0x2102); // ℂ
                points.push(0x210D); // ℍ
                points.push(0x2115); // ℕ
                points.push(0x2119); // ℙ
                points.push(0x211A); // ℚ
                points.push(0x211D); // ℝ
                points.push(0x2124); // ℤ
                points
            },
            severity: Severity::Warning,
            description: "Mathematical alphanumeric character detected".to_string(),
        },
        // Combining characters that can be stacked
        MaliciousPattern {
            name: "combining-characters".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: (0x0300..=0x036F).collect(), // Combining Diacritical Marks
            severity: Severity::Warning,
            description: "Combining character detected - can be used for stacking attacks"
                .to_string(),
        },
        // Invisible separators and formatting characters
        MaliciousPattern {
            name: "invisible-separators".to_string(),
            category: MaliciousCategory::ZeroWidth,
            code_points: vec![
                0x00A0, // Non-breaking space
                0x1680, // Ogham Space Mark
                0x180E, // Mongolian Vowel Separator
                0x2000, 0x2001, 0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009,
                0x200A, // Various space characters
                0x2028, // Line Separator
                0x2029, // Paragraph Separator
                0x202F, // Narrow No-Break Space
                0x205F, // Medium Mathematical Space
                0x3000, // Ideographic Space (fullwidth space)
            ],
            severity: Severity::Warning,
            description: "Invisible separator detected".to_string(),
        },
    ]
}

/// Check if a code point is in any malicious pattern
pub fn is_malicious(code_point: u32) -> Option<&'static str> {
    pattern_for(code_point).map(|p| p.name.as_str())
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

        // Should have both Error and Warning severity patterns
        assert!(patterns.iter().any(|p| p.severity == Severity::Error));
        assert!(patterns.iter().any(|p| p.severity == Severity::Warning));
    }

    #[test]
    fn test_malicious_patterns_static() {
        // T058: Verify get_malicious_patterns() returns the same pointer on repeated calls
        // (i.e., it's cached via once_cell::sync::Lazy, not rebuilt each time)
        let first = get_malicious_patterns() as *const Vec<MaliciousPattern>;
        let second = get_malicious_patterns() as *const Vec<MaliciousPattern>;
        assert_eq!(
            first, second,
            "get_malicious_patterns() should return the same static reference"
        );
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
