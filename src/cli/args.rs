//! CLI argument parsing

use crate::cli::output::ColorMode;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "unicleaner")]
#[command(version)]
#[command(about = "Detect malicious Unicode characters in source code")]
#[command(
    long_about = "Unicleaner scans source code for potentially malicious Unicode characters \
                  including:\n- Zero-width characters (U+200B, U+200C, U+200D, U+FEFF)\n- \
                  Bidirectional override characters (U+202A-U+202E) - Trojan Source attacks\n- \
                  Homoglyphs - visually similar characters from different scripts\n- \
                  Non-printable control characters outside standard ASCII range\n\nUse \
                  'unicleaner <COMMAND> --help' for command-specific help."
)]
#[command(author = "unicleaner contributors")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "human", global = true)]
    pub format: OutputFormat,

    /// Control color output (auto, always, never)
    #[arg(long, value_enum, default_value = "auto", global = true)]
    pub color: ColorOption,

    /// Disable color output (deprecated: use --color=never)
    #[arg(long, global = true, conflicts_with = "color")]
    pub no_color: bool,

    /// Show only summary (suppress individual violations)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Minimum severity level to report (error, warning, info)
    #[arg(long, value_enum, global = true)]
    pub severity: Option<SeverityLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scan files for malicious Unicode (default command)
    Scan {
        /// Paths to scan (files or directories)
        #[arg(value_name = "PATH", default_value = ".")]
        paths: Vec<PathBuf>,

        /// Scan only files changed in git (diff mode)
        #[arg(long)]
        diff: bool,

        /// Maximum number of parallel threads (default: number of CPUs)
        #[arg(short = 'j', long)]
        jobs: Option<usize>,

        /// Force a specific encoding (auto-detect if not specified)
        #[arg(long, value_enum)]
        encoding: Option<EncodingOption>,
    },

    /// Generate a default configuration file
    Init {
        /// Output path for configuration file
        #[arg(value_name = "FILE", default_value = "unicleaner.toml")]
        output: PathBuf,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },

    /// List available language presets
    ListPresets,
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
pub enum ColorOption {
    /// Auto-detect based on TTY and NO_COLOR
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

impl ColorOption {
    /// Convert to ColorMode for use in output module
    pub fn to_color_mode(self) -> ColorMode {
        match self {
            Self::Auto => ColorMode::Auto,
            Self::Always => ColorMode::Always,
            Self::Never => ColorMode::Never,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SeverityLevel {
    /// Only show errors
    Error,
    /// Show warnings and errors
    Warning,
    /// Show all violations including info
    Info,
}

impl SeverityLevel {
    /// Convert to Severity for use in filtering
    pub fn to_severity(self) -> crate::unicode::malicious::Severity {
        use crate::unicode::malicious::Severity;
        match self {
            Self::Error => Severity::Error,
            Self::Warning => Severity::Warning,
            Self::Info => Severity::Info,
        }
    }
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
        // Validate subcommand-specific arguments
        if let Some(Command::Scan { paths, jobs, .. }) = &self.command {
            // Validate that at least one path is provided
            if paths.is_empty() {
                return Err("At least one path must be provided".to_string());
            }

            // Validate jobs parameter
            if let Some(jobs) = jobs {
                if *jobs == 0 {
                    return Err("Number of jobs must be at least 1".to_string());
                }
            }
        }

        Ok(())
    }

    /// Get the command, defaulting to Scan if none specified
    pub fn get_command(&self) -> Command {
        self.command.clone().unwrap_or(Command::Scan {
            paths: vec![PathBuf::from(".")],
            diff: false,
            jobs: None,
            encoding: None,
        })
    }

    /// Get the effective color mode, handling the deprecated --no-color flag
    pub fn get_color_mode(&self) -> ColorMode {
        if self.no_color {
            ColorMode::Never
        } else {
            self.color.to_color_mode()
        }
    }
}

impl Clone for Command {
    fn clone(&self) -> Self {
        match self {
            Command::Scan {
                paths,
                diff,
                jobs,
                encoding,
            } => Command::Scan {
                paths: paths.clone(),
                diff: *diff,
                jobs: *jobs,
                encoding: *encoding,
            },
            Command::Init { output, force } => Command::Init {
                output: output.clone(),
                force: *force,
            },
            Command::ListPresets => Command::ListPresets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default() {
        let args = Args::try_parse_from(vec!["unicleaner"]).unwrap();
        assert_eq!(args.config, None);
        assert!(!args.no_color);
        assert!(!args.quiet);
        assert!(!args.verbose);
        assert!(args.command.is_none());
    }

    #[test]
    fn test_args_scan_with_paths() {
        let args = Args::try_parse_from(vec!["unicleaner", "scan", "src", "tests"]).unwrap();
        if let Some(Command::Scan { paths, .. }) = args.command {
            assert_eq!(paths, vec![PathBuf::from("src"), PathBuf::from("tests")]);
        } else {
            panic!("Expected Scan command");
        }
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
        let args = Args::try_parse_from(vec!["unicleaner", "scan", "src"]).unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_args_validate_jobs_zero() {
        let args = Args::try_parse_from(vec!["unicleaner", "scan", "--jobs", "0"]).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_args_encoding_flag() {
        let args =
            Args::try_parse_from(vec!["unicleaner", "scan", "--encoding", "utf16-le"]).unwrap();
        if let Some(Command::Scan { encoding, .. }) = args.command {
            assert!(encoding.is_some());
            assert!(matches!(encoding.unwrap(), EncodingOption::Utf16Le));
        } else {
            panic!("Expected Scan command");
        }
    }

    #[test]
    fn test_init_command() {
        let args = Args::try_parse_from(vec!["unicleaner", "init"]).unwrap();
        if let Some(Command::Init { output, force }) = args.command {
            assert_eq!(output, PathBuf::from("unicleaner.toml"));
            assert!(!force);
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_init_command_with_path() {
        let args = Args::try_parse_from(vec!["unicleaner", "init", "custom.toml"]).unwrap();
        if let Some(Command::Init { output, .. }) = args.command {
            assert_eq!(output, PathBuf::from("custom.toml"));
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_list_presets_command() {
        let args = Args::try_parse_from(vec!["unicleaner", "list-presets"]).unwrap();
        assert!(matches!(args.command, Some(Command::ListPresets)));
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

    #[test]
    fn test_color_option_auto() {
        let args = Args::try_parse_from(vec!["unicleaner", "--color", "auto"]).unwrap();
        assert!(matches!(args.color, ColorOption::Auto));
        assert_eq!(args.get_color_mode(), ColorMode::Auto);
    }

    #[test]
    fn test_color_option_always() {
        let args = Args::try_parse_from(vec!["unicleaner", "--color", "always"]).unwrap();
        assert!(matches!(args.color, ColorOption::Always));
        assert_eq!(args.get_color_mode(), ColorMode::Always);
    }

    #[test]
    fn test_color_option_never() {
        let args = Args::try_parse_from(vec!["unicleaner", "--color", "never"]).unwrap();
        assert!(matches!(args.color, ColorOption::Never));
        assert_eq!(args.get_color_mode(), ColorMode::Never);
    }

    #[test]
    fn test_color_option_default() {
        let args = Args::try_parse_from(vec!["unicleaner"]).unwrap();
        assert!(matches!(args.color, ColorOption::Auto));
        assert_eq!(args.get_color_mode(), ColorMode::Auto);
    }

    #[test]
    fn test_no_color_flag() {
        let args = Args::try_parse_from(vec!["unicleaner", "--no-color"]).unwrap();
        assert!(args.no_color);
        assert_eq!(args.get_color_mode(), ColorMode::Never);
    }

    #[test]
    fn test_color_option_conversion() {
        assert_eq!(ColorOption::Auto.to_color_mode(), ColorMode::Auto);
        assert_eq!(ColorOption::Always.to_color_mode(), ColorMode::Always);
        assert_eq!(ColorOption::Never.to_color_mode(), ColorMode::Never);
    }

    #[test]
    fn test_severity_flag() {
        let args = Args::try_parse_from(vec!["unicleaner", "--severity", "error"]).unwrap();
        assert!(args.severity.is_some());
        assert!(matches!(args.severity.unwrap(), SeverityLevel::Error));
    }

    #[test]
    fn test_severity_default() {
        let args = Args::try_parse_from(vec!["unicleaner"]).unwrap();
        assert!(args.severity.is_none());
    }

    #[test]
    fn test_severity_level_conversion() {
        use crate::unicode::malicious::Severity;
        assert_eq!(SeverityLevel::Error.to_severity(), Severity::Error);
        assert_eq!(SeverityLevel::Warning.to_severity(), Severity::Warning);
        assert_eq!(SeverityLevel::Info.to_severity(), Severity::Info);
    }
}
