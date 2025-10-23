//! CLI argument parsing

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "unicleaner")]
#[command(version, about, long_about = None)]
#[command(author = "unicleaner contributors")]
pub struct Args {
    /// Paths to scan (files or directories)
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "human")]
    pub format: OutputFormat,

    /// Disable color output
    #[arg(long)]
    pub no_color: bool,

    /// Show only summary (suppress individual violations)
    #[arg(short, long)]
    pub quiet: bool,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Scan only files changed in git (diff mode)
    #[arg(long)]
    pub diff: bool,

    /// Maximum number of parallel threads (default: number of CPUs)
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Force a specific encoding (auto-detect if not specified)
    #[arg(long, value_enum)]
    pub encoding: Option<EncodingOption>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// GitHub Actions format
    Github,
    /// GitLab CI format
    Gitlab,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EncodingOption {
    /// UTF-8 encoding
    Utf8,
    /// UTF-16 Little Endian
    Utf16Le,
    /// UTF-16 Big Endian
    Utf16Be,
    /// UTF-32 Little Endian
    Utf32Le,
    /// UTF-32 Big Endian
    Utf32Be,
}

impl EncodingOption {
    /// Convert to DetectedEncoding for use in scanner
    pub fn to_detected_encoding(self) -> crate::scanner::encoding::DetectedEncoding {
        use crate::scanner::encoding::DetectedEncoding;
        match self {
            Self::Utf8 => DetectedEncoding::Utf8,
            Self::Utf16Le => DetectedEncoding::Utf16Le,
            Self::Utf16Be => DetectedEncoding::Utf16Be,
            Self::Utf32Le => DetectedEncoding::Utf32Le,
            Self::Utf32Be => DetectedEncoding::Utf32Be,
        }
    }
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate that at least one path is provided
        if self.paths.is_empty() {
            return Err("At least one path must be provided".to_string());
        }

        // Validate jobs parameter
        if let Some(jobs) = self.jobs {
            if jobs == 0 {
                return Err("Number of jobs must be at least 1".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default() {
        let args = Args::try_parse_from(vec!["unicleaner"]).unwrap();
        assert_eq!(args.paths, vec![PathBuf::from(".")]);
        assert_eq!(args.config, None);
        assert!(!args.no_color);
        assert!(!args.quiet);
        assert!(!args.verbose);
    }

    #[test]
    fn test_args_with_paths() {
        let args = Args::try_parse_from(vec!["unicleaner", "src", "tests"]).unwrap();
        assert_eq!(
            args.paths,
            vec![PathBuf::from("src"), PathBuf::from("tests")]
        );
    }

    #[test]
    fn test_args_with_config() {
        let args = Args::try_parse_from(vec!["unicleaner", "--config", "unicleaner.toml"]).unwrap();
        assert_eq!(args.config, Some(PathBuf::from("unicleaner.toml")));
    }

    #[test]
    fn test_args_no_color() {
        let args = Args::try_parse_from(vec!["unicleaner", "--no-color"]).unwrap();
        assert!(args.no_color);
    }

    #[test]
    fn test_args_output_format() {
        let args = Args::try_parse_from(vec!["unicleaner", "--format", "json"]).unwrap();
        assert!(matches!(args.format, OutputFormat::Json));
    }

    #[test]
    fn test_args_validate_success() {
        let args = Args::try_parse_from(vec!["unicleaner", "src"]).unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validate_jobs_zero() {
        let args = Args::try_parse_from(vec!["unicleaner", "--jobs", "0"]).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_args_encoding_flag() {
        let args = Args::try_parse_from(vec!["unicleaner", "--encoding", "utf16-le"]).unwrap();
        assert!(args.encoding.is_some());
        assert!(matches!(args.encoding.unwrap(), EncodingOption::Utf16Le));
    }

    #[test]
    fn test_encoding_option_conversion() {
        use crate::scanner::encoding::DetectedEncoding;

        assert_eq!(
            EncodingOption::Utf8.to_detected_encoding(),
            DetectedEncoding::Utf8
        );
        assert_eq!(
            EncodingOption::Utf16Le.to_detected_encoding(),
            DetectedEncoding::Utf16Le
        );
        assert_eq!(
            EncodingOption::Utf32Be.to_detected_encoding(),
            DetectedEncoding::Utf32Be
        );
    }
}
