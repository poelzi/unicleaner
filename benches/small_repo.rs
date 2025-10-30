// Benchmark for small repository scanning (T081)
// ~100 files

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use unicleaner::scanner::file_scanner::scan_file;

fn create_small_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create 100 small files
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("file_{}.rs", i));
        let mut file = fs::File::create(&file_path).expect("Failed to create file");

        writeln!(file, "// File {}", i).expect("Failed to write");
        writeln!(file, "pub fn test_{}() {{", i).expect("Failed to write");
        writeln!(file, "    let x = {};", i).expect("Failed to write");
        writeln!(file, "    println!(\"Value: {{}}\", x);").expect("Failed to write");
        writeln!(file, "}}").expect("Failed to write");
    }

    temp_dir
}

fn bench_small_repo(c: &mut Criterion) {
    let repo = create_small_repo();
    let files: Vec<_> = fs::read_dir(repo.path())
        .expect("Failed to read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    c.bench_function("scan_small_repo_100_files", |b| {
        b.iter(|| {
            for file in &files {
                let _ = black_box(scan_file(file));
            }
        });
    });
}

criterion_group!(benches, bench_small_repo);
criterion_main!(benches);
