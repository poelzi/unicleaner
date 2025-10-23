# Implementation Plan: Unicode Malicious Character Detector

**Branch**: `001-unicode-malicious-detector` | **Date**: 2025-10-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-unicode-malicious-detector/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a CLI security tool that scans source code repositories to detect malicious Unicode characters (zero-width, bidirectional overrides, homoglyphs) that could hide backdoors or exploits. The tool uses deny-by-default security with TOML configuration for language-specific character allowlists, supports scanning full repositories or Git changesets, integrates with CI/CD pipelines (GitHub Actions, GitLab CI), and provides both human-readable colored output and machine-parseable JSON reports.

## Technical Context

**Language/Version**: Rust (stable channel, currently 1.75+)  
**Primary Dependencies**: clap v4 (CLI), toml v0.8 + serde v1 (config), unicode-segmentation v1.10 (Unicode), git2 v0.18 (Git), chardetng v0.1 (encoding), owo-colors v4 (output)  
**Storage**: Configuration files (TOML), Unicode character database (embedded), no persistent database required  
**Testing**: cargo test (unit), assert_cmd v2 + predicates v3 (integration), cargo-fuzz v0.11 (fuzzing), proptest v1 (property-based)  
**Target Platform**: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64) - cross-platform CLI  
**Project Type**: Single project (standalone CLI tool)  
**Performance Goals**: Scan 10,000 files in <30 seconds, process individual files at >1MB/sec, minimal memory footprint (<500MB for large repos)  
**Constraints**: Must work offline (no network calls), must handle invalid UTF-8 gracefully, must integrate with CI environments (exit codes, no TTY)  
**Scale/Scope**: Support repositories up to 100,000 files, handle files up to 100MB each, support 50+ Unicode language presets

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Based on the Unicleaner Constitution v1.0.0:

### ✅ I. Rust-First
- [x] Project uses Rust stable channel
- [x] Will follow Rust API guidelines
- [x] Will use Result/Option for error handling
- [x] Will use cargo for dependency management
- [x] No warnings policy will be enforced

### ✅ II. CLI Interface  
- [x] Tool exposes all functionality via CLI
- [x] Accepts text input (args, flags, file paths)
- [x] Outputs to stdout (results) and stderr (errors/warnings)
- [x] Provides --help and --version
- [x] Returns proper exit codes (0=success, 1=violations, 2=errors)
- [x] Supports pipes and redirection

### ✅ III. Test-First (NON-NEGOTIABLE)
- [x] TDD will be followed: write tests → approve → verify fail → implement
- [x] All features will have tests written first
- [x] Red-Green-Refactor cycle will be enforced

### ✅ IV. Comprehensive Testing Strategy
- [x] Unit tests planned (individual Unicode detection functions, config parsing)
- [x] Integration tests planned (CLI end-to-end, file scanning workflows)
- [x] Fuzz testing planned (malformed Unicode input, corrupt TOML configs)
- [x] Property-based testing applicable (Unicode range validation, config merging)

### ✅ V. Color Output Support
- [x] Auto-detect TTY vs non-TTY
- [x] Support --color=auto|always|never flag
- [x] Respect NO_COLOR environment variable
- [x] Will use owo-colors or similar library
- [x] All output readable without colors

### ✅ VI. Nix Integration
- [x] Will provide flake.nix at repo root
- [x] Package: CLI binary as default output
- [x] Overlays: Provided for integration
- [x] Checks: cargo test, clippy, rustfmt, cargo-fuzz
- [x] DevShell: Rust toolchain + cargo-fuzz + cargo-tarpaulin + clippy + rustfmt
- [x] Multi-platform: x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin

### ✅ VII. Code Quality
- [x] cargo clippy will be enforced (no warnings)
- [x] cargo fmt will be enforced
- [x] RUSTFLAGS="-D warnings" in CI
- [x] Meaningful names, clear documentation

### ✅ VIII. Documentation
- [x] README will include: purpose, installation, usage, dev setup
- [x] Public API will have rustdoc comments
- [x] Complex algorithms will be documented inline
- [x] CLI --help will be comprehensive

**GATE STATUS**: ✅ **PASSED** - All constitutional requirements satisfied. No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
unicleaner/
├── Cargo.toml                    # Rust project manifest
├── Cargo.lock                    # Dependency lock file
├── flake.nix                     # Nix flake configuration
├── flake.lock                    # Nix dependency lock
├── README.md                     # Project documentation
├── LICENSE                       # License file
├── .gitignore                    # Git ignore patterns
│
├── src/
│   ├── main.rs                   # CLI entry point
│   ├── lib.rs                    # Library root (for reusable logic)
│   │
│   ├── cli/
│   │   ├── mod.rs                # CLI module root
│   │   ├── args.rs               # Argument parsing (clap)
│   │   ├── output.rs             # Output formatting (colored/JSON)
│   │   └── exit_codes.rs         # Exit code constants
│   │
│   ├── scanner/
│   │   ├── mod.rs                # Scanner module root
│   │   ├── file_scanner.rs       # File scanning logic
│   │   ├── unicode_detector.rs   # Unicode character detection
│   │   ├── encoding.rs           # Encoding detection/handling
│   │   └── git_diff.rs           # Git changeset integration
│   │
│   ├── config/
│   │   ├── mod.rs                # Config module root
│   │   ├── parser.rs             # TOML config parsing
│   │   ├── rules.rs              # Character allowlist rules
│   │   ├── presets.rs            # Language presets (Greek, Cyrillic, etc.)
│   │   └── validation.rs         # Config validation
│   │
│   ├── unicode/
│   │   ├── mod.rs                # Unicode module root
│   │   ├── database.rs           # Unicode character database
│   │   ├── ranges.rs             # Unicode range definitions
│   │   ├── categories.rs         # Character categorization
│   │   └── malicious.rs          # Malicious pattern definitions
│   │
│   └── report/
│       ├── mod.rs                # Report module root
│       ├── violation.rs          # Violation data structures
│       ├── formatter.rs          # Human-readable formatting
│       └── json.rs               # JSON output formatting
│
├── tests/
│   ├── integration/
│   │   ├── cli_tests.rs          # End-to-end CLI tests
│   │   ├── scan_tests.rs         # Full scan workflow tests
│   │   ├── config_tests.rs       # Configuration integration tests
│   │   └── fixtures/             # Test files with malicious Unicode
│   │       ├── clean/            # Files with no issues
│   │       ├── zero_width/       # Files with zero-width chars
│   │       ├── bidi/             # Files with bidi overrides
│   │       └── homoglyphs/       # Files with homoglyph attacks
│   │
│   └── contract/
│       ├── exit_codes.rs         # Exit code contract tests
│       └── json_schema.rs        # JSON output schema validation
│
├── fuzz/
│   ├── Cargo.toml                # Fuzzing project manifest
│   └── fuzz_targets/
│       ├── fuzz_unicode.rs       # Fuzz Unicode detection
│       ├── fuzz_config.rs        # Fuzz TOML parsing
│       └── fuzz_file_scan.rs     # Fuzz file scanning
│
├── examples/
│   ├── github-workflow.yml       # GitHub Actions example
│   ├── gitlab-ci.yml             # GitLab CI example
│   └── unicleaner.toml           # Example configuration file
│
└── benches/
    └── scan_performance.rs       # Performance benchmarks (criterion)
```

**Structure Decision**: Single project structure selected. This is a standalone CLI tool with no web frontend, mobile app, or separate backend service. All functionality is contained in a single Rust binary with a library component (lib.rs) for testability and potential future reuse.

## Complexity Tracking

> **No violations - all constitution requirements satisfied**

N/A - No complexity violations to justify.

## Post-Design Constitution Re-Check

*Re-evaluated after Phase 1 design completion*

### ✅ All Principles Still Satisfied

1. **Rust-First**: Technology stack confirmed - all Rust crates, idiomatic patterns
2. **CLI Interface**: Full CLI spec defined in contracts/cli-interface.yaml
3. **Test-First**: Testing infrastructure fully specified (unit, integration, fuzz, property)
4. **Comprehensive Testing**: Four testing layers confirmed with specific tools
5. **Color Output**: owo-colors selected, NO_COLOR support documented
6. **Nix Integration**: Flake structure defined with naersk + rust-overlay
7. **Code Quality**: Clippy, rustfmt enforcement confirmed
8. **Documentation**: Quickstart guide created, CLI contract documented

### Design Decisions Aligned with Constitution

- **Dependencies**: All are well-maintained Rust crates (no C bindings except git2/libgit2 which is standard)
- **Testing Tools**: Industry-standard Rust testing ecosystem selected
- **Project Structure**: Clean module separation supports testability
- **Error Handling**: Will use Result/Option throughout as required
- **Performance**: Rayon for parallelism aligns with performance requirements

**GATE STATUS**: ✅ **PASSED** - Design phase maintains full constitution compliance
