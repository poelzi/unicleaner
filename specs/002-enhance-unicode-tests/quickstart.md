# Quickstart for Enhanced Unicode Tests

## Running the Tests

1. Ensure Rust and Cargo are installed.
2. Navigate to the project root.
3. Run `cargo test` to execute the full test suite, including new Unicode attack tests.
4. For fuzz testing: `cargo fuzz run fuzz_unicode_detector` (assuming fuzz target setup).

## Adding New Test Cases

- Create fixtures in `tests/integration/fixtures/` with malicious Unicode examples.
- Add integration tests in `tests/integration/scan_tests.rs` to verify detection.