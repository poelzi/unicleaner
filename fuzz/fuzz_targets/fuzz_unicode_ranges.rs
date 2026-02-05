//! Fuzz target for Unicode range operations (integer overflow and boundary
//! conditions) Tests that range arithmetic never panics with valid inputs

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicleaner::unicode::ranges::UnicodeRange;

/// Normalize a raw u32 to a valid Unicode code point (0..=0x10FFFF)
fn to_code_point(raw: u32) -> u32 {
    raw % (0x10FFFF + 1)
}

fuzz_target!(|data: &[u8]| {
    if data.len() >= 8 {
        // Extract two u32 values and normalize to valid Unicode code points
        let raw1 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let raw2 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let cp1 = to_code_point(raw1);
        let cp2 = to_code_point(raw2);
        let start1 = cp1.min(cp2);
        let end1 = cp1.max(cp2);

        // Test range creation with valid values
        let range1 = UnicodeRange::new(start1, end1);

        // Test contains with boundary values
        let _ = range1.contains(0u32);
        let _ = range1.contains(0x7Fu32);
        let _ = range1.contains(0x10FFFFu32);

        // Test with the range's own boundaries
        let _ = range1.contains(start1);
        let _ = range1.contains(end1);

        // If we have more data, test range operations
        if data.len() >= 16 {
            let raw3 = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
            let raw4 = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
            let cp3 = to_code_point(raw3);
            let cp4 = to_code_point(raw4);
            let start2 = cp3.min(cp4);
            let end2 = cp3.max(cp4);
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

        // Also test contains with arbitrary fuzzed code points
        if data.len() >= 12 {
            let raw_cp = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
            let _ = range1.contains(to_code_point(raw_cp));
        }
    }
});
