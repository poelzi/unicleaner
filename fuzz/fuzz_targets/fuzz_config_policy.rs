//! Fuzz target for configuration policy enforcement
//! Tests that security policy decisions are sound with arbitrary inputs

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;
use unicleaner::config::Configuration;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 8 {
        // Split data into path and code point
        let split_point = (data[0] as usize % data.len()).max(4);

        // Create arbitrary path from first part of data
        if let Ok(path_str) = std::str::from_utf8(&data[1..split_point]) {
            let path = PathBuf::from(path_str);

            // Create code point from remaining data
            let remaining = &data[split_point..];
            if remaining.len() >= 4 {
                let code_point =
                    u32::from_le_bytes([remaining[0], remaining[1], remaining[2], remaining[3]]);

                // Test with default config
                let config = Configuration::default();
                let _ = config.is_code_point_allowed(&path, code_point);
                let _ = config.get_allowed_ranges(&path);

                // Test with deny_by_default = false
                let mut config2 = Configuration::default();
                config2.deny_by_default = false;
                let _ = config2.is_code_point_allowed(&path, code_point);

                // Test merging configurations
                let mut config3 = Configuration::default();
                config3.merge(config.clone());
                let _ = config3.is_code_point_allowed(&path, code_point);

                // Test with boundary code points
                let _ = config.is_code_point_allowed(&path, 0);
                let _ = config.is_code_point_allowed(&path, 0x7F);
                let _ = config.is_code_point_allowed(&path, 0x10FFFF);
                let _ = config.is_code_point_allowed(&path, u32::MAX);
            }
        }
    }
});
