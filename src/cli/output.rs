use std::io::{self, IsTerminal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorStream {
    Stdout,
    Stderr,
}

/// Determine if colors should be used based on mode and stream
pub fn should_use_color(mode: ColorMode, stream: ColorStream) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            // Check NO_COLOR environment variable first
            // https://no-color.org/
            if is_no_color_set() {
                return false;
            }

            // Then check if the stream is a terminal
            match stream {
                ColorStream::Stdout => io::stdout().is_terminal(),
                ColorStream::Stderr => io::stderr().is_terminal(),
            }
        }
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
        assert!(should_use_color(ColorMode::Always, ColorStream::Stdout));
        assert!(should_use_color(ColorMode::Always, ColorStream::Stderr));
    }

    #[test]
    fn test_color_mode_never() {
        assert!(!should_use_color(ColorMode::Never, ColorStream::Stdout));
        assert!(!should_use_color(ColorMode::Never, ColorStream::Stderr));
    }

    #[test]
    #[serial_test::serial]
    fn test_color_mode_auto_with_no_color() {
        // Save original NO_COLOR state
        let original = env::var("NO_COLOR").ok();

        // Test 1: NO_COLOR set to any value (e.g., "1")
        // According to https://no-color.org/, any value means colors disabled
        // SAFETY: This test is run serially (#[serial_test::serial]) so no
        // other threads are concurrently reading environment variables.
        unsafe { env::set_var("NO_COLOR", "1") };
        assert!(
            is_no_color_set(),
            "NO_COLOR standard: is_no_color_set() should return true when NO_COLOR='1'"
        );

        // Test 2: NO_COLOR set to empty string
        // Per NO_COLOR standard, even empty value means colors disabled
        unsafe { env::set_var("NO_COLOR", "") };
        assert!(
            is_no_color_set(),
            "NO_COLOR standard: is_no_color_set() should return true when NO_COLOR='' (empty)"
        );

        // Test 3: NO_COLOR unset
        unsafe { env::remove_var("NO_COLOR") };
        assert!(
            !is_no_color_set(),
            "NO_COLOR standard: is_no_color_set() should return false when NO_COLOR is unset"
        );

        // Restore original state
        match original {
            Some(val) => unsafe { env::set_var("NO_COLOR", val) },
            None => unsafe { env::remove_var("NO_COLOR") },
        }
    }

    #[test]
    fn test_is_no_color_set() {
        // This test just checks the function works without modifying env
        // The actual behavior is tested in test_color_mode_auto_with_no_color
        let _result = is_no_color_set();
        // Just ensure it doesn't panic
    }
}
