//! Fuzz target for encoding detection (T053)
//! Tests that encoding detection never panics on arbitrary byte sequences

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to detect if data is binary - should never panic
    let _ = unicleaner::scanner::encoding::is_binary(data);

    // Try to decode the bytes - should handle any encoding gracefully
    let _ = unicleaner::scanner::encoding::detect_and_decode(data);

    // Try to decode with encoding info - should never panic
    let _ = unicleaner::scanner::encoding::detect_decode_with_encoding(data);
});
