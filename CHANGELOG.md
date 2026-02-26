# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-23

### Added

- Initial release of Unicleaner - malicious Unicode character detector
- **Core Scanner (US1: P1)**
  - Detection of zero-width characters (U+200B, U+200C, U+200D, U+FEFF)
  - Detection of bidirectional override characters (U+202A-U+202E) - Trojan Source attacks
  - Detection of homoglyphs from various scripts (Cyrillic, Greek, etc.)
  - Detection of non-printable control characters
  - Parallel file scanning with configurable thread count
  - Support for UTF-8, UTF-16 (LE/BE), and UTF-32 (LE/BE) encodings
  - Binary file detection and skipping
  - Precise violation reporting with file path, line number, column number

- **Configuration System (US2: P2)**
  - TOML-based configuration file support
  - Deny-by-default security model
  - 50+ language-specific presets (Rust, Python, JavaScript, etc.)
  - File-pattern-based rules using glob patterns
  - Custom Unicode range allowlists and denylists
  - Configuration validation and merging

- **Git/CI Integration (US3: P3)**
  - Git diff mode to scan only changed files
  - GitHub Actions output format (::error annotations)
  - GitLab CI output format (JSON)
  - Exit codes for CI/CD: 0 (clean), 1 (violations), 2 (error)
  - Example workflow files for GitHub Actions and GitLab CI

- **Reporting & Output (US4: P4)**
  - Human-readable colored terminal output
  - JSON output for machine parsing
  - Compact JSON mode for piping
  - TTY auto-detection for color output
  - NO_COLOR environment variable support
  - `--color` flag (auto/always/never)
  - `--severity` filtering (error/warning/info)
  - `--quiet` mode for summary only
  - `--verbose` mode for progress messages

- **CLI Commands**
  - `scan` - Scan files for malicious Unicode
  - `init` - Generate default configuration file
  - `list-presets` - Show available language presets

- **Quality & Testing**
  - 150 tests total (115 unit + 34 integration + 1 doc test)
  - Property-based testing with proptest
  - Fuzzing infrastructure for Unicode detection, config parsing, and file scanning
  - Performance benchmarks with criterion
  - Full clippy and rustfmt compliance

### Documentation

- Comprehensive README with usage examples
- Quickstart guide with common scenarios
- CLI reference documentation
- API documentation with rustdoc
- CI/CD integration examples
- Configuration examples

### Performance

- Parallel scanning using rayon
- Fast UTF-8 detection path
- BOM-based encoding detection
- Efficient Unicode range lookups

### Security

- Deny-by-default security model
- Comprehensive malicious Unicode detection
- Protection against Trojan Source attacks
- Safe encoding handling for all Unicode formats

[1.0.0]: https://github.com/poelzi/unicleaner/releases/tag/v1.0.0
