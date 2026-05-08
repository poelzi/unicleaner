//! Throughput benchmark for `unicleaner::cleaner::clean`.
//!
//! Two cases:
//! 1. **fast path** — 4 MiB of clean ASCII. Measures the
//!    `Cow::Borrowed` no-allocation path and the `needs_mutation`
//!    pre-scan cost.
//! 2. **steady state** — 4 MiB of mostly-clean text seeded with
//!    ~1 % zero-width-space violations. Measures the cleaning
//!    hot loop.
//!
//! Per spec.md SC-003, the strict-mode throughput target is
//! ≥ 200 MiB/s on the project's reference benchmark host. The
//! benchmark itself does not assert a hard floor — that gate is
//! enforced by Criterion's regression baseline in CI.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use unicleaner::cleaner::{CleanPolicy, clean};

const SIZE: usize = 4 * 1024 * 1024; // 4 MiB

fn build_clean_input() -> String {
    // Repeat a printable ASCII line until we hit SIZE bytes.
    let line = "fn main() { println!(\"hello, world!\"); } // ascii filler\n";
    let mut out = String::with_capacity(SIZE + line.len());
    while out.len() < SIZE {
        out.push_str(line);
    }
    out.truncate(SIZE);
    out
}

fn build_dirty_input() -> String {
    // Sprinkle a U+200B every ~100 chars to give ~1 % violation density.
    let mut out = String::with_capacity(SIZE + 64);
    let mut counter: usize = 0;
    while out.len() < SIZE {
        out.push_str("the quick brown fox jumps over the lazy dog ");
        counter += 1;
        if counter % 2 == 0 {
            out.push('\u{200B}');
        }
        out.push('\n');
    }
    out.truncate(SIZE);
    out
}

fn bench_clean_fast_path(c: &mut Criterion) {
    let input = build_clean_input();
    let policy = CleanPolicy::strict();

    let mut group = c.benchmark_group("clean_throughput");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("strict_clean_ascii_4MiB", |b| {
        b.iter(|| {
            let r = clean(black_box(&input), &policy);
            black_box(r);
        });
    });
    group.finish();
}

fn bench_clean_steady_state(c: &mut Criterion) {
    let input = build_dirty_input();
    let policy = CleanPolicy::strict();

    let mut group = c.benchmark_group("clean_throughput");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("strict_dirty_1pct_4MiB", |b| {
        b.iter(|| {
            let r = clean(black_box(&input), &policy);
            black_box(r);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_clean_fast_path, bench_clean_steady_state);
criterion_main!(benches);
