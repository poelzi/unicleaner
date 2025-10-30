// Benchmark for Unicode-heavy files (T084)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Write;
use tempfile::NamedTempFile;
use unicleaner::scanner::file_scanner::scan_file;

fn create_unicode_heavy_file() -> NamedTempFile {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // Create file with lots of Unicode characters
    for i in 0..1000 {
        writeln!(temp, "// 测试中文字符 🌍🚀✨ Test {}", i).expect("Failed to write");
        writeln!(temp, "pub fn test_{}() {{", i).expect("Failed to write");
        writeln!(temp, "    let message = \"你好世界 Hello World\";").expect("Failed to write");
        writeln!(temp, "    println!(\"{{}} 🎉\", message);").expect("Failed to write");
        writeln!(temp, "}}").expect("Failed to write");
    }

    temp.flush().expect("Failed to flush");
    temp
}

fn create_mixed_scripts_file() -> NamedTempFile {
    let mut temp = NamedTempFile::new().expect("Failed to create temp file");

    // Mix of Latin, Cyrillic, Greek, Arabic, etc.
    for i in 0..500 {
        writeln!(temp, "// Αλφα Бета Гамма Δελτα {}", i).expect("Failed to write");
        writeln!(temp, "pub fn test_{}() {{", i).expect("Failed to write");
        writeln!(temp, "    let x = \"{} العربية עברית\";", i).expect("Failed to write");
        writeln!(temp, "}}").expect("Failed to write");
    }

    temp.flush().expect("Failed to flush");
    temp
}

fn bench_unicode_heavy(c: &mut Criterion) {
    let file = create_unicode_heavy_file();

    c.bench_function("scan_unicode_heavy_1000_lines", |b| {
        b.iter(|| {
            let _ = black_box(scan_file(file.path()));
        });
    });
}

fn bench_mixed_scripts(c: &mut Criterion) {
    let file = create_mixed_scripts_file();

    c.bench_function("scan_mixed_scripts_500_lines", |b| {
        b.iter(|| {
            let _ = black_box(scan_file(file.path()));
        });
    });
}

criterion_group!(benches, bench_unicode_heavy, bench_mixed_scripts);
criterion_main!(benches);
