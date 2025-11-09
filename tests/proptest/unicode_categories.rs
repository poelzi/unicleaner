// Property-based tests for Unicode category invariants (T041)
// Tests that Unicode character categorization is consistent

use proptest::prelude::*;
use unicleaner::unicode::categories::{is_bidi_control, is_homoglyph_risk, is_invisible};

// Property: All bidi control characters should be in the U+202x or U+206x range
proptest! {
    #[test]
    fn bidi_controls_in_expected_range(c in any::<char>()) {
        if is_bidi_control(c) {
            let code = c as u32;
            prop_assert!(
                (0x202A..=0x202E).contains(&code) ||  // RLE, LRE, PDF, RLO, LRO
                (0x2066..=0x2069).contains(&code) ||  // LRI, RLI, FSI, PDI
                (0x200E..=0x200F).contains(&code),    // LRM, RLM
                "Bidi control char U+{:04X} not in expected range", code
            );
        }
    }
}

// Property: Zero-width characters are truly zero-width
proptest! {
    #[test]
    fn zero_width_chars_identified(c in any::<char>()) {
        let code = c as u32;
        let known_zero_width = matches!(code,
            0x200B | // ZWSP
            0x200C | // ZWNJ
            0x200D | // ZWJ
            0xFEFF   // BOM/ZWNBSP
        );

        if known_zero_width {
            prop_assert!(
                is_invisible(c),
                "Zero-width char U+{:04X} should be marked invisible", code
            );
        }
    }
}

// Property: ASCII characters should never be flagged as homoglyph risks
proptest! {
    #[test]
    fn ascii_not_homoglyph_risk(c in 0u8..128u8) {
        let ch = c as char;
        prop_assert!(
            !is_homoglyph_risk(ch),
            "ASCII character '{}' (U+{:04X}) should not be homoglyph risk",
            ch, c as u32
        );
    }
}

// Property: Cyrillic lookalikes are flagged
proptest! {
    #[test]
    fn cyrillic_lookalikes_detected(c in any::<char>()) {
        let code = c as u32;
        // Common Cyrillic homoglyphs
        let known_homoglyphs = matches!(code,
            0x0430 | // а (Cyrillic 'a')
            0x0435 | // е (Cyrillic 'e')
            0x043E | // о (Cyrillic 'o')
            0x0440 | // р (Cyrillic 'p')
            0x0441 | // с (Cyrillic 'c')
            0x0445   // х (Cyrillic 'x')
        );

        if known_homoglyphs {
            prop_assert!(
                is_homoglyph_risk(c),
                "Cyrillic homoglyph U+{:04X} should be detected", code
            );
        }
    }
}

// Property: Character category functions never panic
proptest! {
    #[test]
    fn category_functions_dont_panic(c in any::<char>()) {
        // These should never panic regardless of input
        let _ = is_invisible(c);
        let _ = is_bidi_control(c);
        let _ = is_homoglyph_risk(c);
    }
}
