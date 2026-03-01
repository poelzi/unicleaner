//! Configuration module for TOML config parsing and rules

pub mod parser;
pub mod presets;
pub mod rules;
pub mod validation;

use crate::unicode::ranges::UnicodeRange;
use rules::FileRule;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Global deny-by-default setting
    pub deny_by_default: bool,

    /// Language-specific presets
    pub language_presets: HashMap<String, String>,

    /// File-specific rules
    pub file_rules: Vec<FileRule>,

    /// Configuration file path (for reference)
    pub config_path: PathBuf,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            deny_by_default: true,
            language_presets: HashMap::new(),
            file_rules: Vec::new(),
            config_path: PathBuf::from("unicleaner.toml"),
        }
    }
}

impl Configuration {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        parser::load_config(path)
    }

    /// Merge this configuration with another (other takes precedence)
    pub fn merge(&mut self, other: Configuration) {
        // User config overrides defaults
        if !other.deny_by_default {
            self.deny_by_default = other.deny_by_default;
        }

        // Merge language presets
        for (lang, preset) in other.language_presets {
            self.language_presets.insert(lang, preset);
        }

        // Append file rules (will be sorted by priority later)
        self.file_rules.extend(other.file_rules);

        // Sort rules by priority
        rules::sort_rules_by_priority(&mut self.file_rules);
    }

    /// Get allowed ranges for a specific file
    pub fn get_allowed_ranges(&self, file_path: &Path) -> Option<Vec<UnicodeRange>> {
        // Check if any file-specific rule matches
        if let Some(rule) = rules::find_matching_rule(&self.file_rules, file_path) {
            if !rule.allowed_ranges.is_empty() {
                return Some(rule.allowed_ranges.clone());
            }
        }

        // Check for language preset based on file extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            if let Some(preset_name) = self.language_presets.get(ext) {
                if let Some(preset) = presets::get_preset(preset_name) {
                    return Some(preset.allowed_ranges);
                }
            }
        }

        // No specific configuration found
        None
    }

    /// Check if a code point is allowed in a specific file
    pub fn is_code_point_allowed(&self, file_path: &Path, code_point: u32) -> bool {
        // Allow-by-default: only explicit denies matter.
        if !self.deny_by_default {
            if let Some(rule) = rules::find_matching_rule(&self.file_rules, file_path) {
                if rule.denied_code_points.contains(&code_point) {
                    return false;
                }
            }
            return true;
        }

        // Deny-by-default: apply explicit denies, then allowlist.
        if let Some(rule) = rules::find_matching_rule(&self.file_rules, file_path) {
            if rule.denied_code_points.contains(&code_point) {
                return false;
            }

            if !rule.allowed_ranges.is_empty() {
                return rule.allowed_ranges.iter().any(|r| r.contains(code_point));
            }
        }

        // Check language presets
        if let Some(ranges) = self.get_allowed_ranges(file_path) {
            return ranges.iter().any(|range| range.contains(code_point));
        }

        // Fall back to global default allowlist (safe ASCII).
        matches!(code_point, 0x0009..=0x000D | 0x0020..=0x007E)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_default_with_user_config() {
        let mut default_config = Configuration::default();
        let user_config = Configuration {
            deny_by_default: false,
            ..Default::default()
        };

        default_config.merge(user_config);
        assert!(
            !default_config.deny_by_default,
            "User config should override default"
        );
    }

    #[test]
    fn test_merge_preserves_user_rules() {
        let mut config1 = Configuration::default();
        let mut config2 = Configuration::default();

        // Add a rule to config2
        let rule = rules::FileRule::new("*.rs").expect("Valid pattern");
        config2.file_rules.push(rule);

        config1.merge(config2);
        assert_eq!(config1.file_rules.len(), 1, "Should have merged the rule");
    }

    #[test]
    fn test_merge_combines_presets() {
        let mut config1 = Configuration::default();
        let mut config2 = Configuration::default();

        config1
            .language_presets
            .insert("rs".to_string(), "rust".to_string());
        config2
            .language_presets
            .insert("py".to_string(), "python".to_string());

        config1.merge(config2);
        assert_eq!(
            config1.language_presets.len(),
            2,
            "Should combine both presets"
        );
        assert!(config1.language_presets.contains_key("rs"));
        assert!(config1.language_presets.contains_key("py"));
    }

    #[test]
    fn test_merge_handles_empty_user_config() {
        let mut default_config = Configuration::default();
        let original_deny = default_config.deny_by_default;

        let empty_config = Configuration::default();
        default_config.merge(empty_config);

        assert_eq!(default_config.deny_by_default, original_deny);
    }

    #[test]
    fn test_is_code_point_allowed_deny_by_default() {
        let config = Configuration::default(); // deny_by_default = true
        let path = PathBuf::from("test.txt");

        // ASCII should be allowed
        assert!(config.is_code_point_allowed(&path, 0x0041)); // 'A'
        assert!(!config.is_code_point_allowed(&path, 0x007F)); // DEL

        // Non-ASCII should be denied
        assert!(!config.is_code_point_allowed(&path, 0x0080));
        assert!(!config.is_code_point_allowed(&path, 0x200B)); // Zero-width
        // space
    }

    #[test]
    fn test_is_code_point_allowed_allow_by_default() {
        let config = Configuration {
            deny_by_default: false,
            ..Default::default()
        };
        let path = PathBuf::from("test.txt");

        // Everything should be allowed by default
        assert!(config.is_code_point_allowed(&path, 0x0041));
        assert!(config.is_code_point_allowed(&path, 0x0080));
        assert!(config.is_code_point_allowed(&path, 0x200B));
    }

    #[test]
    fn test_get_allowed_ranges_no_match() {
        let config = Configuration::default();
        let path = PathBuf::from("test.txt");

        let ranges = config.get_allowed_ranges(&path);
        assert!(ranges.is_none(), "Should return None when no rules match");
    }

    #[test]
    fn test_get_allowed_ranges_with_preset() {
        let mut config = Configuration::default();
        config
            .language_presets
            .insert("rs".to_string(), "rust".to_string());

        let path = PathBuf::from("test.rs");
        let ranges = config.get_allowed_ranges(&path);

        assert!(
            ranges.is_some(),
            "Should return ranges for .rs files with rust preset"
        );
    }

    #[test]
    fn test_allow_by_default_with_denied_code_points() {
        let mut config = Configuration {
            deny_by_default: false,
            ..Default::default()
        };
        let mut rule = rules::FileRule::new("*.rs").expect("valid pattern");
        rule.denied_code_points.push(0x0041); // Deny 'A'
        config.file_rules.push(rule);

        let path = PathBuf::from("test.rs");
        assert!(
            !config.is_code_point_allowed(&path, 0x0041),
            "Explicitly denied code point should be rejected"
        );
        assert!(
            config.is_code_point_allowed(&path, 0x0042),
            "'B' should still be allowed"
        );
    }

    #[test]
    fn test_get_allowed_ranges_rule_with_empty_ranges() {
        let mut config = Configuration::default();
        // Rule matches but has empty allowed_ranges -> falls through to preset
        let rule = rules::FileRule::new("*.rs").expect("valid pattern");
        config.file_rules.push(rule);
        config
            .language_presets
            .insert("rs".to_string(), "rust".to_string());

        let path = PathBuf::from("test.rs");
        let ranges = config.get_allowed_ranges(&path);
        assert!(
            ranges.is_some(),
            "Should fall through to preset when rule has empty ranges"
        );
    }

    #[test]
    fn test_get_allowed_ranges_invalid_preset() {
        let mut config = Configuration::default();
        config
            .language_presets
            .insert("rs".to_string(), "nonexistent_preset".to_string());

        let path = PathBuf::from("test.rs");
        let ranges = config.get_allowed_ranges(&path);
        assert!(ranges.is_none(), "Invalid preset name should return None");
    }

    #[test]
    fn test_deny_by_default_with_rule_allowlist() {
        let mut config = Configuration::default(); // deny_by_default = true
        let mut rule = rules::FileRule::new("*.rs").expect("valid pattern");
        rule.allowed_ranges.push(UnicodeRange::new(0x0000, 0x00FF)); // Allow Basic Latin + Latin-1
        config.file_rules.push(rule);

        let path = PathBuf::from("test.rs");
        assert!(
            config.is_code_point_allowed(&path, 0x00E9),
            "'é' should be allowed by rule allowlist"
        );
        assert!(
            !config.is_code_point_allowed(&path, 0x0100),
            "Code point outside allowlist should be denied"
        );
    }

    #[test]
    fn test_deny_by_default_rule_with_denied_code_points() {
        let mut config = Configuration::default(); // deny_by_default = true
        let mut rule = rules::FileRule::new("*.rs").expect("valid pattern");
        rule.denied_code_points.push(0x0041); // Deny 'A'
        rule.allowed_ranges.push(UnicodeRange::new(0x0000, 0x007F)); // Allow ASCII
        config.file_rules.push(rule);

        let path = PathBuf::from("test.rs");
        assert!(
            !config.is_code_point_allowed(&path, 0x0041),
            "'A' explicitly denied even though in allowlist"
        );
        assert!(
            config.is_code_point_allowed(&path, 0x0042),
            "'B' should still be allowed"
        );
    }
}
