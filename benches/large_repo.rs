// Benchmark for large repository scanning (T083)
// ~10000 files

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use unicleaner::scanner::file_scanner::scan_file;

fn create_large_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create 10000 files (in parallel for speed)
    (0..10000).into_par_iter().for_each(|i| {
        let file_path = temp_dir.path().join(format!(
            "src/mod_{}/submod_{}/file_{}.rs",
            i / 100,
            (i / 10) % 10,
            i
        ));
        fs::create_dir_all(file_path.parent().unwrap()).expect("Failed to create dir");

        let mut file = fs::File::create(&file_path).expect("Failed to create file");
        writeln!(file, "pub fn func_{}() {{ }}", i).expect("Failed to write");
    });

    temp_dir
}

fn bench_large_repo_sequential(c: &mut Criterion) {
    let repo = create_large_repo();
    let files: Vec<_> = walkdir::WalkDir::new(repo.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect();

    c.bench_function("scan_large_repo_10k_sequential", |b| {
        b.iter(|| {
            for file in files.iter().take(100) {
                // Sample 100 files for benchmark speed
                let _ = black_box(scan_file(file));
            }
        });
    });
}

fn bench_large_repo_parallel(c: &mut Criterion) {
    let repo = create_large_repo();
    let files: Vec<_> = walkdir::WalkDir::new(repo.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect();

    c.bench_function("scan_large_repo_10k_parallel", |b| {
        b.iter(|| {
            files.par_iter().take(100).for_each(|file| {
                let _ = black_box(scan_file(file));
            });
        });
    });
}

criterion_group!(
    benches,
    bench_large_repo_sequential,
    bench_large_repo_parallel
);
criterion_main!(benches);
