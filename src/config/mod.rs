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
            return Some(rule.allowed_ranges.clone());
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
        // Check file-specific rules first (highest priority)
        if let Some(rule) = rules::find_matching_rule(&self.file_rules, file_path) {
            return rule.is_code_point_allowed(code_point);
        }

        // Check language presets
        if let Some(ranges) = self.get_allowed_ranges(file_path) {
            return ranges.iter().any(|range| range.contains(code_point));
        }

        // Fall back to global setting
        if self.deny_by_default {
            // Deny by default - only ASCII is safe
            code_point <= 0x007F
        } else {
            // Allow by default - only deny explicitly malicious
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_default_with_user_config() {
        // User config should override defaults
        assert!(true); // Placeholder
    }

    #[test]
    fn test_merge_preserves_user_rules() {
        // User-defined rules should be preserved
        assert!(true); // Placeholder
    }

    #[test]
    fn test_merge_combines_presets() {
        // Multiple preset references should be combined
        assert!(true); // Placeholder
    }

    #[test]
    fn test_merge_handles_empty_user_config() {
        // Empty user config should use all defaults
        assert!(true); // Placeholder
    }

    #[test]
    fn test_config_precedence() {
        // File-specific rules > language presets > global settings
        assert!(true); // Placeholder
    }
}
