#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;
use unicleaner::scanner::file_scanner::scan_file;

fuzz_target!(|data: &[u8]| {
    // Create a temporary file with the fuzz data
    if let Ok(mut temp_file) = NamedTempFile::new() {
        // Write fuzzer data to file
        let _ = temp_file.write_all(data);
        let _ = temp_file.flush();

        // Try to scan the file - should never panic regardless of content
        let _result = scan_file(temp_file.path());

        // We don't care about the result, just that scanning doesn't panic
        // Invalid encoding, binary data, malformed UTF-8 should all be handled gracefully
    }
});
