// Additional coverage tests for cli/args module
use std::path::PathBuf;
use std::str::FromStr;

use unicleaner::cli::args::{Command, OutputFormat, OutputSpec};

// --- OutputSpec::from_str tests ---

#[test]
fn test_output_spec_parse_json() {
    let spec = OutputSpec::from_str("json:output.json").unwrap();
    assert!(matches!(spec.format, OutputFormat::Json));
    assert_eq!(spec.path, PathBuf::from("output.json"));
}

#[test]
fn test_output_spec_parse_markdown() {
    let spec = OutputSpec::from_str("markdown:report.md").unwrap();
    assert!(matches!(spec.format, OutputFormat::Markdown));
    assert_eq!(spec.path, PathBuf::from("report.md"));
}

#[test]
fn test_output_spec_parse_md_alias() {
    let spec = OutputSpec::from_str("md:report.md").unwrap();
    assert!(matches!(spec.format, OutputFormat::Markdown));
    assert_eq!(spec.path, PathBuf::from("report.md"));
}

#[test]
fn test_output_spec_parse_human() {
    let spec = OutputSpec::from_str("human:out.txt").unwrap();
    assert!(matches!(spec.format, OutputFormat::Human));
    assert_eq!(spec.path, PathBuf::from("out.txt"));
}

#[test]
fn test_output_spec_parse_github() {
    let spec = OutputSpec::from_str("github:gh.txt").unwrap();
    assert!(matches!(spec.format, OutputFormat::Github));
    assert_eq!(spec.path, PathBuf::from("gh.txt"));
}

#[test]
fn test_output_spec_parse_gitlab() {
    let spec = OutputSpec::from_str("gitlab:gl.txt").unwrap();
    assert!(matches!(spec.format, OutputFormat::Gitlab));
    assert_eq!(spec.path, PathBuf::from("gl.txt"));
}

#[test]
fn test_output_spec_missing_colon() {
    let result = OutputSpec::from_str("no-colon");
    assert!(result.is_err());
}

#[test]
fn test_output_spec_unknown_format() {
    let result = OutputSpec::from_str("bogus:file.txt");
    assert!(result.is_err());
}

// --- Command clone tests ---

#[test]
fn test_command_clone_format_report() {
    let cmd = Command::FormatReport {
        input: Some(PathBuf::from("report.json")),
    };
    let cloned = cmd.clone();
    if let Command::FormatReport { input } = cloned {
        assert_eq!(input, Some(PathBuf::from("report.json")));
    } else {
        panic!("Expected FormatReport variant");
    }
}

#[test]
fn test_command_clone_init() {
    let cmd = Command::Init {
        output: PathBuf::from("my.toml"),
        force: true,
    };
    let cloned = cmd.clone();
    if let Command::Init { output, force } = cloned {
        assert_eq!(output, PathBuf::from("my.toml"));
        assert!(force);
    } else {
        panic!("Expected Init variant");
    }
}

// --- get_command default test ---
// Note: test_get_command_default requires clap::Parser trait which is only
// available inside the library crate. This test is exercised by the existing
// unit test in src/cli/args.rs. We test the Command clone here as a proxy.

#[test]
fn test_command_clone_scan() {
    let cmd = Command::Scan {
        paths: vec![PathBuf::from(".")],
        diff: false,
        jobs: None,
        encoding: None,
        outputs: vec![],
    };
    let cloned = cmd.clone();
    if let Command::Scan { paths, diff, .. } = cloned {
        assert_eq!(paths, vec![PathBuf::from(".")]);
        assert!(!diff);
    } else {
        panic!("Expected Scan variant");
    }
}
