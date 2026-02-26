# Implementation Plan: Fix Review Findings

**Branch**: `004-fix-review-findings` | **Date**: 2026-02-05 (updated 2026-02-26) | **Spec**: `spec.md`  
**Input**: Feature specification from `/specs/004-fix-review-findings/spec.md`

## Summary

Fix all 17 review findings by restoring disconnected CLI behavior (config loading, encoding override, jobs handling), correcting policy-enforcement semantics, aligning JSON/CI contracts, and completing performance/build hygiene cleanup. Every finding is backed by a dedicated test and executed under constitution-required Test-First + user-approval gates.

## Technical Context

**Language/Version**: Rust stable (edition 2024, MSRV 1.85+)  
**Primary Dependencies**: `clap`, `serde`/`toml`, `rayon`, `once_cell`, `unicode-segmentation`, `globset`, `owo-colors`  
**Storage**: Filesystem scanning + TOML configuration files  
**Testing**: `cargo test`, `assert_cmd`/`predicates`, `proptest`, `cargo-fuzz`  
**Target Platform**: Linux primary; macOS/Windows secondary  
**Project Type**: Single Rust binary crate with library modules  
**Performance Goals**: Avoid per-file rebuild of malicious-pattern and preset registries; keep scan throughput stable on large repos  
**Constraints**: Pure Rust dependencies preferred, no OpenSSL/C toolchain coupling, reproducible Nix builds  
**Scale/Scope**: 17 findings mapped to 23 functional requirements across 4 implementation phases

## Constitution Check

*GATE: Must pass before implementation. Re-check after design/task updates.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Rust-First | PASS | All changes remain in Rust and existing Rust/Nix toolchain |
| II. CLI Interface | PASS | Fixes align flags, exit codes, stdout/stderr behavior, and workflow contract |
| III. Test-First (NON-NEGOTIABLE) | PASS | Execution flow is explicit: write tests -> request user approval -> verify fail -> implement -> verify pass |
| IV. Comprehensive Testing | PASS | Unit + integration coverage per finding, plus fuzz smoke gate on scanner paths |
| V. Color Output Support | PASS | No change to color behavior; `--no-color` and non-TTY compatibility preserved |
| VI. Nix Integration | PASS | Flake cleanup remains reproducible and check-oriented |
| VII. Code Quality | PASS | Requires clippy clean, fmt check, and warning-free compile gates |
| VIII. Documentation | PASS | README/DOCKER/workflow/docs updated to match actual CLI behavior |

### Mandatory Execution Gates

1. **Test-First**: For each finding, write the test first and keep implementation blocked until tests exist.
2. **User Approval Gate**: Present new/updated tests for review and obtain explicit approval before implementation steps.
3. **Red Gate**: Verify the new tests fail for the intended reason.
4. **Green Gate**: Implement minimal fix and verify tests pass.
5. **Fuzz Gate**: Run fuzz smoke targets for affected scanner/parsing paths before final merge.

## Project Structure

### Documentation (this feature)

```text
specs/004-fix-review-findings/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── main.rs
├── cli/
│   ├── args.rs
│   ├── output.rs
│   └── exit_codes.rs
├── config/
│   ├── mod.rs
│   ├── parser.rs
│   ├── presets.rs
│   ├── rules.rs
│   └── validation.rs
├── scanner/
│   ├── file_scanner.rs
│   ├── parallel.rs
│   ├── encoding.rs
│   ├── unicode_detector.rs
│   ├── walker.rs
│   └── git_diff.rs
├── unicode/
│   ├── malicious.rs
│   ├── blocks.rs
│   ├── ranges.rs
│   └── categories.rs
└── report/
    ├── mod.rs
    ├── violation.rs
    ├── json.rs
    └── formatter.rs

tests/
├── integration.rs
└── integration/
    ├── config_tests.rs
    ├── encoding_tests.rs
    ├── output_tests.rs
    ├── scan_tests.rs
    └── ...

fuzz/
└── fuzz_targets/
    ├── fuzz_parallel_scanner.rs
    └── fuzz_walker.rs
```

**Structure Decision**: Keep single-crate structure; remediate by targeted edits to existing modules and tests.

## Phase Summary

### Phase 1: CLI Correctness (Findings #1-4)

- Wire config loading through `run_scan()` -> `scan_files_parallel()` -> `scan_file_with_config()`.
- Fix `unicleaner init` output schema to match parser (`[global]`, `[[rules]]`, `[languages.<ext>]`).
- Wire `--encoding` override through scanning pipeline.
- Ensure `--jobs` controls actual rayon pool via `pool.install(...)`.

### Phase 2: Policy Enforcement (Finding #5)

- Enforce most-specific rule selection (`find_matching_rule`).
- Ensure explicit deny entries override allow ranges.
- Preserve always-deny Trojan Source bidi controls in both allow/deny defaults.

### Phase 3: Output and CI Alignment (Findings #6-9)

- Align contract schema with actual serialized `ScanResult` at `specs/001-unicode-malicious-detector/contracts/json-schema.json`.
- Update PR workflow flags/field usage to actual CLI/JSON.
- Report character-based columns and expose byte offsets.
- Classify scan errors precisely and enforce exit code 3 for errors-only scans.

### Phase 4: Performance and Hygiene (Findings #10-17)

- Cache malicious patterns and presets statically.
- Improve binary detection heuristic robustness.
- Align docs with implemented flags.
- Remove OpenSSL/pkg-config from flake inputs.
- Fix `.gitignore`/`Cargo.lock` policy and bench target declarations.
- Ensure all tests under `tests/` are valid and wired.

## Verification Gates

Run before merge:

```bash
cargo test --all-features --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
cargo bench --no-run
nix flake check
```

Run fuzz smoke gate on affected scanner surfaces:

```bash
just fuzz fuzz-parallel-scanner 30
just fuzz fuzz-walker 30
```

## Complexity Tracking

No constitution exceptions required. All work stays within existing architecture and quality gates.
