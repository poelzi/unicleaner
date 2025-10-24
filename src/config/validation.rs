//! Configuration validation logic

use crate::config::{presets, Configuration};
use crate::Error;

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

    // Validate all file rules have valid glob patterns (already validated during creation)
    // Validate Unicode ranges
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
                    "Invalid Unicode range: end ({:#X}) exceeds maximum Unicode code point (U+10FFFF)",
                    range.end
                )));
            }
        }
    }

    Ok(())
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
    fn test_validate_circular_preset_references() {
        // Not applicable since we use built-in presets only
        // If we later support custom presets that reference each other,
        // this test would check for circular references
    }
}
