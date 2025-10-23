//! TOML configuration file parsing

use crate::config::{Configuration, FileRule};
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// TOML configuration file structure
#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    global: GlobalConfig,

    #[serde(default)]
    languages: HashMap<String, LanguageConfig>,

    #[serde(default)]
    rules: Vec<RuleConfig>,
}

#[derive(Debug, Deserialize)]
struct GlobalConfig {
    #[serde(default = "default_deny_by_default")]
    deny_by_default: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            deny_by_default: true,
        }
    }
}

fn default_deny_by_default() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct LanguageConfig {
    preset: String,
}

#[derive(Debug, Deserialize)]
struct RuleConfig {
    pattern: String,

    #[serde(default)]
    allowed_ranges: Vec<[u32; 2]>,

    #[serde(default)]
    denied_characters: Vec<u32>,
}

/// Load configuration from a TOML file
pub fn load_config(path: impl AsRef<Path>) -> Result<Configuration, Error> {
    let path = path.as_ref();

    // Read file
    let contents = fs::read_to_string(path)
        .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

    // Parse TOML
    parse_config(&contents, path)
}

/// Parse TOML configuration string
pub fn parse_config(toml: &str, path: &Path) -> Result<Configuration, Error> {
    let config_file: ConfigFile =
        toml::from_str(toml).map_err(|e| Error::Config(format!("Failed to parse TOML: {}", e)))?;

    let mut config = Configuration {
        deny_by_default: config_file.global.deny_by_default,
        language_presets: HashMap::new(),
        file_rules: Vec::new(),
        config_path: path.to_path_buf(),
    };

    // Load language presets
    for (lang, lang_config) in config_file.languages {
        config.language_presets.insert(lang, lang_config.preset);
    }

    // Load file rules
    for rule_config in config_file.rules {
        let mut rule = FileRule::new(&rule_config.pattern)?;

        // Add allowed ranges
        for range in rule_config.allowed_ranges {
            rule = rule.with_allowed_range(range[0], range[1], None);
        }

        // Add denied characters
        for code_point in rule_config.denied_characters {
            rule = rule.with_denied_code_point(code_point);
        }

        config.file_rules.push(rule);
    }

    // Sort rules by priority
    crate::config::rules::sort_rules_by_priority(&mut config.file_rules);

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_config() {
        let toml = "";
        let config = parse_config(toml, Path::new("test.toml")).unwrap();

        // Should use defaults
        assert!(config.deny_by_default);
        assert!(config.language_presets.is_empty());
        assert!(config.file_rules.is_empty());
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[global]
deny_by_default = true
        "#;

        let config = parse_config(toml, Path::new("test.toml")).unwrap();
        assert!(config.deny_by_default);
    }

    #[test]
    fn test_parse_config_with_language_preset() {
        let toml = r#"
[languages.rust]
preset = "rust"
        "#;

        let config = parse_config(toml, Path::new("test.toml")).unwrap();
        assert_eq!(
            config.language_presets.get("rust"),
            Some(&"rust".to_string())
        );
    }

    #[test]
    fn test_parse_config_with_custom_rules() {
        let toml = r#"
[[rules]]
pattern = "*.rs"
allowed_ranges = [[0x0000, 0x007F]]
        "#;

        let config = parse_config(toml, Path::new("test.toml")).unwrap();
        assert_eq!(config.file_rules.len(), 1);
        assert_eq!(config.file_rules[0].pattern, "*.rs");
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = r#"
[global
invalid syntax
        "#;

        let result = parse_config(toml, Path::new("test.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[global]").unwrap();
        writeln!(file, "deny_by_default = false").unwrap();

        let config = load_config(file.path()).unwrap();
        assert!(!config.deny_by_default);
    }
}
