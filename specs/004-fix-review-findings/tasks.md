# Tasks: Fix Review Findings

**Input**: Design documents from `/specs/004-fix-review-findings/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included per constitution (Test-First is NON-NEGOTIABLE). Each finding has a dedicated test written before the fix.

**Organization**: Tasks grouped by user story (mapped from review findings). Each story is independently testable.

## Constitution Execution Gates (MANDATORY)

- For every story: write test tasks first and keep implementation blocked until tests exist.
- After writing tests and before implementation, request explicit user approval of the proposed tests.
- Verify the new tests fail for the intended reason before writing implementation code.
- Implement minimal fix, then verify tests pass and no regressions are introduced.
- Run fuzz smoke gates for affected scanner paths in final polish phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Exact file paths included in all task descriptions

---

## Phase 1: Setup

**Purpose**: No new project setup needed — all changes modify existing files. This phase handles prerequisite signature changes that unblock multiple stories.

- [X] T001 Add `byte_offset: usize` field to `Violation` struct in `src/report/violation.rs` and update all `Violation::new()` call sites to supply a default value of 0
- [X] T002 Add `encoding_override: Option<DetectedEncoding>` parameter to `scan_file_with_config()` in `src/scanner/file_scanner.rs` and update call sites
- [X] T003 Change `scan_files_parallel()` signature in `src/scanner/parallel.rs` to accept `config: &Configuration` and `encoding_override: Option<DetectedEncoding>`, update all call sites including `src/main.rs`
- [X] T004 Verify `cargo test` and `cargo clippy` still pass after signature changes

**Checkpoint**: All function signatures updated. Existing behavior preserved. No new functionality yet.

---

## Phase 2: Foundational

**Purpose**: Core config loading infrastructure that unblocks US1, US2, US5 and all config-dependent stories.

- [X] T005 Write test `test_config_loading_from_file` in `tests/integration/config_tests.rs`: given a valid config file allowing Greek block, scanning a file with Greek chars produces no Greek violations
- [X] T006 Write test `test_config_auto_discovery` in `tests/integration/config_tests.rs`: given `unicleaner.toml` in CWD, scan auto-loads it
- [X] T007 Write test `test_config_missing_file_error` in `tests/integration/config_tests.rs`: given `--config missing.toml`, scan exits with error
- [X] T008 Write test `test_config_default_when_no_file` in `tests/integration/config_tests.rs`: given no config file, scan uses defaults and succeeds
- [X] T009 Verify tests T005-T008 FAIL (config not yet wired)

**Approval Gate (Foundational)**: After T005-T009 are complete, obtain user approval before starting T010-T013.

- [X] T010 Implement config loading in `run_scan()` in `src/main.rs`: extract `config` from `--config` flag or auto-discover `unicleaner.toml`, load via `Configuration::from_file()`, fall back to `Configuration::default()`
- [X] T011 Wire loaded config through `scan_files_parallel()` → `scan_file_with_config()` in `src/scanner/parallel.rs` (replace `scan_file(file)` call with `scan_file_with_config(file, config, encoding_override)`)
- [X] T012 Verify tests T005-T008 PASS
- [X] T013 Update `examples/unicleaner.toml` if needed to stay in sync with config loading changes per AGENTS.md Config Sync Rules

**Checkpoint**: Config loading works end-to-end. `--config` flag is functional. Auto-discovery works. Error on missing file.

---

## Phase 3: User Story 1 — Config-Driven Scanning (Priority: P1) — MVP

**Goal**: Config file governs scan behavior. Users can customize Unicode policy via TOML.

**Independent Test**: Scan with config that allows Greek → no Greek violations; scan without config → Greek violations reported.

### Tests for US1

- [X] T014 [US1] Write test `test_config_changes_scan_behavior` in `tests/integration/config_tests.rs`: scan same file with and without config, verify different violation counts (SC-001)

**Approval Gate (US1)**: After T014 is written and failing, obtain user approval before starting T015-T016.

### Implementation for US1

- [X] T015 [US1] Ensure `apply_config_rules()` in `src/scanner/file_scanner.rs` correctly uses the passed `config` parameter (currently it does, but was never called with non-default config from parallel scanner)
- [X] T016 [US1] Verify test T014 PASSES

**Checkpoint**: US1 complete. Config-driven scanning verified end-to-end.

---

## Phase 4: User Story 3 — Valid Init Config (Priority: P1)

**Goal**: `unicleaner init` generates config that parses successfully with current schema.

**Independent Test**: Run init → parse output → assert no error.

### Tests for US3

- [X] T017 [US3] Write test `test_init_generates_valid_config` in `tests/integration/config_tests.rs`: run `run_init()` to tempdir, then `Configuration::from_file()` on output → assert Ok
- [X] T018 [US3] Write test `test_init_config_matches_parser_schema` in `tests/integration/config_tests.rs`: verify generated config uses `[global]`, `[[rules]]`, `[languages.<ext>]` sections

**Approval Gate (US3)**: After T017-T018 are written and failing, obtain user approval before starting T019-T020.

### Implementation for US3

- [X] T019 [US3] Rewrite `run_init()` TOML template in `src/main.rs` to match parser schema: use `[global]` for `deny_by_default`, `[[rules]]` for file rules, `[languages.<ext>]` for presets
- [X] T020 [US3] Verify tests T017-T018 PASS

**Checkpoint**: US3 complete. Init generates valid, parseable config.

---

## Phase 5: User Story 2 — Correct Policy Enforcement (Priority: P1)

**Goal**: Rule priority, denied chars, and always-deny bidi patterns enforced correctly.

**Independent Test**: Denied char in allowed range → reported. Bidi in allow-by-default → reported. File-specific rule beats global.

### Tests for US2

- [X] T021 [P] [US2] Write test `test_denied_chars_override_allowed_ranges` in `tests/integration/config_tests.rs`: config with allowed range 0x2000-0x206F and denied_characters [0x200B], file with U+200B → violation reported
- [X] T022 [P] [US2] Write test `test_always_deny_bidi_in_allow_by_default` in `tests/integration/config_tests.rs`: allow-by-default config, file with U+202E (RLO) → violation reported
- [X] T023 [P] [US2] Write test `test_file_specific_rule_overrides_global` in `tests/integration/config_tests.rs`: global rule allows all, `*.rs` rule restricts to ASCII only, scan `.rs` file with Greek → violation reported
- [X] T024 [P] [US2] Write test `test_find_matching_rule_used_not_all_rules` in `src/scanner/file_scanner.rs` (unit test): verify only most specific rule is used for policy decision
- [X] T025 [US2] Verify tests T021-T024 FAIL

**Approval Gate (US2)**: After T021-T025 are complete, obtain user approval before starting T026-T029.

### Implementation for US2

- [X] T026 [US2] Add `is_always_deny_pattern()` helper function in `src/scanner/file_scanner.rs` that returns true for bidi override controls U+202A-202E and bidi isolate controls U+2066-2069
- [X] T027 [US2] Rewrite `apply_config_rules()` in `src/scanner/file_scanner.rs`: (1) always keep always-deny patterns, (2) use `find_matching_rule()` for single rule, (3) check `denied_characters` (stored internally as denied code points) before `allowed_ranges`, (4) respect deny/allow-by-default mode
- [X] T028 [US2] Verify tests T021-T024 PASS
- [X] T029 [US2] Update `examples/unicleaner.toml` to explicitly deny bidi controls if they fall within allowed ranges, per Config Sync Rules

**Checkpoint**: US2 complete. Policy enforcement is correct and security-sound.

---

## Phase 6: User Story 4 — Working Concurrency Control (Priority: P2)

**Goal**: `--jobs N` controls actual rayon thread count.

**Independent Test**: `--jobs 2` → rayon pool uses exactly 2 threads.

### Tests for US4

- [X] T030 [US4] Write test `test_jobs_controls_thread_count` in `src/scanner/parallel.rs` (unit test): call `scan_files_parallel` with `num_threads=Some(2)`, verify `rayon::current_num_threads() == 2` inside the scanning closure

**Approval Gate (US4)**: After T030 is written and failing, obtain user approval before starting T031-T032.

### Implementation for US4

- [X] T031 [US4] Fix `scan_files_parallel()` in `src/scanner/parallel.rs`: use `pool.install(|| { files.par_iter()... })` instead of creating and dropping the pool
- [X] T032 [US4] Verify test T030 PASSES

**Checkpoint**: US4 complete. `--jobs` flag controls concurrency.

---

## Phase 7: User Story 5 — Working Encoding Override (Priority: P2)

**Goal**: `--encoding utf16-le` forces file decoding.

**Independent Test**: Create UTF-16LE file → scan with override → correct decoding.

### Tests for US5

- [X] T033 [P] [US5] Write test `test_encoding_override_utf16le` in `tests/integration/encoding_tests.rs`: create UTF-16LE file with zero-width space, scan with encoding override → violation found
- [X] T034 [P] [US5] Write test `test_encoding_override_bypasses_detection` in `src/scanner/file_scanner.rs` (unit test): pass encoding override, verify auto-detection is skipped

**Approval Gate (US5)**: After T033-T034 are written and failing, obtain user approval before starting T035-T038.

### Implementation for US5

- [X] T035 [US5] Extract `encoding` from `Command::Scan` destructuring in `run_scan()` in `src/main.rs` (currently ignored via `..` pattern)
- [X] T036 [US5] Add `EncodingOption` → `DetectedEncoding` conversion function in `src/cli/args.rs` or `src/scanner/encoding.rs`
- [X] T037 [US5] Implement encoding override logic in `scan_file_with_config()` in `src/scanner/file_scanner.rs`: when `encoding_override` is Some, skip `detect_decode_with_encoding()` and decode directly with the specified encoding
- [X] T038 [US5] Verify tests T033-T034 PASS

**Checkpoint**: US5 complete. `--encoding` flag works.

---

## Phase 8: User Story 8 — Correct Column Reporting (Priority: P2)

**Goal**: Column numbers are character-based, not byte offsets. `byte_offset` available separately.

**Independent Test**: File with multi-byte chars → column matches char position.

### Tests for US8

- [X] T039 [US8] Write test `test_column_is_char_based_not_byte` in `src/scanner/unicode_detector.rs` (unit test): string with 3-byte UTF-8 char (e.g., emoji) before a zero-width space → column should be char position, not byte position
- [X] T040 [US8] Write test `test_byte_offset_field_populated` in `src/scanner/unicode_detector.rs` (unit test): verify `byte_offset` field is set correctly on violations

**Approval Gate (US8)**: After T039-T040 are written and failing, obtain user approval before starting T041-T043.

### Implementation for US8

- [X] T041 [US8] Modify `detect_in_string()` in `src/scanner/unicode_detector.rs`: track `char_column` counter (increments by `grapheme.chars().count()` per grapheme) instead of using byte index from `grapheme_indices`; set `byte_offset` from the grapheme byte index
- [X] T042 [US8] Update `Violation::new()` calls in `detect_in_string()` to pass `byte_offset` (from grapheme_indices) and `char_column + 1` as column
- [X] T043 [US8] Verify tests T039-T040 PASS

**Checkpoint**: US8 complete. Column numbers match editor expectations.

---

## Phase 9: User Story 9 — Error Classification and Exit Codes (Priority: P2)

**Goal**: Errors classified by type. Exit code 3 for errors-only.

**Independent Test**: Permission error → `PermissionDenied` type. Errors without violations → exit code 3.

### Tests for US9

- [X] T044 [P] [US9] Write test `test_permission_error_classified` in `src/scanner/parallel.rs` (unit test): scan unreadable file → error type is `PermissionDenied`
- [X] T045 [P] [US9] Write test `test_exit_code_3_errors_only` in `src/report/mod.rs` (unit test): ScanResult with errors but no violations → `exit_code() == 3`
- [X] T046 [P] [US9] Write test `test_exit_code_1_violations_with_errors` in `src/report/mod.rs` (unit test): ScanResult with both violations and errors → `exit_code() == 1`

**Approval Gate (US9)**: After T044-T046 are written and failing, obtain user approval before starting T047-T049.

### Implementation for US9

- [X] T047 [US9] Implement error classification in `scan_files_parallel()` in `src/scanner/parallel.rs`: map `std::io::ErrorKind::PermissionDenied` → `ErrorType::PermissionDenied`, detect encoding errors → `ErrorType::EncodingError`, fallback → `ErrorType::IoError`
- [X] T048 [US9] Update `exit_code()` in `src/report/mod.rs`: return 3 when `!self.errors.is_empty() && self.violations.is_empty()`; return 1 when `!self.violations.is_empty()` (regardless of errors)
- [X] T049 [US9] Verify tests T044-T046 PASS

**Checkpoint**: US9 complete. CI can distinguish error conditions via exit codes.

---

## Phase 10: User Story 6 — JSON Schema Alignment (Priority: P2)

**Goal**: JSON output matches documented schema.

**Independent Test**: Validate CLI JSON output against schema.

### Tests for US6

- [X] T050 [US6] Write test `test_json_output_matches_schema` in `tests/integration/output_tests.rs`: run scan with `--format json`, parse output, verify all expected fields exist (`violations`, `files_scanned`, `files_clean`, `files_with_violations`, `errors`, `duration`, `config_used`) and violation fields (`file_path`, `line`, `column`, `byte_offset`, `code_point`, `pattern_name`, `category`, `severity`, `message`, `encoding`)

**Approval Gate (US6)**: After T050 is written and failing, obtain user approval before starting T051-T052.

### Implementation for US6

- [X] T051 [US6] Update `specs/001-unicode-malicious-detector/contracts/json-schema.json` to match actual `ScanResult` serialization including new `byte_offset` field
- [X] T052 [US6] Verify test T050 PASSES

**Checkpoint**: US6 complete. JSON schema is documented and verified.

---

## Phase 11: User Story 7 — Working CI Workflow (Priority: P2)

**Goal**: GitHub Actions PR check workflow uses correct flags and field references.

**Independent Test**: Workflow YAML uses `--format json` and references existing JSON fields.

### Tests for US7

- [X] T053 [US7] Write test `test_pr_check_workflow_uses_valid_flags` in `tests/integration/output_tests.rs`: parse `.github/workflows/pr-check.yml`, assert no occurrence of `--output json` (should be `--format json`)

**Approval Gate (US7)**: After T053 is written and failing, obtain user approval before starting T054-T057.

### Implementation for US7

- [X] T054 [US7] Update `.github/workflows/pr-check.yml`: replace `--output json` with `--format json`, replace `--output` with `-f` or `--format` throughout
- [X] T055 [US7] Update jq field references in `.github/workflows/pr-check.yml`: `.description` → `.message`, `.pattern` → `.pattern_name`, and any other mismatched fields
- [X] T056 [US7] Add explicit `scan` subcommand to workflow invocations in `.github/workflows/pr-check.yml` for clarity
- [X] T057 [US7] Verify test T053 PASSES

**Checkpoint**: US7 complete. CI workflow uses correct CLI invocations.

---

## Phase 12: User Story 10 — Static Pattern and Preset Caching (Priority: P3)

**Goal**: Patterns and presets initialized once, not per-file.

**Independent Test**: Verify static initialization via pointer equality or call-count.

### Tests for US10

- [X] T058 [P] [US10] Write test `test_malicious_patterns_static` in `src/unicode/malicious.rs` (unit test): call `get_malicious_patterns()` twice, verify same pointer (or same Arc/reference)
- [X] T059 [P] [US10] Write test `test_presets_static` in `src/config/presets.rs` (unit test): call `get_all_presets()` twice, verify same pointer

**Approval Gate (US10)**: After T058-T059 are written and failing, obtain user approval before starting T060-T063.

### Implementation for US10

- [X] T060 [US10] Wrap `get_malicious_patterns()` return value in `once_cell::sync::Lazy<Vec<MaliciousPattern>>` in `src/unicode/malicious.rs`; update function to return `&'static Vec<MaliciousPattern>` or equivalent
- [X] T061 [US10] Wrap `get_all_presets()` return value in `once_cell::sync::Lazy<HashMap<String, LanguagePreset>>` in `src/config/presets.rs`; update `get_preset()` to use the static
- [X] T062 [US10] Update `detect_in_string()` in `src/scanner/unicode_detector.rs` to use the static patterns reference
- [X] T063 [US10] Verify tests T058-T059 PASS

**Checkpoint**: US10 complete. No per-file allocation of patterns/presets.

---

## Phase 13: User Story 11 — Robust Binary Detection (Priority: P3)

**Goal**: Binary files detected by multiple heuristics, not just consecutive nulls.

**Independent Test**: Binary file with few nulls → still detected as binary.

### Tests for US11

- [X] T064 [P] [US11] Write test `test_binary_detection_control_byte_ratio` in `src/scanner/encoding.rs` (unit test): create byte array with 40% control bytes but zero consecutive nulls → `is_binary()` returns true
- [X] T065 [P] [US11] Write test `test_text_file_not_falsely_binary` in `src/scanner/encoding.rs` (unit test): normal UTF-8 text with occasional control chars (TAB, CR, LF) → `is_binary()` returns false

**Approval Gate (US11)**: After T064-T065 are written and failing, obtain user approval before starting T066-T067.

### Implementation for US11

- [X] T066 [US11] Add control-byte ratio heuristic to `is_binary()` in `src/scanner/encoding.rs`: count bytes in ranges 0x00-0x08 and 0x0E-0x1F (excluding TAB 0x09, LF 0x0A, CR 0x0D) in first 8KB; if ratio > 30%, classify as binary
- [X] T067 [US11] Verify tests T064-T065 PASS

**Checkpoint**: US11 complete. Fewer binary files cause scan errors.

---

## Phase 14: User Story 12 — Accurate Documentation (Priority: P3)

**Goal**: README and docs reference only existing CLI flags.

**Independent Test**: All documented flags accepted by CLI parser.

### Tests for US12

- [X] T068 [US12] Write test `test_readme_flags_exist_in_cli` in `tests/integration/output_tests.rs`: extract CLI flag patterns from `README.md`, verify each exists in clap args definition

**Approval Gate (US12)**: After T068 is written and failing, obtain user approval before starting T069-T071.

### Implementation for US12

- [X] T069 [P] [US12] Audit and update `README.md`: remove references to `--output <FILE>`, `--json`, `--threads`, `--max-file-size`; replace with current flags (`--format`, `--jobs`, `--encoding`, etc.)
- [X] T070 [P] [US12] Audit and update `docs/DOCKER.md`: replace invalid flag references with current CLI flags
- [X] T071 [US12] Verify test T068 PASSES

**Checkpoint**: US12 complete. Documentation matches reality.

---

## Phase 15: User Story 13 — Clean Build System (Priority: P3)

**Goal**: Remove OpenSSL from Nix, fix .gitignore, fix bench harness.

**Independent Test**: Nix build succeeds. `cargo bench` compiles all targets.

### Tests for US13

- [X] T072 [P] [US13] Write test `test_gitignore_does_not_list_cargo_lock` in `tests/integration/config_tests.rs`: read `.gitignore`, assert `Cargo.lock` line is not present
- [X] T073 [P] [US13] Write test `test_all_bench_targets_configured` in `tests/integration/config_tests.rs`: read `Cargo.toml`, list `benches/*.rs` files, verify each has a `[[bench]]` entry with `harness = false`

**Approval Gate (US13)**: After T072-T073 are written and failing, obtain user approval before starting T074-T078.

### Implementation for US13

- [X] T074 [P] [US13] Remove `openssl` and `pkg-config` from `buildInputs` and `nativeBuildInputs` in `flake.nix`
- [X] T075 [P] [US13] Remove `Cargo.lock` line from `.gitignore`
- [X] T076 [P] [US13] Add `[[bench]]` entries to `Cargo.toml` for `unicode_heavy`, `small_repo`, `medium_repo`, `large_repo`, `memory_usage` (all with `harness = false`)
- [X] T077 [US13] Verify tests T072-T073 PASS
- [X] T078 [US13] Verify `cargo bench --no-run` compiles all 6 benchmark targets

**Checkpoint**: US13 complete. Build system is clean.

---

## Phase 16: User Story 14 — All Tests Compile and Run (Priority: P3)

**Goal**: No orphaned/stale test files. All test code under `tests/` compiles.

**Independent Test**: `cargo test` compiles all test files without errors.

### Tests for US14

- [X] T079 [US14] Write test `test_all_test_files_compiled` in `tests/integration/config_tests.rs`: list all `.rs` files under `tests/` (excluding fixtures), verify each is either included by `tests/integration.rs` or is a top-level test target

**Approval Gate (US14)**: After T079 is written and failing, obtain user approval before starting T080-T085.

### Implementation for US14

- [X] T080 [US14] Audit `tests/performance/` directory: remove files referencing non-existent APIs, wire valid files into `tests/integration.rs` or remove
- [X] T081 [US14] Audit `tests/unit/` directory: remove files referencing non-existent APIs (e.g., `detect_malicious_unicode`), wire valid files into test harness or remove
- [X] T082 [US14] Audit `tests/regression/` directory: remove stale files or wire into test harness
- [X] T083 [US14] Audit `tests/contract/` directory: wire `exit_codes.rs` into test harness or remove if redundant
- [X] T084 [US14] Verify `cargo test` compiles and runs all test files without orphans
- [X] T085 [US14] Verify test T079 PASSES

**Checkpoint**: US14 complete. All test code compiles and runs.

---

## Phase 17: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all stories.

- [X] T086 Run full `cargo test --all` — all tests pass
- [X] T087 Run `cargo clippy -- -D warnings` — no warnings
- [X] T088 Run `cargo fmt --check` — formatting clean
- [X] T089 Verify `cargo bench --no-run` — all benchmarks compile
- [X] T090 Verify each of the 17 review findings has at least one dedicated test case (SC-017)
- [X] T091 Update `examples/unicleaner.toml` if any config schema changes were made, per Config Sync Rules
- [X] T092 Run fuzz smoke target `fuzz_parallel_scanner` and confirm no crashes/timeouts for scanner pipeline
- [X] T093 Run fuzz smoke target `fuzz_walker` and confirm no crashes/timeouts for traversal/path handling

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1 Config Scanning)**: Depends on Phase 2
- **Phase 4 (US3 Init Config)**: Depends on Phase 2 (independent of US1)
- **Phase 5 (US2 Policy Enforcement)**: Depends on Phase 2 (benefits from US1 being done)
- **Phase 6 (US4 Jobs)**: Depends on Phase 1 only (T003 signature change)
- **Phase 7 (US5 Encoding)**: Depends on Phase 1 (T002, T003 signature changes)
- **Phase 8 (US8 Columns)**: Depends on Phase 1 (T001 byte_offset field)
- **Phase 9 (US9 Error/Exit)**: Depends on Phase 1 only
- **Phase 10 (US6 JSON Schema)**: Depends on Phase 8 (byte_offset field in output)
- **Phase 11 (US7 CI Workflow)**: Depends on Phase 10 (JSON schema known)
- **Phase 12 (US10 Static Cache)**: No dependencies (independent optimization)
- **Phase 13 (US11 Binary Detection)**: No dependencies
- **Phase 14 (US12 Docs)**: Best done after all CLI changes (Phases 3-9)
- **Phase 15 (US13 Build System)**: No dependencies
- **Phase 16 (US14 Stale Tests)**: No dependencies
- **Phase 17 (Polish)**: Depends on all previous phases

### User Story Dependencies

```
Phase 1 (Setup)
  └──→ Phase 2 (Foundational: config loading)
         ├──→ Phase 3 (US1: config scanning) ──→ MVP ✓
         ├──→ Phase 4 (US3: init config) [parallel with US1]
         └──→ Phase 5 (US2: policy enforcement)
  ├──→ Phase 6 (US4: jobs flag) [parallel after Phase 1]
  ├──→ Phase 7 (US5: encoding) [parallel after Phase 1]
  ├──→ Phase 8 (US8: columns) [parallel after Phase 1]
  │      └──→ Phase 10 (US6: JSON schema)
  │             └──→ Phase 11 (US7: CI workflow)
  └──→ Phase 9 (US9: error/exit codes) [parallel after Phase 1]

Independent (any time):
  Phase 12 (US10: static cache)
  Phase 13 (US11: binary detection)
  Phase 14 (US12: docs) [best after CLI phases]
  Phase 15 (US13: build system)
  Phase 16 (US14: stale tests)
```

### Parallel Opportunities

**After Phase 1 completes**, these can run in parallel:
- Phase 6 (US4: jobs flag)
- Phase 7 (US5: encoding override)
- Phase 8 (US8: column fix)
- Phase 9 (US9: error classification)
- Phase 12 (US10: static cache)
- Phase 13 (US11: binary detection)
- Phase 15 (US13: build system)
- Phase 16 (US14: stale tests)

**After Phase 2 completes**, additionally:
- Phase 3 (US1), Phase 4 (US3), Phase 5 (US2) can start

---

## Parallel Example: After Phase 1

```bash
# These can all run simultaneously on different files:
Task: "T030 [US4] Write test for jobs thread count" (src/scanner/parallel.rs)
Task: "T033 [US5] Write test for encoding override" (tests/integration/encoding_tests.rs)
Task: "T039 [US8] Write test for char-based columns" (src/scanner/unicode_detector.rs)
Task: "T044 [US9] Write test for error classification" (src/scanner/parallel.rs)
Task: "T058 [US10] Write test for static patterns" (src/unicode/malicious.rs)
Task: "T064 [US11] Write test for binary detection" (src/scanner/encoding.rs)
Task: "T075 [US13] Remove Cargo.lock from .gitignore" (.gitignore)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (signature changes)
2. Complete Phase 2: Foundational (config loading)
3. Complete Phase 3: US1 (config-driven scanning)
4. **STOP and VALIDATE**: `cargo test` passes, config actually changes scan behavior
5. This is the MVP — the most impactful fix (Finding #1)

### Incremental Delivery

1. Phase 1 + 2 → Foundation ready
2. Phase 3 (US1) → Config works → **MVP**
3. Phase 4 (US3) → Init generates valid config
4. Phase 5 (US2) → Policy enforcement correct → **Security milestone**
5. Phases 6-9 (US4, US5, US8, US9) → CLI flags work, output correct → **CI-ready milestone**
6. Phases 10-11 (US6, US7) → JSON schema + CI workflow aligned
7. Phases 12-16 (US10-US14) → Performance + hygiene → **Complete**
8. Phase 17 → Final validation

---

## Notes

- Constitution requires strict TDD: write test → get user approval → verify fails → implement → verify passes
- [P] tasks can run in parallel (different files, no dependencies)
- [Story] label maps task to user story for traceability
- Each finding from review.md has at least one test task
- `examples/unicleaner.toml` must stay in sync with config changes (AGENTS.md rule)
- Fuzz smoke validation is mandatory before merge for affected scanner surfaces
- Commit after each phase checkpoint for clean git history
