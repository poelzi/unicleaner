# Fuzzing Setup

This directory contains fuzzing infrastructure for unicleaner using `cargo-fuzz`.

## Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Ensure you're using nightly Rust (required for fuzzing)
rustup default nightly
```

## Fuzz Targets

### unicode_detection.rs
Fuzzes the Unicode character detection logic with arbitrary strings.

```bash
cargo fuzz run fuzz_unicode
```

### file_scanner.rs
Fuzzes the file scanning functionality with arbitrary file content.

```bash
cargo fuzz run fuzz_file_scan
```

### config_parser.rs
Fuzzes the TOML configuration parser with arbitrary TOML content.

```bash
cargo fuzz run fuzz_config
```

### encoding_detection.rs
Fuzzes encoding detection with arbitrary byte sequences.

```bash
cargo fuzz run encoding_detection
```

### homoglyph_detector.rs
Fuzzes homoglyph detection with arbitrary Unicode strings.

```bash
cargo fuzz run homoglyph_detector
```

## Running Fuzz Tests

### Quick test (10 seconds)
```bash
cargo fuzz run fuzz_unicode -- -max_total_time=10
```

### Longer fuzzing session (1 hour)
```bash
cargo fuzz run fuzz_unicode -- -max_total_time=3600
```

### With specific corpus
```bash
cargo fuzz run fuzz_unicode corpus/unicode/
```

### Parallel fuzzing (4 jobs)
```bash
cargo fuzz run fuzz_unicode -- -jobs=4
```

## Corpus

The `corpus/` directory contains initial test cases to seed the fuzzer:

- `corpus/unicode/` - Unicode attack patterns
- `corpus/config/` - TOML configuration samples
- `corpus/encoding/` - Various encoding samples

Fuzz testing will automatically discover new interesting inputs and save them to the corpus.

## Dictionary

`dictionary.txt` contains common attack patterns and keywords to guide the fuzzer:

- Bidi control characters
- Zero-width characters
- Common identifier names
- TOML keywords

## Artifacts

If fuzzing finds a crash or hang, it will be saved in `fuzz/artifacts/`:

```
fuzz/artifacts/
├── fuzz_unicode/
│   ├── crash-abc123
│   └── timeout-def456
└── fuzz_config/
    └── crash-xyz789
```

### Reproducing crashes

```bash
cargo fuzz run fuzz_unicode fuzz/artifacts/fuzz_unicode/crash-abc123
```

## CI Integration

For CI pipelines, run fuzzing with a time limit:

```bash
# Run for 5 minutes in CI
cargo fuzz run fuzz_unicode -- -max_total_time=300
```

## Coverage

To see code coverage from fuzzing:

```bash
cargo fuzz coverage fuzz_unicode
```

## Tips

1. **Start with short runs**: Begin with 10-second runs to ensure setup is correct
2. **Use dictionaries**: The dictionary.txt file guides the fuzzer toward interesting inputs
3. **Monitor memory**: Fuzzing can use significant RAM, especially for large inputs
4. **Parallel fuzzing**: Use `-jobs=N` for faster coverage
5. **Minimize crashes**: Use `cargo fuzz cmin` to minimize corpus

## Minimizing Crashes

If a crash is found, minimize it to the smallest reproducing input:

```bash
cargo fuzz tmin fuzz_unicode fuzz/artifacts/fuzz_unicode/crash-abc123
```

## Cleaning Up

Remove corpus and artifacts:

```bash
rm -rf corpus/ artifacts/
```

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing best practices](https://github.com/rust-fuzz/trophy-case)
