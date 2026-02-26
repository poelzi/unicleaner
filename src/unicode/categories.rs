//! Unicode character categorization

/// Check if a character is a bidirectional text control character
///
/// This includes characters that can be used in Trojan Source attacks:
/// - U+202A to U+202E: LRE, RLE, PDF, LRO, RLO
/// - U+2066 to U+2069: LRI, RLI, FSI, PDI
/// - U+200E to U+200F: LRM, RLM
/// - U+061C: ALM (Arabic Letter Mark)
pub fn is_bidi_control(c: char) -> bool {
    let code = c as u32;
    matches!(code,
        0x061C |          // ALM
        0x200E..=0x200F |  // LRM, RLM
        0x202A..=0x202E |  // LRE, RLE, PDF, LRO, RLO
        0x2066..=0x2069    // LRI, RLI, FSI, PDI
    )
}

/// Check if a character is invisible or zero-width
///
/// This includes:
/// - Zero-width spaces (ZWSP, ZWNJ, ZWJ, ZWNBSP/BOM)
/// - Combining diacritical marks
/// - Invisible separators and formatting characters
pub fn is_invisible(c: char) -> bool {
    let code = c as u32;
    matches!(code,
        // Zero-width characters
        0x200B |  // ZWSP
        0x200C |  // ZWNJ
        0x200D |  // ZWJ
        0xFEFF |  // ZWNBSP (BOM)
        // Combining Diacritical Marks
        0x0300..=0x036F |
        // Invisible separators
        0x00A0 |  // Non-breaking space
        0x1680 |  // Ogham Space Mark
        0x180E |  // Mongolian Vowel Separator
        0x2000..=0x200A |  // Various spaces
        0x2028 |  // Line Separator
        0x2029 |  // Paragraph Separator
        0x202F |  // Narrow No-Break Space
        0x205F |  // Medium Mathematical Space
        0x3000    // Ideographic Space
    )
}

/// Check if a character poses a homoglyph risk
///
/// This includes characters that look similar to ASCII but are from different
/// scripts:
/// - Cyrillic lookalikes (а, е, о, р, с, х, у, etc.)
/// - Greek lookalikes (α, ο, ρ, ν, etc.)
/// - Fullwidth ASCII variants
/// - Mathematical alphanumeric symbols
pub fn is_homoglyph_risk(c: char) -> bool {
    let code = c as u32;
    matches!(code,
        // Cyrillic homoglyphs
        0x0430 | 0x0435 | 0x043E | 0x0440 | 0x0441 | 0x0445 | 0x0443 |  // а е о р с х у
        0x0410 | 0x0415 | 0x041E | 0x0420 | 0x0421 | 0x0425 |           // А Е О Р С Х
        // Greek homoglyphs
        0x03B1 | 0x03BF | 0x03C1 | 0x03BD |  // α ο ρ ν
        0x0391 | 0x039F | 0x03A1 |           // Α Ο Ρ
        // Fullwidth forms
        0xFF01..=0xFF5E |
        // Mathematical alphanumeric symbols
        0x1D400..=0x1D7FF |
        // Letterlike Symbols that look like letters
        0x2102 | 0x210D | 0x2115 | 0x2119 | 0x211A | 0x211D | 0x2124
    )
}

/// Get the category name for a character
///
/// Returns the malicious category name if the character is malicious,
/// or "normal" for regular characters.
pub fn get_category(c: char) -> String {
    use crate::unicode::malicious::is_malicious;

    let code = c as u32;

    if let Some(pattern_name) = is_malicious(code) {
        // Determine category based on pattern name
        if pattern_name.contains("bidi")
            || pattern_name.contains("directional")
            || pattern_name.contains("override")
            || pattern_name.contains("embedding")
            || pattern_name.contains("isolate")
        {
            "bidi-control".to_string()
        } else if pattern_name.contains("zero-width")
            || pattern_name.contains("invisible")
            || pattern_name.contains("combining")
        {
            "invisible".to_string()
        } else if pattern_name.contains("homoglyph")
            || pattern_name.contains("cyrillic")
            || pattern_name.contains("greek")
            || pattern_name.contains("fullwidth")
            || pattern_name.contains("mathematical")
        {
            "homoglyph".to_string()
        } else {
            "malicious".to_string()
        }
    } else {
        "normal".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bidi_control() {
        // Test bidi override characters
        assert!(is_bidi_control('\u{202E}')); // RLO
        assert!(is_bidi_control('\u{202D}')); // LRO
        assert!(is_bidi_control('\u{202A}')); // LRE
        assert!(is_bidi_control('\u{202B}')); // RLE
        assert!(is_bidi_control('\u{202C}')); // PDF

        // Test bidi isolates
        assert!(is_bidi_control('\u{2066}')); // LRI
        assert!(is_bidi_control('\u{2067}')); // RLI
        assert!(is_bidi_control('\u{2068}')); // FSI
        assert!(is_bidi_control('\u{2069}')); // PDI

        // Test bidi marks
        assert!(is_bidi_control('\u{200E}')); // LRM
        assert!(is_bidi_control('\u{200F}')); // RLM
        assert!(is_bidi_control('\u{061C}')); // ALM

        // Test non-bidi characters
        assert!(!is_bidi_control('a'));
        assert!(!is_bidi_control('A'));
        assert!(!is_bidi_control('\u{200B}')); // ZWSP
    }

    #[test]
    fn test_is_invisible() {
        // Test zero-width characters
        assert!(is_invisible('\u{200B}')); // ZWSP
        assert!(is_invisible('\u{200C}')); // ZWNJ
        assert!(is_invisible('\u{200D}')); // ZWJ
        assert!(is_invisible('\u{FEFF}')); // BOM

        // Test combining marks
        assert!(is_invisible('\u{0300}')); // Combining grave accent
        assert!(is_invisible('\u{0301}')); // Combining acute accent

        // Test invisible separators
        assert!(is_invisible('\u{00A0}')); // NBSP
        assert!(is_invisible('\u{2000}')); // En quad
        assert!(is_invisible('\u{2028}')); // Line separator

        // Test visible characters
        assert!(!is_invisible('a'));
        assert!(!is_invisible(' ')); // Regular space
    }

    #[test]
    fn test_is_homoglyph_risk() {
        // Test Cyrillic homoglyphs
        assert!(is_homoglyph_risk('\u{0430}')); // Cyrillic а
        assert!(is_homoglyph_risk('\u{0435}')); // Cyrillic е
        assert!(is_homoglyph_risk('\u{043E}')); // Cyrillic о
        assert!(is_homoglyph_risk('\u{0440}')); // Cyrillic р

        // Test Greek homoglyphs
        assert!(is_homoglyph_risk('\u{03B1}')); // Greek α
        assert!(is_homoglyph_risk('\u{03BF}')); // Greek ο
        assert!(is_homoglyph_risk('\u{03C1}')); // Greek ρ

        // Test fullwidth
        assert!(is_homoglyph_risk('\u{FF21}')); // Fullwidth A
        assert!(is_homoglyph_risk('\u{FF41}')); // Fullwidth a

        // Test mathematical
        assert!(is_homoglyph_risk('\u{1D400}')); // Mathematical Bold A
        assert!(is_homoglyph_risk('\u{1D44E}')); // Mathematical Italic a

        // Test ASCII characters (should NOT be homoglyphs)
        assert!(!is_homoglyph_risk('a'));
        assert!(!is_homoglyph_risk('A'));
        assert!(!is_homoglyph_risk('0'));
    }

    #[test]
    fn test_get_category() {
        // Test bidi controls
        assert_eq!(get_category('\u{202E}'), "bidi-control"); // RLO

        // Test invisible characters
        assert_eq!(get_category('\u{200B}'), "invisible"); // ZWSP
        assert_eq!(get_category('\u{0300}'), "invisible"); // Combining mark

        // Test homoglyphs
        assert_eq!(get_category('\u{0430}'), "homoglyph"); // Cyrillic а
        assert_eq!(get_category('\u{FF41}'), "homoglyph"); // Fullwidth a

        // Test normal characters
        assert_eq!(get_category('a'), "normal");
        assert_eq!(get_category('Z'), "normal");
        assert_eq!(get_category('5'), "normal");
    }

    #[test]
    fn test_ascii_never_flagged() {
        // ASCII letters and digits should never be flagged as homoglyphs
        for c in b'a'..=b'z' {
            assert!(
                !is_homoglyph_risk(c as char),
                "ASCII '{}' should not be homoglyph",
                c as char
            );
        }
        for c in b'A'..=b'Z' {
            assert!(
                !is_homoglyph_risk(c as char),
                "ASCII '{}' should not be homoglyph",
                c as char
            );
        }
        for c in b'0'..=b'9' {
            assert!(
                !is_homoglyph_risk(c as char),
                "ASCII '{}' should not be homoglyph",
                c as char
            );
        }
    }
}
