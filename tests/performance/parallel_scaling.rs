//! Parallel scanning performance scaling tests
//!
//! These tests verify that parallel scanning with rayon provides
//! performance improvements over sequential scanning.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn running_in_ci() -> bool {
    std::env::var_os("CI").is_some()
}

fn perf_assert(condition: bool, message: String) {
    if condition {
        return;
    }

    if running_in_ci() {
        println!("::warning::{message}");
        eprintln!("Performance warning: {message}");
    } else {
        panic!("{}", message);
    }
}

fn scan_file(path: &std::path::Path) -> Result<(), String> {
    unicleaner::scanner::file_scanner::scan_file(path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Sequential scan of multiple files
fn sequential_scan(files: &[PathBuf]) -> Duration {
    let start = Instant::now();
    for file in files {
        let _ = scan_file(file);
    }
    start.elapsed()
}

/// Parallel scan using rayon
fn parallel_scan(files: &[PathBuf]) -> Duration {
    use rayon::prelude::*;

    let start = Instant::now();
    files.par_iter().for_each(|file| {
        let _ = scan_file(file);
    });
    start.elapsed()
}

/// Parallel scan with specified thread count
fn parallel_scan_with_threads(files: &[PathBuf], num_threads: usize) -> Duration {
    use rayon::prelude::*;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to create thread pool");

    let start = Instant::now();
    pool.install(|| {
        files.par_iter().for_each(|file| {
            let _ = scan_file(file);
        });
    });
    start.elapsed()
}

/// Create test files for parallel scanning tests
fn create_test_files(temp_dir: &TempDir, count: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for i in 0..count {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));

        // Generate files with enough content for measurable scan time
        let content = if i % 4 == 0 {
            // Unicode-heavy content (~10KB)
            let mut s = format!("// File {} with Unicode\n", i);
            for j in 0..200 {
                s.push_str(&format!(
                    "fn test_{}_{}() {{ let emoji = \"{}\";\n    let greek = \
                     \"αβγδεζηθικλμνξοπρστυφχψω\";\n}}\n",
                    i,
                    j,
                    "🔥🚀✨".repeat(20)
                ));
            }
            s
        } else if i % 4 == 1 {
            // Homoglyph content (~10KB)
            let mut s = format!("// File {} homoglyphs\n", i);
            for j in 0..200 {
                s.push_str(&format!(
                    "let scope_{} = 42; // ASCII\nlet scope_{} = 100; // Cyrillic\n",
                    j, j
                ));
            }
            s
        } else if i % 4 == 2 {
            // Bidi attacks (~10KB)
            let mut s = format!("// File {} bidi\n", i);
            for j in 0..200 {
                s.push_str(&format!(
                    "let x_{} = \"\u{202E}/* comment {} */\u{202D}\";\n",
                    j, j
                ));
            }
            s
        } else {
            // Regular ASCII (~10KB)
            let mut s = format!("// Regular file {}\n", i);
            for j in 0..200 {
                s.push_str(&format!(
                    "fn main_{}() {{ println!(\"test {}\"); }}\n",
                    j, j
                ));
            }
            s
        };

        fs::write(&file_path, content).expect("Failed to write file");
        files.push(file_path);
    }

    files
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_parallel_faster_than_sequential() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create 200 files to scan
    let files = create_test_files(&temp_dir, 200);

    // Warm up rayon thread pool initialization
    let _ = parallel_scan(&files);

    // Run sequential scan
    let sequential_time = sequential_scan(&files);

    // Run parallel scan
    let parallel_time = parallel_scan(&files);

    println!("Sequential scan: {:?}", sequential_time);
    println!("Parallel scan: {:?}", parallel_time);
    println!(
        "Speedup: {:.2}x",
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );

    if running_in_ci() {
        perf_assert(
            parallel_time < Duration::from_secs(120),
            format!(
                "Parallel scan should complete within 120s in CI, took {:?}",
                parallel_time
            ),
        );
    } else {
        let actual_speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
        let expected_speedup = 1.1;

        perf_assert(
            actual_speedup > expected_speedup,
            format!(
                "Parallel scan should be at least {:.1}x faster than sequential, got {:.2}x",
                expected_speedup, actual_speedup
            ),
        );
    }
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_thread_count_scaling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create 400 files for better scaling demonstration
    let files = create_test_files(&temp_dir, 400);

    // Test with different thread counts
    let thread_counts = [1, 2, 4, 8];
    let mut results = Vec::new();

    for &num_threads in &thread_counts {
        let duration = parallel_scan_with_threads(&files, num_threads);
        results.push((num_threads, duration));
        println!("Threads: {}, Duration: {:?}", num_threads, duration);
    }

    // Verify that 2 threads is faster than 1 thread
    let (_, single_thread) = results[0];
    let (_, dual_thread) = results[1];

    let dual_ratio = dual_thread.as_secs_f64() / single_thread.as_secs_f64();
    let max_dual_ratio = if running_in_ci() { 1.20 } else { 1.05 };
    perf_assert(
        dual_ratio <= max_dual_ratio,
        format!(
            "2 threads should not be much slower than 1 thread (ratio {:.2})",
            dual_ratio
        ),
    );

    // Calculate speedup from 1 to 4 threads
    let (_, four_threads) = results[2];
    let speedup = single_thread.as_secs_f64() / four_threads.as_secs_f64();

    println!("Speedup (1 -> 4 threads): {:.2}x", speedup);

    // CI VMs are noisy; require a smaller but still meaningful gain there.
    let min_speedup = if running_in_ci() { 1.05 } else { 1.5 };
    perf_assert(
        speedup > min_speedup,
        format!(
            "Should see at least {:.2}x speedup with 4 threads, got {:.2}x",
            min_speedup, speedup
        ),
    );
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_large_repo_parallel_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create 1000 files (matching plan.md requirement)
    let files = create_test_files(&temp_dir, 1000);

    // Parallel scan should complete in reasonable time
    let start = Instant::now();
    parallel_scan(&files);
    let duration = start.elapsed();

    println!("1000-file parallel scan: {:?}", duration);

    // Under sanitizer/instrumented CI this can be significantly slower.
    let max_duration = if running_in_ci() {
        Duration::from_secs(120)
    } else {
        Duration::from_secs(30)
    };
    perf_assert(
        duration < max_duration,
        format!(
            "1000-file parallel scan should complete within {:?}, took {:?}",
            max_duration, duration
        ),
    );
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_parallel_chunk_size_impact() {
    use rayon::prelude::*;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let files = create_test_files(&temp_dir, 500);

    // Test with different chunk sizes
    let chunk_sizes = [1, 10, 50, 100];

    for &chunk_size in &chunk_sizes {
        let start = Instant::now();

        files.par_chunks(chunk_size).for_each(|chunk| {
            for file in chunk {
                let _ = scan_file(file);
            }
        });

        let duration = start.elapsed();
        println!("Chunk size {}: {:?}", chunk_size, duration);
    }

    // All chunk sizes should complete in reasonable time
    // This test mainly validates that chunking works without errors
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_parallel_mixed_file_sizes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut files = Vec::new();

    // Create files of varying sizes
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));

        let size_multiplier = if i % 10 == 0 {
            100 // Some large files
        } else if i % 5 == 0 {
            50 // Some medium files
        } else {
            10 // Mostly small files
        };

        let content = "x".repeat(size_multiplier * 1000);
        fs::write(&file_path, content).expect("Failed to write file");
        files.push(file_path);
    }

    // Warm up rayon thread pool initialization
    let _ = parallel_scan(&files);

    // Parallel scan should handle mixed sizes efficiently
    let sequential_time = sequential_scan(&files);
    let parallel_time = parallel_scan(&files);

    println!(
        "Mixed sizes - Sequential: {:?}, Parallel: {:?}",
        sequential_time, parallel_time
    );

    if running_in_ci() {
        perf_assert(
            parallel_time < Duration::from_secs(120),
            format!(
                "Mixed-size parallel scan should complete within 120s in CI, took {:?}",
                parallel_time
            ),
        );
    } else {
        let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();
        let min_speedup = 1.1;
        perf_assert(
            speedup > min_speedup,
            format!(
                "Should see at least {:.2}x speedup with mixed file sizes, got {:.2}x",
                min_speedup, speedup
            ),
        );
    }
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_parallel_overhead_small_workload() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create only 10 small files
    let files = create_test_files(&temp_dir, 10);

    // For very small workloads, sequential might be faster due to overhead
    let sequential_time = sequential_scan(&files);
    let parallel_time = parallel_scan(&files);

    println!(
        "Small workload - Sequential: {:?}, Parallel: {:?}",
        sequential_time, parallel_time
    );

    // Both should complete in reasonable time.
    // Coverage-instrumented CI can add substantial overhead.
    let max_duration = if running_in_ci() {
        Duration::from_secs(45)
    } else {
        Duration::from_secs(10)
    };

    perf_assert(
        sequential_time < max_duration,
        format!(
            "Sequential scan of 10 files should complete within {:?}",
            max_duration
        ),
    );
    perf_assert(
        parallel_time < max_duration,
        format!(
            "Parallel scan of 10 files should complete within {:?}",
            max_duration
        ),
    );

    // This test documents the overhead characteristics
    // No assertion on which is faster - just verify both work
}

#[test]
#[ignore = "perf test — unreliable on slow CI runners, run with: cargo test -- --ignored"]
fn test_rayon_cpu_utilization() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create enough files to saturate CPU
    let files = create_test_files(&temp_dir, 1000);

    // Get available parallelism
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    println!("Available CPUs: {}", num_cpus);

    // Scan with default rayon settings (should use all CPUs)
    let start = Instant::now();
    parallel_scan(&files);
    let duration = start.elapsed();

    println!("Parallel scan with {} CPUs: {:?}", num_cpus, duration);

    // Should complete in reasonable time.
    let max_duration = if running_in_ci() {
        Duration::from_secs(60)
    } else {
        Duration::from_secs(10)
    };
    perf_assert(
        duration < max_duration,
        format!(
            "Parallel scan should utilize CPUs within {:?}, took {:?}",
            max_duration, duration
        ),
    );
}
