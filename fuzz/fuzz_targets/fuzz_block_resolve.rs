#![no_main]

use libfuzzer_sys::fuzz_target;
use unicleaner::unicode::blocks::BlockRegistry;

fuzz_target!(|data: &[u8]| {
    // Feed arbitrary bytes as a block name to resolve
    if let Ok(name) = std::str::from_utf8(data) {
        // resolve() must never panic - it should return Ok or Err gracefully
        let _result = BlockRegistry::resolve(name);
    }
});
