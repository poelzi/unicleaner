# Tasks: Unicode Malicious Character Detector

**Input**: Design documents from `/specs/001-unicode-malicious-detector/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Required per Constitution Principle III (Test-First is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths shown assume single project structure per plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create Rust project with cargo new unicleaner
- [x] T002 Set up project structure per plan.md (create all directories)
- [x] T003 [P] Create Cargo.toml with dependencies from research.md
- [x] T004 [P] Create flake.nix with rust-overlay and naersk configuration
- [x] T005 [P] Create .gitignore for Rust project
- [x] T006 [P] Create README.md with project overview
- [x] T007 [P] Set up rustfmt.toml and clippy.toml configuration files
- [x] T008 [P] Create LICENSE file
- [ ] T009 Initialize git repository and create initial commit

**Checkpoint**: Project structure ready for development

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T010 Create src/main.rs with basic CLI entry point structure
- [ ] T011 Create src/lib.rs for library root
- [ ] T012 [P] Create module structure in src/cli/mod.rs
- [ ] T013 [P] Create module structure in src/scanner/mod.rs
- [ ] T014 [P] Create module structure in src/config/mod.rs
- [ ] T015 [P] Create module structure in src/unicode/mod.rs
- [ ] T016 [P] Create module structure in src/report/mod.rs
- [ ] T017 Create src/cli/exit_codes.rs with exit code constants (0, 1, 2, 3)
- [ ] T018 Create error handling types using thiserror in src/lib.rs
- [ ] T019 Set up test fixture directories in tests/integration/fixtures/
- [ ] T020 [P] Create test fixture files with malicious Unicode in tests/integration/fixtures/zero_width/
- [ ] T021 [P] Create test fixture files with bidi overrides in tests/integration/fixtures/bidi/
- [ ] T022 [P] Create test fixture files with homoglyphs in tests/integration/fixtures/homoglyphs/
- [ ] T023 [P] Create clean test files in tests/integration/fixtures/clean/

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Scan Repository for Malicious Unicode (Priority: P1) 🎯 MVP

**Goal**: Detect malicious Unicode characters (zero-width, bidi, homoglyphs) in source code files

**Independent Test**: Can be fully tested by running scanner on test fixtures and verifying all malicious characters are detected

### Tests for User Story 1 ⚠️ WRITE TESTS FIRST

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T024 [P] [US1] Write unit test for UnicodeRange contains() method in src/unicode/ranges.rs
- [ ] T025 [P] [US1] Write unit test for detecting zero-width characters in src/unicode/malicious.rs
- [ ] T026 [P] [US1] Write unit test for detecting bidi override characters in src/unicode/malicious.rs
- [ ] T027 [P] [US1] Write unit test for detecting homoglyphs in src/unicode/malicious.rs
- [ ] T028 [P] [US1] Write unit test for file scanning logic in src/scanner/file_scanner.rs
- [ ] T029 [P] [US1] Write unit test for encoding detection in src/scanner/encoding.rs
- [ ] T030 [P] [US1] Write integration test for CLI scanning command in tests/integration/cli_tests.rs
- [ ] T031 [P] [US1] Write integration test for scanning zero-width fixtures in tests/integration/scan_tests.rs
- [ ] T032 [P] [US1] Write integration test for scanning bidi fixtures in tests/integration/scan_tests.rs
- [ ] T033 [P] [US1] Write integration test for scanning homoglyph fixtures in tests/integration/scan_tests.rs
- [ ] T034 [P] [US1] Write integration test for clean file scanning in tests/integration/scan_tests.rs
- [ ] T035 [P] [US1] Write contract test for exit codes in tests/contract/exit_codes.rs

### Implementation for User Story 1

- [ ] T036 [P] [US1] Implement UnicodeRange struct and methods in src/unicode/ranges.rs
- [ ] T037 [P] [US1] Implement malicious Unicode patterns database in src/unicode/malicious.rs
- [ ] T038 [P] [US1] Implement Unicode character categorization in src/unicode/categories.rs
- [ ] T039 [US1] Implement Unicode character database in src/unicode/database.rs
- [ ] T040 [US1] Implement encoding detection using chardetng in src/scanner/encoding.rs
- [ ] T041 [US1] Implement file scanning core logic in src/scanner/file_scanner.rs
- [ ] T042 [US1] Implement Unicode detection logic in src/scanner/unicode_detector.rs
- [ ] T043 [US1] Implement Violation struct in src/report/violation.rs
- [ ] T044 [US1] Implement ScanResult struct in src/report/mod.rs
- [ ] T045 [US1] Implement basic CLI argument parsing with clap in src/cli/args.rs
- [ ] T046 [US1] Implement directory walking with ignore crate in src/scanner/mod.rs
- [ ] T047 [US1] Integrate scanner with CLI in src/main.rs
- [ ] T048 [US1] Implement basic human-readable output in src/cli/output.rs
- [ ] T049 [US1] Add parallel file scanning with rayon in src/scanner/mod.rs
- [ ] T050 [US1] Verify all US1 tests pass

**Checkpoint**: At this point, User Story 1 (core scanning) is fully functional and independently testable

---

## Phase 4: User Story 2 - Configure Language-Specific Allowed Characters (Priority: P2)

**Goal**: Support TOML configuration files to allow legitimate Unicode for multilingual codebases

**Independent Test**: Can be tested by loading TOML configs and verifying allowed characters pass while malicious ones are caught

### Tests for User Story 2 ⚠️ WRITE TESTS FIRST

- [ ] T051 [P] [US2] Write unit test for TOML config parsing in src/config/parser.rs
- [ ] T052 [P] [US2] Write unit test for LanguagePreset loading in src/config/presets.rs
- [ ] T053 [P] [US2] Write unit test for FileRule matching in src/config/rules.rs
- [ ] T054 [P] [US2] Write unit test for config validation in src/config/validation.rs
- [ ] T055 [P] [US2] Write unit test for config merging logic in src/config/mod.rs
- [ ] T056 [P] [US2] Write integration test for loading config files in tests/integration/config_tests.rs
- [ ] T057 [P] [US2] Write integration test for language preset application in tests/integration/config_tests.rs
- [ ] T058 [P] [US2] Write integration test for file-specific rules in tests/integration/config_tests.rs
- [ ] T059 [P] [US2] Write integration test for deny-by-default behavior in tests/integration/config_tests.rs

### Implementation for User Story 2

- [ ] T060 [P] [US2] Implement LanguagePreset struct and built-in presets in src/config/presets.rs
- [ ] T061 [P] [US2] Implement FileRule struct and pattern matching in src/config/rules.rs
- [ ] T062 [P] [US2] Implement Configuration struct in src/config/mod.rs
- [ ] T063 [US2] Implement TOML parsing with serde in src/config/parser.rs
- [ ] T064 [US2] Implement config validation logic in src/config/validation.rs
- [ ] T065 [US2] Add --config flag to CLI arguments in src/cli/args.rs
- [ ] T066 [US2] Integrate config loading into scanner in src/scanner/file_scanner.rs
- [ ] T067 [US2] Implement rule application during scanning in src/scanner/unicode_detector.rs
- [ ] T068 [US2] Create example configuration file in examples/unicleaner.toml
- [ ] T069 [US2] Implement 'init' command to generate default config in src/cli/mod.rs
- [ ] T070 [US2] Implement 'list-presets' command in src/cli/mod.rs
- [ ] T071 [US2] Verify all US2 tests pass

**Checkpoint**: At this point, User Stories 1 AND 2 are functional - scanning with configuration support

---

## Phase 5: User Story 3 - Lint Changesets in CI/CD Pipeline (Priority: P3)

**Goal**: Support Git diff mode to scan only changed files for CI/CD integration

**Independent Test**: Can be tested by creating Git changes and verifying only modified files are scanned

### Tests for User Story 3 ⚠️ WRITE TESTS FIRST

- [ ] T072 [P] [US3] Write unit test for Git diff detection in src/scanner/git_diff.rs
- [ ] T073 [P] [US3] Write unit test for changeset file filtering in src/scanner/git_diff.rs
- [ ] T074 [P] [US3] Write integration test for --diff flag in tests/integration/cli_tests.rs
- [ ] T075 [P] [US3] Write integration test for Git repository scanning in tests/integration/scan_tests.rs
- [ ] T076 [P] [US3] Write integration test for CI exit codes in tests/integration/cli_tests.rs

### Implementation for User Story 3

- [ ] T077 [US3] Implement Git repository detection with git2 in src/scanner/git_diff.rs
- [ ] T078 [US3] Implement diff calculation for changed files in src/scanner/git_diff.rs
- [ ] T079 [US3] Add --diff flag to CLI arguments in src/cli/args.rs
- [ ] T080 [US3] Integrate diff mode into scanner in src/scanner/mod.rs
- [ ] T081 [US3] Create GitHub Actions example in examples/github-workflow.yml
- [ ] T082 [US3] Create GitLab CI example in examples/gitlab-ci.yml
- [ ] T083 [US3] Handle detached HEAD and staging area in src/scanner/git_diff.rs
- [ ] T084 [US3] Verify all US3 tests pass

**Checkpoint**: User Stories 1, 2, and 3 complete - full CI/CD integration ready

---

## Phase 6: User Story 4 - Generate Human and Machine-Readable Reports (Priority: P4)

**Goal**: Provide colored terminal output and JSON output for different use cases

**Independent Test**: Can be tested by scanning files and verifying both output formats are correct

### Tests for User Story 4 ⚠️ WRITE TESTS FIRST

- [ ] T085 [P] [US4] Write unit test for colored output formatting in src/report/formatter.rs
- [ ] T086 [P] [US4] Write unit test for JSON serialization in src/report/json.rs
- [ ] T087 [P] [US4] Write unit test for TTY detection in src/cli/output.rs
- [ ] T088 [P] [US4] Write unit test for NO_COLOR environment variable in src/cli/output.rs
- [ ] T089 [P] [US4] Write integration test for --json flag in tests/integration/cli_tests.rs
- [ ] T090 [P] [US4] Write integration test for --color flag in tests/integration/cli_tests.rs
- [ ] T091 [P] [US4] Write contract test for JSON schema validation in tests/contract/json_schema.rs

### Implementation for User Story 4

- [ ] T092 [US4] Implement human-readable formatter with owo-colors in src/report/formatter.rs
- [ ] T093 [US4] Implement JSON output serialization in src/report/json.rs
- [ ] T094 [US4] Implement TTY detection and color control in src/cli/output.rs
- [ ] T095 [US4] Add --json flag to CLI arguments in src/cli/args.rs
- [ ] T096 [US4] Add --color flag with auto/always/never options in src/cli/args.rs
- [ ] T097 [US4] Implement NO_COLOR environment variable support in src/cli/output.rs
- [ ] T098 [US4] Integrate output formatters into main reporting in src/main.rs
- [ ] T099 [US4] Add severity level filtering in src/report/mod.rs
- [ ] T100 [US4] Add --quiet and --verbose flags in src/cli/args.rs
- [ ] T101 [US4] Verify all US4 tests pass

**Checkpoint**: All user stories complete - full feature set implemented

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements, performance optimization, and documentation

- [ ] T102 [P] Set up fuzzing infrastructure in fuzz/Cargo.toml
- [ ] T103 [P] Implement fuzz target for Unicode detection in fuzz/fuzz_targets/fuzz_unicode.rs
- [ ] T104 [P] Implement fuzz target for TOML parsing in fuzz/fuzz_targets/fuzz_config.rs
- [ ] T105 [P] Implement fuzz target for file scanning in fuzz/fuzz_targets/fuzz_file_scan.rs
- [ ] T106 [P] Add property-based tests with proptest in tests/unit/
- [ ] T107 [P] Create performance benchmarks with criterion in benches/scan_performance.rs
- [ ] T108 Optimize Unicode range lookups with interval trees in src/unicode/ranges.rs
- [ ] T109 Add memory mapping for large files with memmap2 in src/scanner/file_scanner.rs
- [ ] T110 Update README.md with full documentation
- [ ] T111 Add comprehensive --help text in src/cli/args.rs
- [ ] T112 Add rustdoc comments to all public APIs
- [ ] T113 Run cargo clippy and fix all warnings
- [ ] T114 Run cargo fmt to ensure consistent formatting
- [ ] T115 Run cargo test to verify all tests pass
- [ ] T116 Run cargo-tarpaulin for code coverage report
- [ ] T117 Update flake.nix with all checks (test, clippy, fmt)
- [ ] T118 Test quickstart.md examples for accuracy
- [ ] T119 Create CHANGELOG.md for version 1.0.0
- [ ] T120 Final validation: Run full test suite including fuzz tests

**Checkpoint**: Feature complete and polished

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3 → P4)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Builds on US1 but independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Builds on US1 but independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Enhances US1 output but independently testable

### Within Each User Story

1. Tests MUST be written and FAIL before implementation (Constitution requirement)
2. Data structures before business logic
3. Core implementation before integration
4. Story validation before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T003-T008)
- All Foundational module creations marked [P] can run in parallel (T012-T016, T020-T023)
- Once Foundational phase completes, all user stories CAN start in parallel (if team capacity allows)
- All tests within a user story marked [P] can run in parallel
- All unit implementations marked [P] within a story can run in parallel

---

## Parallel Example: User Story 1

```bash
# After foundational phase, launch all US1 tests together:
Task: "Write unit test for UnicodeRange contains() method"
Task: "Write unit test for detecting zero-width characters"
Task: "Write unit test for detecting bidi override characters"
Task: "Write unit test for detecting homoglyphs"
Task: "Write unit test for file scanning logic"
Task: "Write unit test for encoding detection"
# ... continue with all [P] marked test tasks

# After tests fail, launch all independent implementations:
Task: "Implement UnicodeRange struct and methods"
Task: "Implement malicious Unicode patterns database"
Task: "Implement Unicode character categorization"
# ... continue with all [P] marked implementation tasks
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (with all tests)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo MVP if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (tests then implementation)
   - Developer B: User Story 2 (tests then implementation)
   - Developer C: User Story 3 (tests then implementation)
   - Developer D: User Story 4 (tests then implementation)
3. Stories complete and integrate independently

### Test-Driven Development (MANDATORY)

Per Constitution Principle III:
1. Write tests for a user story FIRST
2. Get user approval of tests
3. Run tests and verify they FAIL
4. Implement code to make tests pass
5. Refactor while keeping tests green
6. NO implementation without failing tests first

---

## Task Summary

- **Total Tasks**: 120
- **Setup Tasks**: 9 (T001-T009)
- **Foundational Tasks**: 14 (T010-T023)
- **User Story 1 Tasks**: 27 (12 tests + 15 implementation)
- **User Story 2 Tasks**: 21 (9 tests + 12 implementation)
- **User Story 3 Tasks**: 13 (5 tests + 8 implementation)
- **User Story 4 Tasks**: 17 (7 tests + 10 implementation)
- **Polish Tasks**: 19 (T102-T120)
- **Parallel Opportunities**: 83 tasks marked with [P]

### MVP Scope
- Minimum Viable Product = Phase 1 + Phase 2 + Phase 3 (US1)
- Total MVP tasks: 50 tasks
- Delivers: Core malicious Unicode detection functionality

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Tests MUST fail before implementing (Constitution requirement)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Use `cargo test` frequently during development
- Run `cargo clippy` before committing