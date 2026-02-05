//! Performance benchmarks for unicleaner scanning operations

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use unicleaner::scanner::encoding::detect_and_decode;
use unicleaner::scanner::file_scanner::scan_file;
use unicleaner::scanner::unicode_detector::detect_in_string;

/// Create a test file with the given content
fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

fn benchmark_unicode_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode_detection");
    let test_path = PathBuf::from("test.rs");

    // Benchmark clean ASCII text
    let clean_ascii = "fn main() {\n    println!(\"Hello, world!\");\n}\n".repeat(100);
    group.bench_function("clean_ascii_1KB", |b| {
        b.iter(|| detect_in_string(black_box(&clean_ascii), &test_path))
    });

    // Benchmark text with malicious Unicode
    let malicious =
        "fn main() {\u{200B}\n    println!(\"Hello\u{202E}, world!\");\n}\n".repeat(100);
    group.bench_function("malicious_1KB", |b| {
        b.iter(|| detect_in_string(black_box(&malicious), &test_path))
    });

    // Benchmark larger files
    let large_clean = "// This is a comment\nfn test() { let x = 42; }\n".repeat(1000);
    group.bench_function("clean_ascii_20KB", |b| {
        b.iter(|| detect_in_string(black_box(&large_clean), &test_path))
    });

    // Benchmark with legitimate Unicode
    let unicode_text = "各国語のテキスト\nΕλληνικά κείμενο\nРусский текст\n".repeat(100);
    group.bench_function("legitimate_unicode_5KB", |b| {
        b.iter(|| detect_in_string(black_box(&unicode_text), &test_path))
    });

    group.finish();
}

fn benchmark_encoding_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_detection");

    // Benchmark UTF-8 detection (fast path)
    let utf8_bytes = b"fn main() {\n    println!(\"Hello, world!\");\n}\n";
    group.bench_function("utf8_fast_path", |b| {
        b.iter(|| detect_and_decode(black_box(utf8_bytes)))
    });

    // Benchmark UTF-16 LE with BOM
    let utf16_le: Vec<u8> = vec![
        0xFF, 0xFE, // BOM
        0x66, 0x00, 0x6E, 0x00, // "fn"
        0x20, 0x00, 0x6D, 0x00, // " m"
    ];
    group.bench_function("utf16_le_with_bom", |b| {
        b.iter(|| detect_and_decode(black_box(&utf16_le)))
    });

    // Benchmark larger UTF-8 file
    let large_utf8 = "// Comment\nfn test() { let x = 42; }\n".repeat(1000);
    group.bench_function("utf8_20KB", |b| {
        b.iter(|| detect_and_decode(black_box(large_utf8.as_bytes())))
    });

    group.finish();
}

fn benchmark_file_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_scanning");
    let temp_dir = TempDir::new().unwrap();

    // Small file
    let small_file = create_test_file(
        &temp_dir,
        "small.rs",
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    );
    group.bench_function("small_file_100B", |b| {
        b.iter(|| scan_file(black_box(&small_file)))
    });

    // Medium file with malicious Unicode
    let medium_content = format!("fn test{} {{\u{200B}\n    let x = {};\n}}\n", 1, 42).repeat(100);
    let medium_file = create_test_file(&temp_dir, "medium.rs", &medium_content);
    group.bench_function("medium_file_5KB", |b| {
        b.iter(|| scan_file(black_box(&medium_file)))
    });

    // Large file
    let large_content =
        "// This is a comment\nfn test() {\n    let x = 42;\n    let y = x + 1;\n}\n".repeat(1000);
    let large_file = create_test_file(&temp_dir, "large.rs", &large_content);
    group.bench_function("large_file_100KB", |b| {
        b.iter(|| scan_file(black_box(&large_file)))
    });

    group.finish();
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    let test_path = PathBuf::from("test.rs");

    // Test how detection scales with file size
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let content = "fn test() { let x = 42; }\n".repeat(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| detect_in_string(black_box(&content), &test_path))
        });
    }

    group.finish();
}

fn benchmark_worst_case(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst_case");
    let test_path = PathBuf::from("test.rs");

    // File with many malicious characters (worst case for detection)
    let worst_case = "\u{200B}".repeat(1000);
    group.bench_function("1000_zero_width_chars", |b| {
        b.iter(|| detect_in_string(black_box(&worst_case), &test_path))
    });

    // Mixed malicious patterns
    let mixed = format!(
        "{}\u{200B}{}\u{202E}{}\u{200C}{}",
        "code", "more", "text", "end"
    )
    .repeat(100);
    group.bench_function("mixed_malicious_patterns", |b| {
        b.iter(|| detect_in_string(black_box(&mixed), &test_path))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_unicode_detection,
    benchmark_encoding_detection,
    benchmark_file_scanning,
    benchmark_scaling,
    benchmark_worst_case
);
criterion_main!(benches);
