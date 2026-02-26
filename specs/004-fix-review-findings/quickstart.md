# Quickstart: Fix Review Findings

**Feature Branch**: `004-fix-review-findings`
**Date**: 2026-02-05

## Overview

This feature fixes 17 code review findings spanning CLI correctness, policy enforcement, output alignment, and build hygiene. The changes are organized into 4 phases with clear dependencies.

## Prerequisites

- Rust 1.85+ (stable, edition 2024)
- Cargo + standard toolchain (clippy, rustfmt)
- Nix (for flake fixes)
- Git (for workflow testing)

## Constitution Execution Gates

Apply these gates to each finding before implementation:

1. Write the test first.
2. Request explicit user approval for the test case(s).
3. Verify the new test fails for the intended reason.
4. Implement the minimum change required to pass.
5. Re-run tests and confirm pass.

## Development Workflow

### Phase 1: CLI Correctness (Start here)

**Finding #1 — Wire config loading**:
1. Modify `run_scan()` in `src/main.rs` to load config via `Configuration::from_file()`.
2. Add `config: &Configuration` parameter to `scan_files_parallel()`.
3. Pass config through to `scan_file_with_config()`.
4. Test: scan with config that allows Greek → verify Greek not reported.

**Finding #2 — Fix init config schema**:
1. Update `run_init()` in `src/main.rs` to generate TOML matching parser schema.
2. Test: run init → parse output → assert no error.

**Finding #3 — Wire encoding override**:
1. Extract `encoding` from `Command::Scan` destructuring in `run_scan()`.
2. Add `encoding_override` parameter through scanning pipeline.
3. Test: create UTF-16LE file → scan with `--encoding utf16-le` → verify decoding.

**Finding #4 — Fix rayon thread pool**:
1. In `scan_files_parallel()`, use `pool.install(|| { ... })`.
2. Test: verify `rayon::current_num_threads()` inside closure matches `--jobs N`.

### Phase 2: Policy Enforcement

**Finding #5 — Fix apply_config_rules()**:
1. Rewrite to use `find_matching_rule()` for single rule selection.
2. Add always-deny list for bidi controls (U+202A-202E, U+2066-2069).
3. Enforce `denied_characters` (stored internally as denied code points) overriding `allowed_ranges`.
4. Tests: denied char in allowed range → reported; bidi in allow-by-default → reported; file-specific rule overrides global.

### Phase 3: Output and CI

**Finding #6 — Update JSON schema contract**:
1. Update `specs/001-unicode-malicious-detector/contracts/json-schema.json` to match `ScanResult` serialization.
2. Test: validate CLI JSON output against schema.

**Finding #7 — Fix CI workflow**:
1. Replace `--output json` with `--format json` in `.github/workflows/pr-check.yml`.
2. Update jq field references to match actual JSON fields.

**Finding #8 — Fix column calculation**:
1. In `detect_in_string()`, track character count instead of byte offset.
2. Add `byte_offset` field to `Violation`.
3. Test: file with multi-byte chars → column matches char position.

**Finding #9 — Fix error classification and exit codes**:
1. Map `std::io::ErrorKind` to `ErrorType` variants in `scan_files_parallel()`.
2. Add exit code 3 logic to `ScanResult::exit_code()`.
3. Tests: permission error → `PermissionDenied`; errors-only → exit code 3.

### Phase 4: Performance and Hygiene

**Findings #10, #11 — Static patterns and presets**:
1. Wrap `get_malicious_patterns()` return in `Lazy<Vec<MaliciousPattern>>`.
2. Wrap `get_all_presets()` return in `Lazy<HashMap<...>>`.
3. Test: verify pointer equality across calls (same static data).

**Finding #12 — Improve binary detection**:
1. Add control-byte ratio check to `is_binary()`.
2. Test: binary file with few nulls → still detected as binary.

**Finding #13 — Update documentation**:
1. Audit README.md and docs/DOCKER.md for invalid flags.
2. Replace with current CLI flags.

**Finding #14 — Remove OpenSSL from Nix**:
1. Remove `openssl` and `pkg-config` from `flake.nix`.
2. Test: `nix build` succeeds without OpenSSL.

**Finding #15 — Fix .gitignore**:
1. Remove `Cargo.lock` line from `.gitignore`.

**Finding #16 — Fix benchmark harness**:
1. Add `[[bench]]` entries for all 6 bench files in `Cargo.toml`.
2. Test: `cargo bench` compiles all targets.

**Finding #17 — Clean up stale tests**:
1. Audit `tests/performance/`, `tests/unit/`, `tests/regression/`, `tests/contract/`.
2. Remove files referencing non-existent APIs.
3. Wire valid files into test harness via `tests/integration.rs`.
4. Test: `cargo test` compiles all test files.

## Testing Strategy

Each finding has a dedicated test. Tests follow strict constitution flow:

1. Write failing test for the finding.
2. Request user approval of the test before implementation.
3. Verify test fails for the intended reason.
4. Implement fix.
5. Verify test passes.
6. Run full suite: `cargo test --all`.
7. Run clippy: `cargo clippy -- -D warnings`.
8. Run fmt check: `cargo fmt --check`.

### Fuzz Smoke Gates (Required Before Merge)

Run scanner-surface fuzz smoke checks after functional tests are green:

```bash
just fuzz fuzz-parallel-scanner 30
just fuzz fuzz-walker 30
```

## Key Files

| Area | Primary Files |
|------|--------------|
| Config loading | `src/main.rs`, `src/scanner/parallel.rs`, `src/scanner/file_scanner.rs` |
| Init schema | `src/main.rs`, `src/config/parser.rs` |
| Encoding | `src/main.rs`, `src/cli/args.rs`, `src/scanner/encoding.rs` |
| Thread pool | `src/scanner/parallel.rs` |
| Policy | `src/scanner/file_scanner.rs`, `src/config/rules.rs` |
| JSON schema | `src/report/mod.rs`, `src/report/json.rs`, `src/report/violation.rs` |
| CI workflow | `.github/workflows/pr-check.yml` |
| Columns | `src/scanner/unicode_detector.rs`, `src/report/violation.rs` |
| Exit codes | `src/report/mod.rs`, `src/cli/exit_codes.rs` |
| Patterns | `src/unicode/malicious.rs` |
| Presets | `src/config/presets.rs` |
| Binary detect | `src/scanner/encoding.rs` |
| Docs | `README.md`, `docs/DOCKER.md` |
| Nix | `flake.nix` |
| Gitignore | `.gitignore` |
| Benchmarks | `Cargo.toml`, `benches/*.rs` |
| Stale tests | `tests/performance/`, `tests/unit/`, `tests/regression/`, `tests/contract/` |
