//! Cleaner API — sanitize strings against malicious Unicode patterns.
//!
//! Pair the [`clean`] entry point with a [`CleanPolicy`] (try
//! [`CleanPolicy::strict`] / [`CleanPolicy::lossy`] /
//! [`CleanPolicy::report_only`]). The result bundles the (possibly
//! mutated) output, the violations the policy reacted to, and a
//! `modified` flag.
//!
//! # Examples
//!
//! ```
//! use std::borrow::Cow;
//! use unicleaner::cleaner::{clean, CleanPolicy};
//!
//! let result = clean("hi\u{200B}there", &CleanPolicy::strict());
//! assert_eq!(result.output.as_ref(), "hithere");
//! assert!(result.modified);
//! assert_eq!(result.violations.len(), 1);
//!
//! // Clean input borrows; no allocation.
//! let result = clean("plain ascii", &CleanPolicy::strict());
//! assert!(matches!(result.output, Cow::Borrowed(_)));
//! ```

pub mod policy;

pub use policy::{CleanAction, CleanPolicy};

use std::borrow::Cow;
use std::path::PathBuf;

use crate::report::Violation;
use crate::unicode::malicious::{MaliciousCategory, MaliciousPattern, Severity, pattern_for};
use crate::unicode::ranges::UnicodeRange;

/// What [`clean`] returns: the (possibly cleaned) string, the violations
/// the policy reacted to, and a `modified` flag.
///
/// The `'a` lifetime ties [`Self::output`] to the original input string
/// when the cleaner did not need to allocate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanResult<'a> {
    /// The cleaned output. `Cow::Borrowed` when nothing changed, owned
    /// otherwise.
    pub output: Cow<'a, str>,
    /// Every code point the policy acted on, in input order.
    pub violations: Vec<Violation>,
    /// `true` iff the cleaner mutated the input. `false` for
    /// [`CleanPolicy::report_only`] regardless of how many violations were
    /// recorded, unless `normalize_nfc` is enabled and the input was not
    /// already in NFC form.
    pub modified: bool,
}

/// Synthetic file path for the `Violation` records emitted from in-process
/// callers. Mirrors the `<inline>` convention used elsewhere.
const INLINE_PATH: &str = "<inline>";

/// Sanitize an input string against the given policy.
///
/// Returns a [`CleanResult`] whose `output` borrows from `input` whenever
/// no mutation occurred and NFC normalization is disabled (or the input
/// was already in NFC).
pub fn clean<'a>(input: &'a str, policy: &CleanPolicy) -> CleanResult<'a> {
    // Single pre-scan covers both the no-NFC and NFC-already-normalized
    // fast paths. NFC's `is_nfc` check is only invoked when the policy
    // asks for it.
    let dirty = needs_mutation(input, policy);
    if !dirty && (!policy.normalize_nfc || unicode_normalization::is_nfc(input)) {
        return CleanResult {
            output: Cow::Borrowed(input),
            violations: Vec::new(),
            modified: false,
        };
    }

    let mut output = String::with_capacity(input.len());
    let mut violations = Vec::new();

    for (line_idx, line) in input.split_inclusive('\n').enumerate() {
        // Strip the trailing '\n' (if any) for column accounting; we still
        // need to copy the newline byte through verbatim to the output.
        let (body, has_newline) = match line.strip_suffix('\n') {
            Some(rest) => (rest, true),
            None => (line, false),
        };

        for (column, (byte_offset, ch)) in (1usize..).zip(body.char_indices()) {
            let cp = ch as u32;
            match decide_action(cp, policy) {
                Some(MatchedAction { pattern, action }) => {
                    let v = Violation::new(
                        PathBuf::from(INLINE_PATH),
                        line_idx + 1,
                        column,
                        byte_offset,
                        cp,
                        pattern.name.clone(),
                        pattern.category,
                        pattern.severity,
                        pattern.description.clone(),
                    );
                    violations.push(v);
                    match action {
                        CleanAction::Strip => {}
                        CleanAction::Replace(rep) => output.push(rep),
                        CleanAction::KeepWithMark => output.push(ch),
                    }
                }
                None => output.push(ch),
            }
        }

        if has_newline {
            output.push('\n');
        }
    }

    let mutated_chars = output.as_str() != input;

    // NFC pass runs after stripping so the malicious-codepoint table's
    // raw `u32` lookups stay valid (research.md Decision 3). We already
    // know `is_nfc(input)` was either irrelevant or false to reach this
    // branch; if `is_nfc(&output)` is also false, the `nfc()` collect is
    // guaranteed to differ.
    let (final_output, nfc_changed) = if policy.normalize_nfc {
        use unicode_normalization::UnicodeNormalization;
        if unicode_normalization::is_nfc(&output) {
            (output, false)
        } else {
            (output.nfc().collect::<String>(), true)
        }
    } else {
        (output, false)
    };

    let modified = mutated_chars || nfc_changed;
    let output = if modified {
        Cow::Owned(final_output)
    } else {
        Cow::Borrowed(input)
    };

    CleanResult {
        output,
        violations,
        modified,
    }
}

/// What [`decide_action`] returns when the policy reacts to a code point.
struct MatchedAction {
    /// The malicious-codepoint entry. For `denied_code_points` and
    /// `deny_by_default` matches this is a synthetic, leaked pattern so
    /// the caller can treat all matches uniformly.
    pattern: &'static MaliciousPattern,
    /// What the policy says to do with this code point.
    action: CleanAction,
}

/// Resolve the action the policy would take for `cp`.
///
/// Order of precedence: built-in malicious table → caller's
/// `denied_code_points` → `deny_by_default` rule. Mirrors
/// `detect_in_string_with_policy` so the cleaner walks the same code
/// points as the detector.
fn decide_action(cp: u32, policy: &CleanPolicy) -> Option<MatchedAction> {
    if let Some(pattern) = pattern_for(cp) {
        return Some(MatchedAction {
            pattern,
            action: policy.effective_action(pattern.category),
        });
    }

    if policy.denied_code_points.contains(&cp) {
        return Some(MatchedAction {
            pattern: explicitly_denied_pattern(),
            action: policy.effective_action(MaliciousCategory::NonStandard),
        });
    }

    if policy.deny_by_default && !is_allowed(cp, policy.allowed_ranges.as_deref()) {
        return Some(MatchedAction {
            pattern: disallowed_pattern(),
            action: policy.effective_action(MaliciousCategory::NonStandard),
        });
    }

    None
}

/// Synthetic `MaliciousPattern` used for `denied_code_points` matches so
/// the cleaner's hot loop can return `&'static MaliciousPattern`
/// uniformly.
fn explicitly_denied_pattern() -> &'static MaliciousPattern {
    use std::sync::OnceLock;
    static PATTERN: OnceLock<MaliciousPattern> = OnceLock::new();
    PATTERN.get_or_init(|| MaliciousPattern {
        name: "explicitly-denied".to_string(),
        category: MaliciousCategory::NonStandard,
        code_points: Vec::new(),
        severity: Severity::Error,
        description: "Code point is explicitly denied by configuration".to_string(),
    })
}

/// Synthetic `MaliciousPattern` for `deny_by_default` matches.
fn disallowed_pattern() -> &'static MaliciousPattern {
    use std::sync::OnceLock;
    static PATTERN: OnceLock<MaliciousPattern> = OnceLock::new();
    PATTERN.get_or_init(|| MaliciousPattern {
        name: "disallowed-code-point".to_string(),
        category: MaliciousCategory::NonStandard,
        code_points: Vec::new(),
        severity: Severity::Error,
        description: "Code point is outside the configured allowlist (deny-by-default)".to_string(),
    })
}

/// Cheap pre-scan: return `true` iff at least one code point in `input`
/// would trigger an action from `policy`.
fn needs_mutation(input: &str, policy: &CleanPolicy) -> bool {
    for ch in input.chars() {
        let cp = ch as u32;
        if pattern_for(cp).is_some() {
            return true;
        }
        if policy.denied_code_points.contains(&cp) {
            return true;
        }
        if policy.deny_by_default && !is_allowed(cp, policy.allowed_ranges.as_deref()) {
            return true;
        }
    }
    false
}

fn is_allowed(cp: u32, allowed_ranges: Option<&[UnicodeRange]>) -> bool {
    if let Some(ranges) = allowed_ranges {
        return ranges.iter().any(|r| r.contains(cp));
    }
    matches!(cp, 0x0009..=0x000D | 0x0020..=0x007E)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_mutation_empty() {
        assert!(!needs_mutation("", &CleanPolicy::strict()));
    }

    #[test]
    fn needs_mutation_clean_ascii() {
        assert!(!needs_mutation("plain ascii", &CleanPolicy::strict()));
    }

    #[test]
    fn needs_mutation_with_zwsp() {
        assert!(needs_mutation("hi\u{200B}there", &CleanPolicy::strict()));
    }

    #[test]
    fn needs_mutation_respects_denied_list() {
        let policy = CleanPolicy::strict().with_denied([0x00E9u32]);
        assert!(needs_mutation("caf\u{00E9}", &policy));
    }

    #[test]
    fn decide_action_returns_none_for_safe_codepoint() {
        let r = decide_action(b'a' as u32, &CleanPolicy::strict());
        assert!(r.is_none());
    }

    #[test]
    fn decide_action_uses_per_category_override() {
        let policy = CleanPolicy::strict()
            .with_action(MaliciousCategory::ZeroWidth, CleanAction::Replace('?'));
        let r = decide_action(0x200B, &policy).expect("ZWSP is malicious");
        assert_eq!(r.action, CleanAction::Replace('?'));
    }

    #[test]
    fn decide_action_falls_through_to_default() {
        let policy = CleanPolicy::lossy();
        let r = decide_action(0x200B, &policy).expect("ZWSP is malicious");
        assert_eq!(r.action, CleanAction::Replace('\u{FFFD}'));
    }

    #[test]
    fn decide_action_returns_synthetic_pattern_for_denied_list() {
        let policy = CleanPolicy::strict().with_denied([0x00E9u32]);
        let r = decide_action(0x00E9, &policy).expect("denied list match");
        assert_eq!(r.pattern.name, "explicitly-denied");
        assert_eq!(r.pattern.category, MaliciousCategory::NonStandard);
    }

    #[test]
    fn invariant_unmodified_means_byte_equal_output() {
        let input = "plain ascii";
        let r = clean(input, &CleanPolicy::strict());
        assert!(!r.modified);
        assert_eq!(r.output.as_ref(), input);
    }

    #[test]
    fn invariant_clean_input_borrows() {
        let input = "plain ascii";
        let r = clean(input, &CleanPolicy::strict());
        assert!(matches!(r.output, Cow::Borrowed(_)));
    }

    #[test]
    fn nfc_already_normalized_input_borrows_when_clean() {
        let input = "plain ascii";
        let r = clean(input, &CleanPolicy::strict().with_nfc(true));
        assert!(matches!(r.output, Cow::Borrowed(_)));
        assert!(!r.modified);
    }

    #[test]
    fn clean_handles_explicitly_denied_codepoint() {
        let policy = CleanPolicy::strict().with_denied([0x00E9u32]);
        let r = clean("caf\u{00E9}", &policy);
        assert_eq!(r.output.as_ref(), "caf");
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].code_point, 0x00E9);
        assert_eq!(r.violations[0].pattern_name, "explicitly-denied");
    }

    #[test]
    fn clean_deny_by_default_strips_outside_ranges() {
        let policy = CleanPolicy::strict()
            .with_allowed_ranges(vec![UnicodeRange::new(0x0020, 0x007E)], true);
        let r = clean("hello\u{00E9}world", &policy);
        assert!(!r.output.contains('\u{00E9}'));
        assert!(r.violations.iter().any(|v| v.code_point == 0x00E9));
        assert_eq!(r.violations[0].pattern_name, "disallowed-code-point");
    }

    #[test]
    fn clean_deny_by_default_default_allowlist_passes_ascii() {
        let policy = CleanPolicy::strict()
            // No ranges configured → fallback safe-ASCII allowlist applies.
            .with_allowed_ranges(Vec::new(), true);
        // Hack: with_allowed_ranges sets allowed_ranges = Some(empty),
        // not None, so the fallback branch isn't hit. Use deny_by_default
        // = true with allowed_ranges = None directly.
        let policy = CleanPolicy {
            deny_by_default: true,
            allowed_ranges: None,
            ..policy
        };
        let r = clean("hello world", &policy);
        // All printable ASCII → no violations.
        assert!(r.violations.is_empty());
        assert!(matches!(r.output, Cow::Borrowed(_)));
    }

    #[test]
    fn needs_mutation_deny_by_default_with_ranges() {
        let policy = CleanPolicy::strict()
            .with_allowed_ranges(vec![UnicodeRange::new(0x0061, 0x007A)], true);
        // 'A' (0x0041) outside allowed [a-z] range → flagged.
        assert!(needs_mutation("A", &policy));
        assert!(!needs_mutation("abc", &policy));
    }

    #[test]
    fn explicitly_denied_pattern_is_singleton() {
        let p1 = explicitly_denied_pattern();
        let p2 = explicitly_denied_pattern();
        assert!(std::ptr::eq(p1, p2));
        assert_eq!(p1.name, "explicitly-denied");
        assert_eq!(p1.severity, Severity::Error);
    }

    #[test]
    fn disallowed_pattern_is_singleton() {
        let p1 = disallowed_pattern();
        let p2 = disallowed_pattern();
        assert!(std::ptr::eq(p1, p2));
        assert_eq!(p1.name, "disallowed-code-point");
        assert_eq!(p1.severity, Severity::Error);
    }

    #[test]
    fn is_allowed_falls_back_to_safe_ascii_when_no_ranges() {
        assert!(is_allowed(0x0041, None)); // 'A'
        assert!(!is_allowed(0x00E9, None)); // 'é'
    }

    #[test]
    fn is_allowed_uses_provided_ranges() {
        let ranges = [UnicodeRange::new(0x0061, 0x007A)];
        assert!(is_allowed(0x0061, Some(&ranges)));
        assert!(!is_allowed(0x0041, Some(&ranges)));
    }
}
