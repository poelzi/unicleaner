//! Unicleaner - Detect malicious Unicode characters in source code
//!
//! This library provides functionality to scan source code for potentially malicious
//! Unicode characters including zero-width characters, bidirectional overrides,
//! homoglyphs, and other security threats.
//!
//! # Example
//!
//! ```no_run
//! // Library usage will be demonstrated after implementation
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
    Git(#[from] git2::Error),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Unicode error: {0}")]
    Unicode(String),
}
