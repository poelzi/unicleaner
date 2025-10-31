//! Fuzz target for Unicode range operations (integer overflow and boundary
//! conditions) Tests that range arithmetic never panics with extreme values

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicleaner::unicode::ranges::UnicodeRange;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 8 {
        // Extract two u32 values for range boundaries
        let start1 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let end1 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        // Test range creation with arbitrary values
        let range1 = UnicodeRange::new(start1, end1);

        // Test contains with boundary values
        let _ = range1.contains(0);
        let _ = range1.contains(0x7F);
        let _ = range1.contains(0x10FFFF);
        let _ = range1.contains(u32::MAX);

        // Test with the range's own boundaries
        let _ = range1.contains(start1);
        let _ = range1.contains(end1);

        // If we have more data, test range operations
        if data.len() >= 16 {
            let start2 = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
            let end2 = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
            let range2 = UnicodeRange::new(start2, end2);

            // Test intersection
            let _ = range1.intersects(&range2);
            let _ = range2.intersects(&range1);

            // Test merging
            let _ = range1.merge(&range2);
            let _ = range2.merge(&range1);
        }

        // Test with description
        if data.len() >= 20 {
            if let Ok(desc) = std::str::from_utf8(&data[16..20]) {
                let _ = UnicodeRange::with_description(start1, end1, desc.to_string());
            }
        }
    }
});
