// Property-based tests for Unicode range boundary conditions (T042)

use proptest::prelude::*;
use unicleaner::unicode::ranges::UnicodeRange;

// Property: Range contains() should work at boundaries
proptest! {
    #[test]
    fn range_boundaries_correct(start in 0u32..0x10FFFF, len in 1u32..1000) {
        let end = start.saturating_add(len).min(0x10FFFF);
        let range = UnicodeRange::new(start, end);

        // Start should be in range
        if let Some(ch) = char::from_u32(start) {
            prop_assert!(range.contains(ch), "Range should contain start char U+{:04X}", start);
        }

        // End should be in range
        if let Some(ch) = char::from_u32(end) {
            prop_assert!(range.contains(ch), "Range should contain end char U+{:04X}", end);
        }

        // One before start should not be in range (if valid)
        if start > 0 {
            if let Some(ch) = char::from_u32(start - 1) {
                prop_assert!(!range.contains(ch), "Range should not contain char before start");
            }
        }

        // One after end should not be in range (if valid)
        if end < 0x10FFFF {
            if let Some(ch) = char::from_u32(end + 1) {
                prop_assert!(!range.contains(ch), "Range should not contain char after end");
            }
        }
    }
}

// Property: Range intersection should be symmetric
proptest! {
    #[test]
    fn range_intersection_symmetric(
        start1 in 0u32..0x10000,
        end1 in 0u32..0x10000,
        start2 in 0u32..0x10000,
        end2 in 0u32..0x10000
    ) {
        let range1 = UnicodeRange::new(start1.min(end1), start1.max(end1));
        let range2 = UnicodeRange::new(start2.min(end2), start2.max(end2));

        let intersects_12 = range1.intersects(&range2);
        let intersects_21 = range2.intersects(&range1);

        prop_assert_eq!(intersects_12, intersects_21, "Intersection should be symmetric");
    }
}

// Property: Character is in exactly one category
proptest! {
    #[test]
    fn char_in_one_category(c in any::<char>()) {
        use unicleaner::unicode::categories::get_category;

        let category = get_category(c);

        // Should have exactly one primary category
        prop_assert!(
            !category.is_empty(),
            "Character U+{:04X} should have a category", c as u32
        );
    }
}

// Property: Surrogate ranges are handled correctly
proptest! {
    #[test]
    fn surrogate_range_handling(code in 0xD800u32..=0xDFFF) {
        // Surrogate code points are not valid chars in Rust
        prop_assert!(
            char::from_u32(code).is_none(),
            "Surrogate code point U+{:04X} should not be valid char", code
        );
    }
}

// Property: All valid Unicode code points can be checked
proptest! {
    #[test]
    fn all_valid_codepoints_checkable(code in 0u32..=0x10FFFF) {
        // Skip surrogates
        if (0xD800..=0xDFFF).contains(&code) {
            return Ok(());
        }

        if let Some(ch) = char::from_u32(code) {
            // Should be able to check without panicking
            let _ = unicleaner::unicode::categories::is_invisible(ch);
            let _ = unicleaner::unicode::categories::is_bidi_control(ch);
            let _ = unicleaner::unicode::categories::is_homoglyph_risk(ch);
        }
    }
}
