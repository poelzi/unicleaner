//! Human-readable output formatting

use crate::report::{ScanResult, Violation};
use crate::unicode::malicious::Severity;
use owo_colors::OwoColorize;

/// Format scan results for human-readable output
pub fn format_human(result: &ScanResult, use_color: bool, verbose: bool) -> String {
    let mut output = String::new();

    if !result.violations.is_empty() {
        output.push_str(&format_violations(&result.violations, use_color, verbose));
        output.push('\n');
    }

    output.push_str(&format_summary(result, use_color));

    output
}

/// Format violations with color support
fn format_violations(violations: &[Violation], use_color: bool, verbose: bool) -> String {
    let mut output = String::new();

    for violation in violations {
        let severity_str = format_severity(violation.severity, use_color);
        let file_line = if use_color {
            format!(
                "{}:{}:{}",
                violation.file_path.display().cyan(),
                violation.line.yellow(),
                violation.column.yellow()
            )
        } else {
            format!(
                "{}:{}:{}",
                violation.file_path.display(),
                violation.line,
                violation.column
            )
        };

        output.push_str(&format!(
            "{} {} {}\n",
            severity_str, file_line, violation.message
        ));

        if verbose {
            output.push_str(&format!(
                "  Character: {} ({})\n",
                violation.code_point_string(),
                violation.pattern_name
            ));
            output.push_str(&format!("  Category: {:?}\n", violation.category));
            output.push_str(&format!("  Encoding: {}\n", violation.encoding_name()));
            if !violation.context.is_empty() {
                output.push_str(&format!("  Context: {}\n", violation.context.trim()));
            }
            output.push('\n');
        }
    }

    output
}

/// Format severity level with colors
fn format_severity(severity: Severity, use_color: bool) -> String {
    if use_color {
        match severity {
            Severity::Error => format!("{}", "ERROR".red().bold()),
            Severity::Warning => format!("{}", "WARNING".yellow()),
            Severity::Info => format!("{}", "INFO".blue()),
        }
    } else {
        match severity {
            Severity::Error => "ERROR".to_string(),
            Severity::Warning => "WARNING".to_string(),
            Severity::Info => "INFO".to_string(),
        }
    }
}

/// Format summary statistics
fn format_summary(result: &ScanResult, use_color: bool) -> String {
    let mut output = String::new();

    let status = if use_color {
        if result.passed() {
            format!("{}", "PASSED".green().bold())
        } else {
            format!("{}", "FAILED".red().bold())
        }
    } else {
        if result.passed() {
            "PASSED".to_string()
        } else {
            "FAILED".to_string()
        }
    };

    output.push_str(&format!("\nScan Result: {}\n", status));
    output.push_str(&format!("Files scanned: {}\n", result.files_scanned));
    output.push_str(&format!("Files clean: {}\n", result.files_clean));
    output.push_str(&format!(
        "Files with violations: {}\n",
        result.files_with_violations
    ));
    output.push_str(&format!("Total violations: {}\n", result.violations.len()));

    if !result.errors.is_empty() {
        output.push_str(&format!("Errors: {}\n", result.errors.len()));
    }

    output.push_str(&format!(
        "Duration: {:.2}s\n",
        result.duration.as_secs_f64()
    ));
    output.push_str(&format!("Exit code: {}\n", result.exit_code()));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unicode::malicious::MaliciousCategory;
    use std::path::PathBuf;
    use std::time::Duration;

    fn create_test_violation() -> Violation {
        Violation::new(
            PathBuf::from("test.rs"),
            10,
            5,
            0x200B,
            "zero-width-space".to_string(),
            MaliciousCategory::ZeroWidth,
            Severity::Error,
            "Zero-width space detected".to_string(),
        )
    }

    #[test]
    fn test_format_human_no_violations() {
        let result = ScanResult {
            violations: vec![],
            files_scanned: 10,
            files_clean: 10,
            files_with_violations: 0,
            errors: vec![],
            duration: Duration::from_secs(1),
            config_used: PathBuf::from("unicleaner.toml"),
        };

        let output = format_human(&result, false, false);
        assert!(output.contains("PASSED"));
        assert!(output.contains("Files scanned: 10"));
        assert!(output.contains("Files clean: 10"));
    }

    #[test]
    fn test_format_human_with_violations() {
        let violation = create_test_violation();
        let result = ScanResult {
            violations: vec![violation],
            files_scanned: 10,
            files_clean: 9,
            files_with_violations: 1,
            errors: vec![],
            duration: Duration::from_secs(1),
            config_used: PathBuf::from("unicleaner.toml"),
        };

        let output = format_human(&result, false, false);
        assert!(output.contains("FAILED"));
        assert!(output.contains("test.rs"));
        assert!(output.contains("Total violations: 1"));
    }

    #[test]
    fn test_format_severity() {
        let error = format_severity(Severity::Error, false);
        assert_eq!(error, "ERROR");

        let warning = format_severity(Severity::Warning, false);
        assert_eq!(warning, "WARNING");

        let info = format_severity(Severity::Info, false);
        assert_eq!(info, "INFO");
    }

    #[test]
    fn test_format_violations_verbose() {
        let violation = create_test_violation();
        let output = format_violations(&[violation], false, true);

        assert!(output.contains("U+200B"));
        assert!(output.contains("zero-width-space"));
        assert!(output.contains("Category:"));
    }

    #[test]
    fn test_format_violations_non_verbose() {
        let violation = create_test_violation();
        let output = format_violations(&[violation], false, false);

        assert!(output.contains("test.rs"));
        assert!(!output.contains("U+200B")); // Should not show details
    }
}
