// Integration test module registration
// This file enables Cargo to discover and run integration tests in the integration/ subdirectory

mod integration {
    mod config_tests;
    mod encoding_tests;
    mod scan_tests;
}
