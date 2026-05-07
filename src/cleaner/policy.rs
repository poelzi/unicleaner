//! `CleanPolicy` and `CleanAction` — operator configuration for the cleaner.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::unicode::malicious::MaliciousCategory;
use crate::unicode::ranges::UnicodeRange;

/// What to do with a code point that the policy considers malicious.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CleanAction {
    /// Remove the code point from the output entirely.
    Strip,
    /// Replace the code point with the given character.
    Replace(char),
    /// Keep the code point in place but record a `Violation`.
    KeepWithMark,
}

/// Operator-controlled configuration for [`crate::cleaner::clean`].
///
/// Cheap to construct; cheap to clone. Pick a preset
/// ([`CleanPolicy::strict`], [`CleanPolicy::lossy`],
/// [`CleanPolicy::report_only`]) and tune with the chained `with_*`
/// mutators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanPolicy {
    /// Used for any malicious code point with no per-category override.
    #[serde(default = "default_strip")]
    pub default_action: CleanAction,

    /// Per-category overrides keyed by [`MaliciousCategory`].
    #[serde(default)]
    pub per_category: BTreeMap<MaliciousCategory, CleanAction>,

    /// Extra code points to treat as malicious. Mirrors
    /// `Configuration.denied_code_points`.
    #[serde(default)]
    pub denied_code_points: Vec<u32>,

    /// When `deny_by_default` is true, code points outside these ranges are
    /// treated as malicious.
    #[serde(default)]
    pub allowed_ranges: Option<Vec<UnicodeRange>>,

    /// See `allowed_ranges`. Default `false`.
    #[serde(default)]
    pub deny_by_default: bool,

    /// Apply NFC normalization to the output after stripping. Default off.
    #[serde(default)]
    pub normalize_nfc: bool,
}

fn default_strip() -> CleanAction {
    CleanAction::Strip
}

impl CleanPolicy {
    /// Strip every malicious code point. NFC off, no overrides, no
    /// allow / deny lists. The safest universal default.
    pub fn strict() -> Self {
        Self {
            default_action: CleanAction::Strip,
            per_category: BTreeMap::new(),
            denied_code_points: Vec::new(),
            allowed_ranges: None,
            deny_by_default: false,
            normalize_nfc: false,
        }
    }

    /// Replace every malicious code point with `U+FFFD` REPLACEMENT
    /// CHARACTER. Useful for log lines where alignment matters.
    pub fn lossy() -> Self {
        Self {
            default_action: CleanAction::Replace('\u{FFFD}'),
            per_category: BTreeMap::new(),
            denied_code_points: Vec::new(),
            allowed_ranges: None,
            deny_by_default: false,
            normalize_nfc: false,
        }
    }

    /// Record violations but do not mutate the input. Drop-in replacement
    /// for `detect_in_string` callers that want the unified return type.
    pub fn report_only() -> Self {
        Self {
            default_action: CleanAction::KeepWithMark,
            per_category: BTreeMap::new(),
            denied_code_points: Vec::new(),
            allowed_ranges: None,
            deny_by_default: false,
            normalize_nfc: false,
        }
    }

    /// Set the action for a specific category, overriding `default_action`.
    pub fn with_action(mut self, category: MaliciousCategory, action: CleanAction) -> Self {
        self.per_category.insert(category, action);
        self
    }

    /// Replace the default action.
    pub fn with_default_action(mut self, action: CleanAction) -> Self {
        self.default_action = action;
        self
    }

    /// Toggle NFC normalization of the cleaned output.
    pub fn with_nfc(mut self, enable: bool) -> Self {
        self.normalize_nfc = enable;
        self
    }

    /// Add code points that should be treated as malicious in addition to
    /// the built-in table.
    pub fn with_denied(mut self, code_points: impl IntoIterator<Item = u32>) -> Self {
        self.denied_code_points.extend(code_points);
        self
    }

    /// Set the allowlist and toggle `deny_by_default`.
    pub fn with_allowed_ranges(mut self, ranges: Vec<UnicodeRange>, deny_by_default: bool) -> Self {
        self.allowed_ranges = Some(ranges);
        self.deny_by_default = deny_by_default;
        self
    }

    /// Resolve the action for a category: per-category override if present,
    /// otherwise the default. Crate-private; the cleaner's hot loop calls
    /// this once per matched code point.
    pub(crate) fn effective_action(&self, category: MaliciousCategory) -> CleanAction {
        self.per_category
            .get(&category)
            .copied()
            .unwrap_or(self.default_action)
    }
}

impl Default for CleanPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::malicious::MaliciousCategory;

    #[test]
    fn strict_defaults_to_strip() {
        let p = CleanPolicy::strict();
        assert_eq!(p.default_action, CleanAction::Strip);
        assert!(!p.normalize_nfc);
        assert!(p.per_category.is_empty());
        assert!(p.denied_code_points.is_empty());
        assert!(p.allowed_ranges.is_none());
        assert!(!p.deny_by_default);
    }

    #[test]
    fn lossy_defaults_to_replace_fffd() {
        let p = CleanPolicy::lossy();
        assert_eq!(p.default_action, CleanAction::Replace('\u{FFFD}'));
        assert!(!p.normalize_nfc);
    }

    #[test]
    fn report_only_defaults_to_keep_with_mark() {
        let p = CleanPolicy::report_only();
        assert_eq!(p.default_action, CleanAction::KeepWithMark);
        assert!(!p.normalize_nfc);
    }

    #[test]
    fn with_action_overrides_per_category() {
        let p = CleanPolicy::strict()
            .with_action(MaliciousCategory::BidiOverride, CleanAction::Replace('?'));
        assert_eq!(
            p.effective_action(MaliciousCategory::BidiOverride),
            CleanAction::Replace('?')
        );
    }

    #[test]
    fn effective_action_falls_through_to_default() {
        let p = CleanPolicy::lossy();
        // No override for ZeroWidth → default (Replace FFFD) wins.
        assert_eq!(
            p.effective_action(MaliciousCategory::ZeroWidth),
            CleanAction::Replace('\u{FFFD}')
        );
    }

    #[test]
    fn presets_are_equal_to_themselves() {
        assert_eq!(CleanPolicy::strict(), CleanPolicy::strict());
        assert_eq!(CleanPolicy::lossy(), CleanPolicy::lossy());
        assert_eq!(CleanPolicy::report_only(), CleanPolicy::report_only());
    }

    #[test]
    fn presets_are_distinct() {
        assert_ne!(CleanPolicy::strict(), CleanPolicy::lossy());
        assert_ne!(CleanPolicy::strict(), CleanPolicy::report_only());
    }

    #[test]
    fn json_round_trip_preserves_equality() {
        let p = CleanPolicy::strict()
            .with_action(MaliciousCategory::Homoglyph, CleanAction::KeepWithMark)
            .with_action(MaliciousCategory::BidiOverride, CleanAction::Replace('?'))
            .with_nfc(true)
            .with_denied([0x2028u32, 0x2029]);

        let json = serde_json::to_string(&p).unwrap();
        let back: CleanPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn with_nfc_toggles_flag() {
        let p = CleanPolicy::strict().with_nfc(true);
        assert!(p.normalize_nfc);
        let p = p.with_nfc(false);
        assert!(!p.normalize_nfc);
    }

    #[test]
    fn with_denied_extends_list() {
        let p = CleanPolicy::strict().with_denied([0x2028u32, 0x2029]);
        assert_eq!(p.denied_code_points, vec![0x2028, 0x2029]);
    }

    #[test]
    fn with_default_action_replaces_default() {
        let p = CleanPolicy::strict().with_default_action(CleanAction::KeepWithMark);
        assert_eq!(p.default_action, CleanAction::KeepWithMark);
    }
}
