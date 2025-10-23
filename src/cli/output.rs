//! Output formatting and color control

use std::io::IsTerminal;

/// Color mode for output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Auto-detect based on TTY and NO_COLOR
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// Determine if colors should be enabled based on mode, TTY detection, and NO_COLOR
pub fn should_use_color(mode: ColorMode, stream: ColorStream) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            // Check NO_COLOR environment variable (see https://no-color.org/)
            if is_no_color_set() {
                return false;
            }

            // Check if stream is a TTY
            is_tty(stream)
        }
    }
}

/// Stream to check for TTY
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorStream {
    Stdout,
    Stderr,
}

/// Check if the given stream is a TTY
pub fn is_tty(stream: ColorStream) -> bool {
    match stream {
        ColorStream::Stdout => std::io::stdout().is_terminal(),
        ColorStream::Stderr => std::io::stderr().is_terminal(),
    }
}

/// Check if NO_COLOR environment variable is set
/// According to https://no-color.org/, any value (even empty) means colors are disabled
pub fn is_no_color_set() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_color_mode_always() {
        // ColorMode::Always should always return true regardless of TTY or NO_COLOR
        assert!(should_use_color(ColorMode::Always, ColorStream::Stdout));
        assert!(should_use_color(ColorMode::Always, ColorStream::Stderr));

        // Even with NO_COLOR set, Always mode should use colors
        env::set_var("NO_COLOR", "1");
        assert!(should_use_color(ColorMode::Always, ColorStream::Stdout));
        env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_color_mode_never() {
        // ColorMode::Never should always return false
        assert!(!should_use_color(ColorMode::Never, ColorStream::Stdout));
        assert!(!should_use_color(ColorMode::Never, ColorStream::Stderr));
    }

    #[test]
    fn test_color_mode_auto_with_no_color() {
        // Save original NO_COLOR state
        let original = env::var("NO_COLOR").ok();

        // Set NO_COLOR to disable colors
        env::set_var("NO_COLOR", "1");
        assert!(!should_use_color(ColorMode::Auto, ColorStream::Stdout));
        assert!(!should_use_color(ColorMode::Auto, ColorStream::Stderr));

        // NO_COLOR with empty value should also disable colors
        env::set_var("NO_COLOR", "");
        assert!(!should_use_color(ColorMode::Auto, ColorStream::Stdout));

        // Restore original state
        match original {
            Some(val) => env::set_var("NO_COLOR", val),
            None => env::remove_var("NO_COLOR"),
        }
    }

    #[test]
    fn test_color_mode_auto_without_no_color() {
        // Save original NO_COLOR state
        let original = env::var("NO_COLOR").ok();

        // Ensure NO_COLOR is not set
        env::remove_var("NO_COLOR");

        // In Auto mode without NO_COLOR, should depend on TTY detection
        // We can't reliably test TTY detection in unit tests since it depends
        // on how tests are run, but we can verify the function doesn't panic
        let _ = should_use_color(ColorMode::Auto, ColorStream::Stdout);
        let _ = should_use_color(ColorMode::Auto, ColorStream::Stderr);

        // Restore original state
        if let Some(val) = original {
            env::set_var("NO_COLOR", val);
        }
    }

    #[test]
    fn test_is_no_color_set() {
        // Save original state
        let original = env::var("NO_COLOR").ok();

        // Test with NO_COLOR unset
        env::remove_var("NO_COLOR");
        assert!(!is_no_color_set());

        // Test with NO_COLOR set to "1"
        env::set_var("NO_COLOR", "1");
        assert!(is_no_color_set());

        // Test with NO_COLOR set to empty string (still counts as set)
        env::set_var("NO_COLOR", "");
        assert!(is_no_color_set());

        // Test with NO_COLOR set to arbitrary value
        env::set_var("NO_COLOR", "disabled");
        assert!(is_no_color_set());

        // Restore original state
        match original {
            Some(val) => env::set_var("NO_COLOR", val),
            None => env::remove_var("NO_COLOR"),
        }
    }

    #[test]
    fn test_is_tty() {
        // We can't reliably test actual TTY detection in unit tests
        // since it depends on how the tests are run (terminal vs CI/CD)
        // But we can verify the function doesn't panic and returns a boolean
        let stdout_is_tty = is_tty(ColorStream::Stdout);
        let stderr_is_tty = is_tty(ColorStream::Stderr);

        assert!(stdout_is_tty == true || stdout_is_tty == false);
        assert!(stderr_is_tty == true || stderr_is_tty == false);
    }

    #[test]
    fn test_color_stream_enum() {
        // Test that ColorStream enum values are distinct
        assert_ne!(ColorStream::Stdout, ColorStream::Stderr);
    }

    #[test]
    fn test_color_mode_enum() {
        // Test that ColorMode enum values are distinct
        assert_ne!(ColorMode::Auto, ColorMode::Always);
        assert_ne!(ColorMode::Auto, ColorMode::Never);
        assert_ne!(ColorMode::Always, ColorMode::Never);
    }
}
