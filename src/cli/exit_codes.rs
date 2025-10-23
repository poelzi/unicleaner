//! Exit code constants following Unix conventions

/// Success - no malicious Unicode found
pub const SUCCESS: i32 = 0;

/// Violations found - malicious Unicode detected
pub const VIOLATIONS_FOUND: i32 = 1;

/// Error - configuration error, I/O error, or other runtime error
pub const ERROR: i32 = 2;

/// Partial success - some files could not be scanned
pub const PARTIAL_SUCCESS: i32 = 3;
