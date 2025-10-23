//! File-specific rules for Unicode character allowlists

use crate::unicode::ranges::UnicodeRange;
use globset::{Glob, GlobMatcher};
use std::path::Path;

/// File-specific rule for Unicode allowlists
#[derive(Debug, Clone)]
pub struct FileRule {
    pub pattern: String,
    pub matcher: GlobMatcher,
    pub allowed_ranges: Vec<UnicodeRange>,
    pub denied_code_points: Vec<u32>,
    pub priority: usize,
}

impl FileRule {
    /// Create a new file rule with a glob pattern
    pub fn new(pattern: impl Into<String>) -> Result<Self, crate::Error> {
        let pattern = pattern.into();
        let glob = Glob::new(&pattern).map_err(|e| {
            crate::Error::Config(format!("Invalid glob pattern '{}': {}", pattern, e))
        })?;

        Ok(Self {
            pattern: pattern.clone(),
            matcher: glob.compile_matcher(),
            allowed_ranges: Vec::new(),
            denied_code_points: Vec::new(),
            priority: calculate_priority(&pattern),
        })
    }

    /// Add an allowed Unicode range
    pub fn with_allowed_range(mut self, start: u32, end: u32, description: Option<String>) -> Self {
        self.allowed_ranges.push(UnicodeRange {
            start,
            end,
            description,
        });
        self
    }

    /// Add a denied code point
    pub fn with_denied_code_point(mut self, code_point: u32) -> Self {
        self.denied_code_points.push(code_point);
        self
    }

    /// Check if this rule matches a file path
    pub fn matches(&self, path: &Path) -> bool {
        self.matcher.is_match(path)
    }

    /// Check if a code point is allowed by this rule
    pub fn is_code_point_allowed(&self, code_point: u32) -> bool {
        // Check if explicitly denied
        if self.denied_code_points.contains(&code_point) {
            return false;
        }

        // Check if in allowed ranges
        self.allowed_ranges
            .iter()
            .any(|range| range.contains(code_point))
    }
}

/// Calculate priority based on pattern specificity
/// More specific patterns get higher priority
fn calculate_priority(pattern: &str) -> usize {
    let mut priority = 0;

    // Exact paths have highest priority
    if !pattern.contains('*') && !pattern.contains('?') {
        priority += 1000;
    }

    // Count directory depth
    priority += pattern.matches('/').count() * 10;

    // Patterns without wildcards are more specific
    if !pattern.contains("**") {
        priority += 50;
    }

    // Single-star patterns are more specific than double-star
    if pattern.contains('*') && !pattern.contains("**") {
        priority += 25;
    }

    priority
}

/// Sort rules by priority (highest first)
pub fn sort_rules_by_priority(rules: &mut [FileRule]) {
    rules.sort_by(|a, b| b.priority.cmp(&a.priority));
}

/// Find the first matching rule for a file
pub fn find_matching_rule<'a>(rules: &'a [FileRule], path: &Path) -> Option<&'a FileRule> {
    rules.iter().find(|rule| rule.matches(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_rule_matches_exact_path() {
        let rule = FileRule::new("src/lib.rs").unwrap();
        assert!(rule.matches(&PathBuf::from("src/lib.rs")));
        assert!(!rule.matches(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_file_rule_matches_glob_pattern() {
        let rule = FileRule::new("*.rs").unwrap();
        assert!(rule.matches(&PathBuf::from("main.rs")));
        assert!(rule.matches(&PathBuf::from("lib.rs")));
        assert!(!rule.matches(&PathBuf::from("main.js")));
    }

    #[test]
    fn test_file_rule_matches_directory_pattern() {
        let rule = FileRule::new("src/**/*.rs").unwrap();
        assert!(rule.matches(&PathBuf::from("src/main.rs")));
        assert!(rule.matches(&PathBuf::from("src/config/mod.rs")));
        assert!(!rule.matches(&PathBuf::from("tests/test.rs")));
    }

    #[test]
    fn test_file_rule_priority_ordering() {
        let exact = FileRule::new("src/lib.rs").unwrap();
        let glob = FileRule::new("src/*.rs").unwrap();
        let recursive = FileRule::new("**/*.rs").unwrap();

        // Exact path should have highest priority
        assert!(exact.priority > glob.priority);
        assert!(glob.priority > recursive.priority);
    }

    #[test]
    fn test_file_rule_with_allowed_ranges() {
        let rule = FileRule::new("*.rs").unwrap().with_allowed_range(
            0x0020,
            0x007E,
            Some("ASCII".to_string()),
        );

        assert!(rule.is_code_point_allowed(0x0041)); // 'A'
        assert!(!rule.is_code_point_allowed(0x200B)); // Zero-width space
    }

    #[test]
    fn test_file_rule_with_denied_characters() {
        let rule = FileRule::new("*.rs")
            .unwrap()
            .with_allowed_range(0x0000, 0xFFFF, None)
            .with_denied_code_point(0x200B); // Explicitly deny zero-width space

        assert!(rule.is_code_point_allowed(0x0041)); // 'A'
        assert!(!rule.is_code_point_allowed(0x200B)); // Denied
    }

    #[test]
    fn test_multiple_rules_for_same_file() {
        let mut rules = vec![
            FileRule::new("**/*.rs")
                .unwrap()
                .with_allowed_range(0x0000, 0xFFFF, None),
            FileRule::new("src/lib.rs")
                .unwrap()
                .with_allowed_range(0x0020, 0x007E, None),
        ];

        sort_rules_by_priority(&mut rules);

        // Most specific rule should be first
        let path = PathBuf::from("src/lib.rs");
        let matching = find_matching_rule(&rules, &path).unwrap();
        assert_eq!(matching.pattern, "src/lib.rs");
    }
}
