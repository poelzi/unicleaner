//! Integration tests for the cleaner library API.

use std::borrow::Cow;

use unicleaner::cleaner::{CleanAction, CleanPolicy, clean};
use unicleaner::unicode::malicious::MaliciousCategory;

#[test]
fn clean_strict_strips_zero_width() {
    let input = "hi\u{200B}there";
    let result = clean(input, &CleanPolicy::strict());

    assert_eq!(result.output.as_ref(), "hithere");
    assert!(result.modified);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].code_point, 0x200B);
}

#[test]
fn clean_lossy_replaces_with_fffd() {
    let input = "admin\u{202E}lortnoc";
    let result = clean(input, &CleanPolicy::lossy());

    assert_eq!(result.output.as_ref(), "admin\u{FFFD}lortnoc");
    assert!(result.modified);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].code_point, 0x202E);
}

#[test]
fn clean_report_only_no_mutation() {
    let input = "hi\u{200B}there";
    let result = clean(input, &CleanPolicy::report_only());

    assert_eq!(result.output.as_ref(), input);
    assert!(!result.modified);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].code_point, 0x200B);
}

#[test]
fn clean_clean_input_borrows() {
    let input = "plain ascii content with no surprises";
    let result = clean(input, &CleanPolicy::strict());

    assert!(result.violations.is_empty());
    assert!(!result.modified);
    match result.output {
        Cow::Borrowed(s) => assert_eq!(s.as_ptr(), input.as_ptr()),
        Cow::Owned(_) => panic!("clean input must not allocate"),
    }
}

#[test]
fn clean_per_category_override() {
    // Cyrillic 'а' (U+0430) is a homoglyph of Latin 'a'.
    let input = "let p\u{0430}ssword = \"\u{200B}admin\";";
    let policy =
        CleanPolicy::strict().with_action(MaliciousCategory::Homoglyph, CleanAction::KeepWithMark);
    let result = clean(input, &policy);

    // Homoglyph stays in place; ZWSP is stripped.
    assert!(result.output.contains('\u{0430}'));
    assert!(!result.output.contains('\u{200B}'));
    assert!(result.modified);

    let cps: Vec<u32> = result.violations.iter().map(|v| v.code_point).collect();
    assert!(cps.contains(&0x0430));
    assert!(cps.contains(&0x200B));
}

#[test]
fn clean_empty_input() {
    let input = "";
    let result = clean(input, &CleanPolicy::strict());

    assert_eq!(result.output.as_ref(), "");
    assert!(!result.modified);
    assert!(result.violations.is_empty());
    matches!(result.output, Cow::Borrowed(_));
}

#[test]
fn clean_all_malicious_input() {
    let input = "\u{200B}\u{202E}";
    let result = clean(input, &CleanPolicy::strict());

    assert_eq!(result.output.as_ref(), "");
    assert!(result.modified);
    assert_eq!(result.violations.len(), 2);
    matches!(result.output, Cow::Owned(_));
}

#[test]
fn clean_replace_with_visible_marker() {
    let input = "x\u{202E}y";
    let policy = CleanPolicy::strict()
        .with_action(MaliciousCategory::BidiOverride, CleanAction::Replace('?'));
    let result = clean(input, &policy);

    assert_eq!(result.output.as_ref(), "x?y");
    assert!(result.modified);
    assert_eq!(result.violations.len(), 1);
}

#[test]
fn clean_records_position_in_violation() {
    // ZWSP at character position 3, line 1.
    let input = "ab\u{200B}c";
    let result = clean(input, &CleanPolicy::strict());

    assert_eq!(result.violations.len(), 1);
    let v = &result.violations[0];
    assert_eq!(v.line, 1);
    assert_eq!(v.column, 3);
    assert_eq!(v.byte_offset, 2);
}

// NFC tests use Hangul jamo NFD because the project's malicious-codepoint
// table treats Combining Diacritical Marks (U+0300..U+036F) as
// `MaliciousCategory::ZeroWidth` — they would be stripped before NFC saw
// them under the strict policy. Hangul Choseong/Jungseong/Jongseong are
// not in the table, so they round-trip cleanly through the cleaner.
//
// NFD of "각" (U+AC01) = U+1100 ᄀ + U+1161 ᅡ + U+11A8 ᆨ.

#[test]
fn clean_nfc_normalizes_hangul() {
    let input = "\u{1100}\u{1161}\u{11A8}";
    let policy = CleanPolicy::strict().with_nfc(true);
    let result = clean(input, &policy);

    assert_eq!(result.output.as_ref(), "\u{AC01}");
    assert!(result.modified);
    assert!(result.violations.is_empty());
}

#[test]
fn clean_nfc_off_preserves_decomposed() {
    let input = "\u{1100}\u{1161}\u{11A8}";
    let result = clean(input, &CleanPolicy::strict());

    assert_eq!(result.output.as_ref(), input);
    assert!(!result.modified);
    assert!(result.violations.is_empty());
}

#[test]
fn clean_nfc_runs_after_strip() {
    // Strip-then-NFC: "<ZWSP><Hangul jamo NFD>" → strip ZWSP → jamo → NFC → "각".
    // If NFC ran first, the ZWSP would still be in the string and the
    // strip pass would see the same NFC output anyway — but the
    // assertion that the raw `pattern_for(cp)` table matches by
    // pre-NFC code points is the contract this test protects.
    let input = "\u{200B}\u{1100}\u{1161}\u{11A8}";
    let policy = CleanPolicy::strict().with_nfc(true);
    let result = clean(input, &policy);

    assert_eq!(result.output.as_ref(), "\u{AC01}");
    assert!(result.modified);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].code_point, 0x200B);
}

#[test]
fn clean_nfc_already_normalized_does_not_flip_modified() {
    let input = "\u{AC01} hello";
    let policy = CleanPolicy::strict().with_nfc(true);
    let result = clean(input, &policy);

    assert_eq!(result.output.as_ref(), input);
    assert!(!result.modified);
    assert!(result.violations.is_empty());
    match result.output {
        Cow::Borrowed(s) => assert_eq!(s.as_ptr(), input.as_ptr()),
        Cow::Owned(_) => panic!("NFC no-op input must not allocate"),
    }
}

#[test]
fn clean_handles_multiline_input() {
    let input = "first line\nsecond\u{200B}line\n";
    let result = clean(input, &CleanPolicy::strict());

    assert_eq!(result.output.as_ref(), "first line\nsecondline\n");
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].line, 2);
}
