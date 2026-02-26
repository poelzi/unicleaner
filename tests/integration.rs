// Include all integration test modules
#[path = "integration/trojan_source_tests.rs"]
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

#[path = "integration/diff_tests.rs"]
mod diff_tests;

#[path = "integration/block_config_tests.rs"]
mod block_config_tests;

#[path = "contract/exit_codes.rs"]
mod exit_codes;

#[path = "regression/mod.rs"]
mod regression;

#[path = "performance/benchmarks.rs"]
mod perf_benchmarks;

#[path = "performance/memory.rs"]
mod perf_memory;

#[path = "performance/time_limits.rs"]
mod perf_time_limits;

#[path = "performance/memory_limits.rs"]
mod perf_memory_limits;

#[path = "performance/parallel_scaling.rs"]
mod perf_parallel_scaling;
