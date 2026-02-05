// Include all integration test modules
mod trojan_source_tests;

// Include the integration test subdirectory modules
#[path = "integration/homoglyph_tests.rs"]
mod homoglyph_tests;

#[path = "integration/zero_width_tests.rs"]
mod zero_width_tests;

#[path = "integration/normalization_tests.rs"]
mod normalization_tests;

#[path = "integration/encoding_tests.rs"]
mod encoding_tests;

#[path = "integration/config_tests.rs"]
mod config_tests;

#[path = "integration/scan_tests.rs"]
mod scan_tests;

#[path = "integration/output_tests.rs"]
mod output_tests;

#[path = "integration/edge_cases.rs"]
mod edge_cases;

#[path = "integration/block_config_tests.rs"]
mod block_config_tests;
