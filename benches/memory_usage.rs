// Benchmark for memory usage (T085)

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Write;
use tempfile::NamedTempFile;
use unicleaner::scanner::file_scanner::scan_file;

fn create_large_file(size_kb: usize) -> NamedTempFile {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    let line = "fn test() { let x = 42; println!(\"test\"); }\n";
    let lines_needed = (size_kb * 1024) / line.len();

    for _ in 0..lines_needed {
        write!(temp, "{}", line).expect("Failed to write");
    }

    temp.flush().expect("Failed to flush");
    temp
}

fn bench_memory_100kb(c: &mut Criterion) {
    let file = create_large_file(100);

    c.bench_function("scan_100kb_file", |b| {
        b.iter(|| {
            let _ = black_box(scan_file(file.path()));
        });
    });
}

fn bench_memory_1mb(c: &mut Criterion) {
    let file = create_large_file(1024);

    c.bench_function("scan_1mb_file", |b| {
        b.iter(|| {
            let _ = black_box(scan_file(file.path()));
        });
    });
}

fn bench_memory_10mb(c: &mut Criterion) {
    let file = create_large_file(10 * 1024);

    c.bench_function("scan_10mb_file", |b| {
        b.iter(|| {
            let _ = black_box(scan_file(file.path()));
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);  // Fewer samples for large files
    targets = bench_memory_100kb, bench_memory_1mb, bench_memory_10mb
);
criterion_main!(benches);
