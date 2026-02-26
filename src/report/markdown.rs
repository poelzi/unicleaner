//! Markdown output formatting

use crate::report::{ScanResult, Violation};
use crate::unicode::malicious::Severity;

/// Format scan results as Markdown
pub fn format_markdown(result: &ScanResult, verbose: bool) -> String {
    let mut output = String::new();

    if result.violations.is_empty() {
        output.push_str("# Unicleaner Scan: PASSED\n\n");
        output.push_str("No violations found.\n\n");
    } else {
        output.push_str("# Unicleaner Scan: FAILED\n\n");
        output.push_str(&format_violations_markdown(&result.violations, verbose));
    }

    output.push_str(&format_summary_markdown(result));

    output
}

/// Group violations by file and format as Markdown sections
fn format_violations_markdown(violations: &[Violation], verbose: bool) -> String {
    use std::collections::BTreeMap;

    // Group violations by file path (BTreeMap for sorted output)
    let mut by_file: BTreeMap<&std::path::Path, Vec<&Violation>> = BTreeMap::new();
    for v in violations {
        by_file.entry(&v.file_path).or_default().push(v);
    }

    let mut output = String::new();

    for (file, file_violations) in &by_file {
        output.push_str(&format!("## `{}`\n\n", file.display()));

        for v in file_violations {
            let severity_badge = severity_badge(v.severity);
            output.push_str(&format!(
                "- {} **Line {}:{}** — {}\n",
                severity_badge, v.line, v.column, v.message
            ));

            if verbose {
                output.push_str(&format!(
                    "  - Character: `{}` (`{}`)\n",
                    v.code_point_string(),
                    v.pattern_name
                ));
                output.push_str(&format!("  - Category: `{:?}`\n", v.category));
                output.push_str(&format!("  - Encoding: {}\n", v.encoding_name()));
                if !v.context.is_empty() {
                    output.push_str(&format!("  - Context: `{}`\n", v.context.trim()));
                }
            }
        }

        output.push('\n');
    }

    output
}

fn severity_badge(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "🔴 ERROR",
        Severity::Warning => "🟡 WARNING",
        Severity::Info => "🔵 INFO",
    }
}

/// Format summary statistics as a Markdown table
fn format_summary_markdown(result: &ScanResult) -> String {
    let mut output = String::new();

    output.push_str("## Summary\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Files scanned | {} |\n", result.files_scanned));
    output.push_str(&format!("| Files clean | {} |\n", result.files_clean));
    output.push_str(&format!(
        "| Files with violations | {} |\n",
        result.files_with_violations
    ));
    output.push_str(&format!(
        "| Total violations | {} |\n",
        result.violations.len()
    ));

    if !result.errors.is_empty() {
        output.push_str(&format!("| Errors | {} |\n", result.errors.len()));
    }

    output.push_str(&format!(
        "| Duration | {:.2}s |\n",
        result.duration.as_secs_f64()
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::malicious::MaliciousCategory;
    use std::path::PathBuf;
    use std::time::Duration;

    fn create_test_violation(file: &str, line: usize, severity: Severity) -> Violation {
        Violation::new(
            PathBuf::from(file),
            line,
            5,
            0,
            0x200B,
            "zero-width-space".to_string(),
            MaliciousCategory::ZeroWidth,
            severity,
            "Zero-width space detected".to_string(),
        )
    }

    fn create_test_result(violations: Vec<Violation>) -> ScanResult {
        let files_with_violations = violations
            .iter()
            .map(|v| &v.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();
        ScanResult {
            files_scanned: 10,
            files_clean: 10 - files_with_violations,
            files_with_violations,
            violations,
            errors: vec![],
            duration: Duration::from_millis(42),
            config_used: PathBuf::from("unicleaner.toml"),
        }
    }

    #[test]
    fn test_markdown_no_violations() {
        let result = create_test_result(vec![]);
        let output = format_markdown(&result, false);
        assert!(output.contains("# Unicleaner Scan: PASSED"));
        assert!(output.contains("No violations found."));
        assert!(output.contains("| Files scanned | 10 |"));
    }

    #[test]
    fn test_markdown_with_violations() {
        let result = create_test_result(vec![
            create_test_violation("src/main.rs", 10, Severity::Error),
            create_test_violation("src/main.rs", 20, Severity::Warning),
            create_test_violation("src/lib.rs", 5, Severity::Info),
        ]);
        let output = format_markdown(&result, false);
        assert!(output.contains("# Unicleaner Scan: FAILED"));
        assert!(output.contains("## `src/main.rs`"));
        assert!(output.contains("## `src/lib.rs`"));
        assert!(output.contains("ERROR"));
        assert!(output.contains("WARNING"));
        assert!(output.contains("INFO"));
        assert!(output.contains("**Line 10:5**"));
    }

    #[test]
    fn test_markdown_verbose() {
        let result = create_test_result(vec![create_test_violation("test.rs", 1, Severity::Error)]);
        let output = format_markdown(&result, true);
        assert!(output.contains("`U+200B`"));
        assert!(output.contains("`zero-width-space`"));
        assert!(output.contains("`ZeroWidth`"));
    }

    #[test]
    fn test_markdown_summary_table() {
        let result = create_test_result(vec![]);
        let output = format_markdown(&result, false);
        assert!(output.contains("## Summary"));
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("| Duration |"));
    }

    #[test]
    fn test_severity_badges() {
        assert!(severity_badge(Severity::Error).contains("ERROR"));
        assert!(severity_badge(Severity::Warning).contains("WARNING"));
        assert!(severity_badge(Severity::Info).contains("INFO"));
    }
}
