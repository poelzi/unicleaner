//! Configuration validation logic

use crate::Error;
use crate::config::{Configuration, presets};

/// Validate a configuration
pub fn validate_config(config: &Configuration) -> Result<(), Error> {
    // Validate all language presets exist
    for (lang, preset_name) in &config.language_presets {
        if presets::get_preset(preset_name).is_none() {
            return Err(Error::Config(format!(
                "Unknown preset '{}' for language '{}'",
                preset_name, lang
            )));
        }
    }

    // Validate all file rules have valid glob patterns (already validated during
    // creation) Validate Unicode ranges
    for rule in &config.file_rules {
        for range in &rule.allowed_ranges {
            if range.start > range.end {
                return Err(Error::Config(format!(
                    "Invalid Unicode range: start ({:#X}) > end ({:#X})",
                    range.start, range.end
                )));
            }

            if range.end > 0x10FFFF {
                return Err(Error::Config(format!(
                    "Invalid Unicode range: end ({:#X}) exceeds maximum Unicode code point \
                     (U+10FFFF)",
                    range.end
                )));
            }
        }

        // In deny-by-default mode, allowlists that omit ASCII are almost always
        // a misconfiguration (they will flag normal source code). Deny-only rules
        // (no allowlist) are allowed and fall back to presets / global defaults.
        if config.deny_by_default
            && !rule.allowed_ranges.is_empty()
            && !has_safe_ascii_allowlist(&rule.allowed_ranges)
        {
            return Err(Error::Config(format!(
                "Rule '{}' defines an allowlist but does not include safe ASCII. \
                 In deny-by-default mode this will flag most source code. \
                 Include [0x0009, 0x000D] and [0x0020, 0x007E] (or allowed_blocks = ['ascii']).",
                rule.pattern
            )));
        }
    }

    Ok(())
}

fn has_safe_ascii_allowlist(ranges: &[crate::unicode::ranges::UnicodeRange]) -> bool {
    let required = [0x0009u32, 0x0020, 0x0041, 0x0061, 0x007E];
    required
        .iter()
        .all(|&cp| ranges.iter().any(|r| r.contains(cp)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FileRule;

    #[test]
    fn test_validate_empty_config() {
        let config = Configuration::new();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_unicode_range() {
        let mut config = Configuration::new();
        let rule = FileRule::new("*.rs")
            .unwrap()
            .with_allowed_range(0x0100, 0x0010, None); // Invalid: start > end
        config.file_rules.push(rule);

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_overlapping_ranges() {
        // Overlapping ranges should be allowed (they're valid)
        let mut config = Configuration::new();
        let rule = FileRule::new("*.rs")
            .unwrap()
            .with_allowed_range(0x0000, 0x00FF, None)
            .with_allowed_range(0x0080, 0x0100, None); // Overlaps
        config.file_rules.push(rule);

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_invalid_glob_pattern() {
        // Invalid glob should fail during FileRule creation, not validation
        let result = FileRule::new("[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_conflicting_rules() {
        // Conflicting rules are allowed (priority determines which applies)
        let mut config = Configuration::new();
        config.file_rules.push(FileRule::new("*.rs").unwrap());
        config.file_rules.push(FileRule::new("src/*.rs").unwrap());

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_unknown_preset() {
        let mut config = Configuration::new();
        config
            .language_presets
            .insert("rust".to_string(), "nonexistent".to_string());

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_deny_by_default_rule_missing_ascii_rejected() {
        let mut config = Configuration::new();
        config.deny_by_default = true;

        // Cyrillic-only allowlist: almost certainly a misconfiguration.
        let rule = FileRule::new("*.rs")
            .unwrap()
            .with_allowed_range(0x0400, 0x04FF, None);
        config.file_rules.push(rule);

        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_deny_by_default_deny_only_rule_allowed() {
        let mut config = Configuration::new();
        config.deny_by_default = true;

        // Deny-only rules are allowed and fall back to presets / defaults.
        let rule = FileRule::new("*.rs")
            .unwrap()
            .with_denied_code_point(0x200B);
        config.file_rules.push(rule);

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_circular_preset_references() {
        // Not applicable since we use built-in presets only
        // If we later support custom presets that reference each other,
        // this test would check for circular references
    }
}
