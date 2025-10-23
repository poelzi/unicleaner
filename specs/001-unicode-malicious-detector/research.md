# Research: Unicode Malicious Character Detector

**Phase**: 0 - Research & Technology Selection  
**Date**: 2025-10-23  
**Purpose**: Resolve technology choices and research best practices for Unicode security scanning

## Research Questions

From Technical Context NEEDS CLARIFICATION:
1. CLI parsing library
2. TOML parsing library
3. Unicode data source and libraries
4. Git integration approach
5. Encoding detection library
6. Color output library
7. Testing infrastructure details

---

## 1. CLI Argument Parsing

### Decision: **clap v4**

### Rationale:
- Most popular Rust CLI parsing library (industry standard)
- Derives from structs (type-safe, compile-time checked)
- Built-in --help generation with good formatting
- Supports subcommands, multiple value types, validation
- Good error messages for invalid arguments
- Shell completion generation (bash, zsh, fish)
- Actively maintained with strong ecosystem

### Alternatives Considered:
- **structopt**: Merged into clap v3+, now deprecated
- **argh**: Lightweight but less features, not suitable for complex CLI
- **pico-args**: Minimal, manual parsing, not ergonomic for this use case

### Implementation Notes:
```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "unicleaner")]
#[command(about = "Detect malicious Unicode in source code")]
struct Cli {
    #[arg(help = "Path to scan (directory, file, or Git repo)")]
    path: PathBuf,
    
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,
    
    #[arg(long)]
    json: bool,
    
    #[arg(long, short = 'c')]
    config: Option<PathBuf>,
}
```

---

## 2. TOML Configuration Parsing

### Decision: **toml v0.8** + **serde v1**

### Rationale:
- `toml` crate is the de-facto standard for TOML in Rust
- Integrates seamlessly with serde for (de)serialization
- Good error messages for invalid TOML
- Supports TOML 1.0.0 specification
- Type-safe deserialization into Rust structs
- Well-maintained and stable

### Alternatives Considered:
- **toml_edit**: For preserving formatting/comments, not needed here
- **config**: Multi-format config library, overkill for TOML-only

### Implementation Notes:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    global: GlobalRules,
    presets: Vec<String>,
    file_rules: Vec<FileRule>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileRule {
    pattern: String,
    allowed_ranges: Vec<UnicodeRange>,
}
```

---

## 3. Unicode Data and Detection

### Decision: **unicode-normalization v0.1** + **unicode-segmentation v1.10** + **Custom malicious pattern database**

### Rationale:
- **unicode-segmentation**: Provides grapheme cluster iteration (important for detecting combining characters)
- **unicode-normalization**: NFC/NFD normalization for canonical equivalence checks
- **Custom database**: Malicious patterns are domain-specific, not in Unicode standard libs
  - Zero-width: U+200B, U+200C, U+200D, U+FEFF
  - Bidi overrides: U+202A-U+202E, U+2066-U+2069
  - Homoglyphs: Requires custom mapping tables (e.g., Cyrillic 'а' vs Latin 'a')
  - Control chars: U+0000-U+001F (excluding tab, LF, CR), U+007F-U+009F

### Alternatives Considered:
- **unicode-rs/unicode-data**: Low-level, would need to build detection on top
- **ucd-parse**: Parses Unicode Character Database files, too low-level
- **Third-party homoglyph DBs**: Exist but need vetting for quality

### Implementation Notes:
- Embed malicious character definitions as const data structures
- Use grapheme cluster iteration to detect zero-width and combining abuse
- Implement homoglyph detection using confusables.txt from Unicode.org
- Unicode version: Target Unicode 15.0+ (Rust std supports latest)

### Data Sources:
- Unicode Confusables: https://www.unicode.org/Public/security/latest/confusables.txt
- Unicode Categories: Rust std::char methods (is_control, etc.)
- Trojan Source research: https://trojansource.codes/ (Boucher & Ross, 2021)

---

## 4. Git Integration for Changeset Scanning

### Decision: **git2 v0.18** (libgit2 bindings)

### Rationale:
- Most mature Git library for Rust
- Does not require `git` binary in PATH (embedded libgit2)
- Can compute diffs programmatically
- Access to repository state, commits, staging area
- Well-maintained, used by major projects (cargo, gitoxide)

### Alternatives Considered:
- **gix (gitoxide)**: Pure Rust, but less mature than git2
- **Shell out to `git diff`**: Fragile, requires git binary, parsing text output
- **No Git integration**: Scan all files, but defeats changeset use case

### Implementation Notes:
```rust
use git2::{Repository, DiffOptions};

fn get_changed_files(repo_path: &Path) -> Result<Vec<PathBuf>> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_workdir(Some(&head), None)?;
    
    // Extract file paths from diff
    // Filter for added/modified files only
}
```

### Edge Cases to Handle:
- Uninitialized Git repos (scan all files)
- Detached HEAD state
- Staged vs unstaged changes (support both modes)
- Binary files in diff (skip or detect encoding)

---

## 5. Encoding Detection

### Decision: **chardetng v0.1** (Firefox's character encoding detector)

### Rationale:
- Port of Firefox's detector (battle-tested on real web content)
- Handles UTF-8, UTF-16, Latin-1, and many legacy encodings
- Good accuracy on real-world mixed-encoding files
- Rust-native, no C dependencies
- Graceful degradation for uncertain encodings

### Alternatives Considered:
- **encoding_rs**: Mozilla's encoding library, but no auto-detection
- **charset**: Older, less accurate
- **Assume UTF-8 only**: Too restrictive, fails on legacy codebases

### Implementation Notes:
- Try UTF-8 first (fast path for 95% of modern code)
- Fall back to chardetng for non-UTF-8 files
- Convert detected encoding to UTF-8 for processing
- Report encoding in violation output if non-UTF-8
- Skip binary files (heuristic: null bytes in first 8KB)

```rust
use chardetng::EncodingDetector;

fn detect_and_decode(bytes: &[u8]) -> Result<String> {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }
    
    // Detect encoding
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    
    // Decode to UTF-8
    let (decoded, _, _) = encoding.decode(bytes);
    Ok(decoded.into_owned())
}
```

---

## 6. Color Output

### Decision: **owo-colors v4**

### Rationale:
- Zero dependencies (important for slim binary)
- Compile-time styled strings (no runtime overhead)
- Supports 256-color and truecolor
- TTY detection built-in
- NO_COLOR support built-in
- Chainable API (ergonomic)
- Smaller binary than alternatives

### Alternatives Considered:
- **colored**: Popular but has dependencies, larger binary
- **termcolor**: More complex API, designed for termcolor/WriteColor trait
- **yansi**: Good but less ergonomic API

### Implementation Notes:
```rust
use owo_colors::{OwoColorize, Stream};

// Respects TTY detection automatically
println!("{}", "Error".if_supports_color(Stream::Stdout, |text| text.red()));

// Manual control
use owo_colors::set_override;
set_override(false); // Disable colors
```

### Environment Variable Support:
- Detect `NO_COLOR` env var (https://no-color.org)
- Respect `CLICOLOR_FORCE` for always-on mode
- `--color` flag takes precedence over env vars

---

## 7. Testing Infrastructure

### Decisions:

#### Unit Testing: **Built-in Rust test framework**
- No external dependencies needed
- `#[cfg(test)]` modules in each source file
- `cargo test` standard workflow

#### Integration Testing: **assert_cmd v2 + predicates v3**
- **assert_cmd**: Test CLI binaries end-to-end
- **predicates**: Fluent assertions for output matching
- Tests in `tests/integration/` directory

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_scan_detects_zero_width() {
    Command::cargo_bin("unicleaner")
        .unwrap()
        .arg("tests/fixtures/zero_width/")
        .assert()
        .failure()
        .stdout(predicate::str::contains("U+200B"));
}
```

#### Fuzz Testing: **cargo-fuzz v0.11** (libFuzzer)
- Industry standard for Rust fuzzing
- Uses LLVM's libFuzzer (coverage-guided)
- Separate `fuzz/` directory with targets
- Run with: `cargo +nightly fuzz run fuzz_unicode`

```rust
// fuzz/fuzz_targets/fuzz_unicode.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = unicleaner::detect_malicious(s);
    }
});
```

#### Property-Based Testing: **proptest v1**
- Generate random test cases based on properties
- Good for testing Unicode range logic and config merging

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_unicode_range_contains(
        start in 0u32..0x10FFFF,
        end in 0u32..0x10FFFF,
    ) {
        let range = UnicodeRange::new(start.min(end), start.max(end));
        prop_assert!(range.contains(start.min(end)));
    }
}
```

#### Coverage: **cargo-tarpaulin v0.27**
- Rust code coverage tool
- Integrates with CI
- Outputs lcov format for coverage reports

---

## 8. Additional Dependencies

### JSON Output: **serde_json v1**
- Standard JSON library for Rust
- Integrates with serde
- Fast and reliable

### File Walking: **walkdir v2** or **ignore v0.4**
- **walkdir**: Simple recursive directory traversal
- **ignore**: Respects .gitignore patterns (better for code scanning)
- **Decision**: Use **ignore** to respect .gitignore by default

### Glob Patterns: **globset v0.4**
- Compile glob patterns for file matching in config
- From same authors as ripgrep (well-tested)
- Efficient multi-pattern matching

---

## 9. Performance Considerations

### Parallel Scanning: **rayon v1.8**
- Data parallelism for scanning multiple files
- Zero-cost abstraction over thread pools
- Simple API: `.par_iter()` instead of `.iter()`

```rust
use rayon::prelude::*;

files.par_iter()
    .map(|file| scan_file(file, &config))
    .collect()
```

### Memory Mapping Large Files: **memmap2 v0.9**
- For files >10MB, use memory mapping instead of reading into memory
- Reduces memory footprint
- OS handles paging

---

## 10. Nix Flake Dependencies

### Decision: Use **rust-overlay** + **naersk**

### Rationale:
- **rust-overlay**: Provides latest Rust toolchains in Nix
- **naersk**: Builds Rust projects in Nix efficiently (caches dependencies)
- Alternative: **crane** (newer, similar to naersk)

### Flake Structure:
```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  
  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };
      in {
        packages.default = # Build with naersk
        devShells.default = # Dev environment
        checks = # Tests, clippy, fmt
      }
    );
}
```

---

## Summary of Technology Stack

| Component | Library | Version | Rationale |
|-----------|---------|---------|-----------|
| CLI Parsing | clap | v4 | Industry standard, derive macros |
| TOML Config | toml + serde | v0.8 + v1 | De-facto standard |
| Unicode Detection | unicode-segmentation + custom | v1.10 + - | Grapheme clusters + domain logic |
| Git Integration | git2 | v0.18 | Mature libgit2 bindings |
| Encoding Detection | chardetng | v0.1 | Firefox's battle-tested detector |
| Color Output | owo-colors | v4 | Zero deps, TTY detection |
| JSON Output | serde_json | v1 | Standard |
| File Walking | ignore | v0.4 | Respects .gitignore |
| Glob Matching | globset | v0.4 | From ripgrep authors |
| Parallel Scanning | rayon | v1.8 | Easy parallelism |
| CLI Testing | assert_cmd + predicates | v2 + v3 | End-to-end CLI tests |
| Fuzzing | cargo-fuzz | v0.11 | Coverage-guided fuzzing |
| Property Testing | proptest | v1 | Generative testing |
| Coverage | cargo-tarpaulin | v0.27 | Code coverage reports |
| Nix Build | naersk + rust-overlay | latest | Efficient Rust in Nix |

---

## Security Considerations

1. **Unicode Normalization Attacks**: Detect NFD vs NFC differences that could hide malicious intent
2. **Homoglyph Databases**: Use Unicode.org's confusables.txt as authoritative source
3. **Regular Updates**: Unicode standard evolves; need process to update character databases
4. **Performance vs Security**: Balance thorough scanning with speed (parallelize where safe)

---

## References

- Unicode Security Mechanisms: https://unicode.org/reports/tr39/
- Trojan Source: Invisible Vulnerabilities (2021): https://trojansource.codes/
- Unicode Confusables: https://www.unicode.org/Public/security/latest/confusables.txt
- Rust Security Book: https://anssi-fr.github.io/rust-guide/
- Cargo Book (Testing): https://doc.rust-lang.org/cargo/guide/tests.html
