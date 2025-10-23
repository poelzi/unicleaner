#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as TOML config
    if let Ok(text) = std::str::from_utf8(data) {
        // Attempt to parse as TOML - should never panic
        let _result: Result<unicleaner::config::Config, _> = toml::from_str(text);

        // We don't care if parsing succeeds or fails, just that it doesn't panic
        // Invalid TOML should return Err, not panic
    }
});
