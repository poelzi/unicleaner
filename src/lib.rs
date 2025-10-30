//! Unicleaner - Detect malicious Unicode characters in source code
//!
//! This library provides functionality to scan source code for potentially
//! malicious Unicode characters including zero-width characters, bidirectional
//! overrides, homoglyphs, and other security threats.
//!
//! # Features
//!
//! - **Zero-width character detection**: U+200B, U+200C, U+200D, U+FEFF
//! - **Bidirectional override detection**: U+202A-U+202E (Trojan Source
//!   attacks)
//! - **Homoglyph detection**: Visually similar characters from different
//!   scripts
//! - **Multi-encoding support**: UTF-8, UTF-16 (LE/BE), UTF-32 (LE/BE)
//! - **Configurable rules**: TOML-based configuration with language presets
//! - **Parallel scanning**: Fast multi-threaded file processing
//!
//! # Example
//!
//! ```no_run
//! use unicleaner::scanner::file_scanner::scan_file;
//! use std::path::Path;
//!
//! // Scan a single file
//! let violations = scan_file(Path::new("src/main.rs")).unwrap();
//! for violation in violations {
//!     println!("Found malicious Unicode at {}:{}", violation.line, violation.column);
//! }
//! ```

// Module declarations
pub mod cli;
pub mod config;
pub mod report;
pub mod scanner;
pub mod unicode;

// Re-export commonly used types
pub use report::{ScanResult, Violation};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type alias for unicleaner operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for unicleaner
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parsing error: {0}")]
    Parse(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Unicode error: {0}")]
    Unicode(String),
}
