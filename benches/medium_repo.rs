// Benchmark for medium repository scanning (T082)
// ~1000 files

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::io::Write;
use tempfile::TempDir;
use unicleaner::scanner::file_scanner::scan_file;

fn create_medium_repo(num_files: usize) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    for i in 0..num_files {
        let file_path = temp_dir
            .path()
            .join(format!("src/module_{}/file_{}.rs", i / 10, i));
        fs::create_dir_all(file_path.parent().unwrap()).expect("Failed to create dir");

        let mut file = fs::File::create(&file_path).expect("Failed to create file");

        writeln!(file, "// Module file {}", i).expect("Failed to write");
        writeln!(file, "pub struct Data{} {{", i).expect("Failed to write");
        writeln!(file, "    value: i32,").expect("Failed to write");
        writeln!(file, "    name: String,").expect("Failed to write");
        writeln!(file, "}}").expect("Failed to write");
        writeln!(file).expect("Failed to write");
        writeln!(file, "impl Data{} {{", i).expect("Failed to write");
        writeln!(file, "    pub fn new(value: i32) -> Self {{").expect("Failed to write");
        writeln!(file, "        Self {{").expect("Failed to write");
        writeln!(file, "            value,").expect("Failed to write");
        writeln!(file, "            name: String::from(\"data\"),").expect("Failed to write");
        writeln!(file, "        }}").expect("Failed to write");
        writeln!(file, "    }}").expect("Failed to write");
        writeln!(file, "}}").expect("Failed to write");
    }

    temp_dir
}

fn bench_medium_repo(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_repo");

    for size in [500, 1000].iter() {
        let repo = create_medium_repo(*size);
        let files: Vec<_> = walkdir::WalkDir::new(repo.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for file in &files {
                    let _ = black_box(scan_file(file));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_medium_repo);
criterion_main!(benches);
