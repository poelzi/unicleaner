//! CLI argument parsing

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "unicleaner")]
#[command(version, about, long_about = None)]
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

    /// Disable color output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Show only summary (suppress individual violations)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
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
}
