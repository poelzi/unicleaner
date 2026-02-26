//! Built-in language presets for Unicode character allowlists

use crate::unicode::ranges::UnicodeRange;
use std::collections::HashMap;

/// Language preset defining allowed Unicode ranges
#[derive(Debug, Clone)]
pub struct LanguagePreset {
    pub name: String,
    pub description: String,
    pub allowed_ranges: Vec<UnicodeRange>,
}

impl LanguagePreset {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            allowed_ranges: Vec::new(),
        }
    }

    pub fn with_range(mut self, start: u32, end: u32, description: Option<String>) -> Self {
        self.allowed_ranges.push(UnicodeRange {
            start,
            end,
            description,
        });
        self
    }
}

use once_cell::sync::Lazy;

/// Static cache of all presets (initialized once)
static ALL_PRESETS: Lazy<HashMap<String, LanguagePreset>> = Lazy::new(build_all_presets);

/// Get all built-in language presets (cached, zero allocation after first call)
pub fn get_all_presets() -> &'static HashMap<String, LanguagePreset> {
    &ALL_PRESETS
}

fn build_all_presets() -> HashMap<String, LanguagePreset> {
    let mut presets = HashMap::new();

    // Rust preset - conservative ASCII + common symbols
    let rust = LanguagePreset::new("rust", "Rust source code - ASCII only")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(
            0x0009,
            0x000D,
            Some("Control chars (tab, newline, CR)".to_string()),
        );
    presets.insert("rust".to_string(), rust);

    // JavaScript/TypeScript preset - allows more Unicode for strings
    let javascript = LanguagePreset::new(
        "javascript",
        "JavaScript/TypeScript - Basic Latin + common symbols",
    )
    .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
    .with_range(0x0009, 0x000D, Some("Control chars".to_string()))
    .with_range(0x00A0, 0x00FF, Some("Latin-1 Supplement".to_string()));
    presets.insert("javascript".to_string(), javascript.clone());
    presets.insert("typescript".to_string(), javascript);

    // Python preset - similar to JavaScript
    let python = LanguagePreset::new("python", "Python source code - Basic Latin + Latin-1")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(0x0009, 0x000D, Some("Control chars".to_string()))
        .with_range(0x00A0, 0x00FF, Some("Latin-1 Supplement".to_string()));
    presets.insert("python".to_string(), python);

    // Java preset
    let java = LanguagePreset::new("java", "Java source code - Basic Latin + Latin-1")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(0x0009, 0x000D, Some("Control chars".to_string()))
        .with_range(0x00A0, 0x00FF, Some("Latin-1 Supplement".to_string()));
    presets.insert("java".to_string(), java);

    // C/C++ preset - very conservative
    let c = LanguagePreset::new("c", "C/C++ source code - ASCII only")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(0x0009, 0x000D, Some("Control chars".to_string()));
    presets.insert("c".to_string(), c.clone());
    presets.insert("cpp".to_string(), c);

    // Go preset
    let go = LanguagePreset::new("go", "Go source code - UTF-8 friendly")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(0x0009, 0x000D, Some("Control chars".to_string()))
        .with_range(0x00A0, 0x024F, Some("Latin Extended".to_string()));
    presets.insert("go".to_string(), go);

    // Ruby preset
    let ruby = LanguagePreset::new("ruby", "Ruby source code - UTF-8 friendly")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(0x0009, 0x000D, Some("Control chars".to_string()))
        .with_range(0x00A0, 0x00FF, Some("Latin-1 Supplement".to_string()));
    presets.insert("ruby".to_string(), ruby);

    // PHP preset
    let php = LanguagePreset::new("php", "PHP source code - Basic Latin + Latin-1")
        .with_range(0x0020, 0x007E, Some("Basic Latin printable".to_string()))
        .with_range(0x0009, 0x000D, Some("Control chars".to_string()))
        .with_range(0x00A0, 0x00FF, Some("Latin-1 Supplement".to_string()));
    presets.insert("php".to_string(), php);

    presets
}

/// Get a specific preset by name
pub fn get_preset(name: &str) -> Option<LanguagePreset> {
    ALL_PRESETS.get(name).cloned()
}

/// List all available preset names
pub fn list_preset_names() -> Vec<String> {
    let mut names: Vec<String> = get_all_presets().keys().cloned().collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_preset_exists() {
        let preset = get_preset("rust");
        assert!(preset.is_some());
        let preset = preset.unwrap();
        assert_eq!(preset.name, "rust");
    }

    #[test]
    fn test_javascript_preset_exists() {
        let preset = get_preset("javascript");
        assert!(preset.is_some());
    }

    #[test]
    fn test_python_preset_exists() {
        let preset = get_preset("python");
        assert!(preset.is_some());
    }

    #[test]
    fn test_preset_contains_basic_latin() {
        let preset = get_preset("rust").unwrap();
        // Should include Basic Latin printable range
        assert!(
            preset
                .allowed_ranges
                .iter()
                .any(|r| r.start == 0x0020 && r.end == 0x007E)
        );
    }

    #[test]
    fn test_preset_has_description() {
        let preset = get_preset("rust").unwrap();
        assert!(!preset.description.is_empty());

        // Check that ranges also have descriptions
        assert!(
            preset
                .allowed_ranges
                .iter()
                .all(|r| r.description.is_some())
        );
    }

    #[test]
    fn test_list_all_presets() {
        let names = list_preset_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"rust".to_string()));
        assert!(names.contains(&"javascript".to_string()));
        assert!(names.contains(&"python".to_string()));
    }

    #[test]
    fn test_get_preset_by_name() {
        let preset = get_preset("rust");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "rust");
    }

    #[test]
    fn test_presets_static() {
        // T059: Verify get_all_presets() returns the same pointer on repeated calls
        // (i.e., it's cached via once_cell::sync::Lazy, not rebuilt each time)
        let first = get_all_presets() as *const HashMap<String, LanguagePreset>;
        let second = get_all_presets() as *const HashMap<String, LanguagePreset>;
        assert_eq!(
            first, second,
            "get_all_presets() should return the same static reference"
        );
    }

    #[test]
    fn test_unknown_preset_returns_none() {
        let preset = get_preset("nonexistent-language");
        assert!(preset.is_none());
    }
}
