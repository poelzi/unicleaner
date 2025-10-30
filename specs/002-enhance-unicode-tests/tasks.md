# Tasks: Enhance Unicode Test Suite

**Input**: Design documents from `/specs/002-enhance-unicode-tests/`
**Prerequisites**: plan.md, research.md, data-model.md, quickstart.md

**Tests**: This feature IS about tests - enhancing the existing test suite

**Organization**: Tasks organized by testing category for comprehensive Unicode attack coverage

## Format: `- [ ] [ID] [P?] Description with file path`

- **[P]**: Can run in parallel (different files, no dependencies)
- **Task ID**: Sequential number (T001, T002, T003...) in execution order
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root (per plan.md)

---

## Phase 1: Setup (Test Infrastructure)

**Purpose**: Set up testing dependencies and infrastructure

- [x] T001 Add proptest dependency to Cargo.toml with version 1.4+
- [x] T002 Add cargo-fuzz to development dependencies in Cargo.toml
- [x] T003 [P] Create fuzz directory structure at fuzz/
- [x] T004 [P] Create fuzz/Cargo.toml with libfuzzer-sys configuration
- [x] T005 [P] Create tests/proptest/ directory for property-based tests
- [x] T006 [P] Create tests/fixtures/unicode_attacks/ directory structure
- [x] T007 [P] Add test-case dependency for parameterized tests in Cargo.toml
- [x] T008 [P] Create tests/common/mod.rs for shared test utilities

**Checkpoint**: Test infrastructure ready for implementation ✅

---

## Phase 2: Trojan Source Attack Tests (CVE-2021-42574, CVE-2021-42694)

**Purpose**: Implement comprehensive tests for Trojan Source vulnerabilities

### Test Fixtures

- [x] T009 [P] Create test fixture with RLO (U+202E) attack in tests/fixtures/unicode_attacks/trojan_source/rlo_attack.rs
- [x] T010 [P] Create test fixture with LRI (U+2066) hiding attack in tests/fixtures/unicode_attacks/trojan_source/lri_hiding.rs
- [x] T011 [P] Create test fixture with RLI (U+2067) reordering in tests/fixtures/unicode_attacks/trojan_source/rli_reorder.rs
- [x] T012 [P] Create test fixture with FSI (U+2068) isolation attack in tests/fixtures/unicode_attacks/trojan_source/fsi_attack.rs
- [x] T013 [P] Create test fixture with PDI (U+2069) pop isolation in tests/fixtures/unicode_attacks/trojan_source/pdi_attack.rs
- [x] T014 [P] Create test fixture combining multiple bidi attacks in tests/fixtures/unicode_attacks/trojan_source/combined_bidi.rs

### Integration Tests

- [x] T015 Write integration test for RLO detection in tests/integration/trojan_source_tests.rs
- [x] T016 Write integration test for LRI/RLI/FSI detection in tests/integration/trojan_source_tests.rs
- [x] T017 Write integration test for nested bidi overrides in tests/integration/trojan_source_tests.rs
- [x] T018 Write integration test for bidi in comments vs code in tests/integration/trojan_source_tests.rs

**Checkpoint**: Trojan Source attack detection fully tested ✅

---

## Phase 3: Homoglyph Attack Tests

**Purpose**: Test detection of visually similar characters used for deception

### Test Fixtures

- [x] T019 [P] Create Cyrillic homoglyph fixtures (а vs a) in tests/fixtures/unicode_attacks/homoglyphs/cyrillic.rs
- [x] T020 [P] Create Greek homoglyph fixtures (ο vs o) in tests/fixtures/unicode_attacks/homoglyphs/greek.rs
- [x] T021 [P] Create mathematical alphanumeric fixtures in tests/fixtures/unicode_attacks/homoglyphs/math_alphanumeric.rs
- [x] T022 [P] Create fullwidth character fixtures in tests/fixtures/unicode_attacks/homoglyphs/fullwidth.rs
- [x] T023 [P] Create confusable identifier test cases in tests/fixtures/unicode_attacks/homoglyphs/identifiers.rs
- [x] T024 [P] Create mixed script fixtures in tests/fixtures/unicode_attacks/homoglyphs/mixed_scripts.rs

### Integration Tests

- [x] T025 Write integration test for Cyrillic homoglyph detection in tests/integration/homoglyph_tests.rs
- [x] T026 Write integration test for Greek homoglyph detection in tests/integration/homoglyph_tests.rs
- [x] T027 Write integration test for mathematical character detection in tests/integration/homoglyph_tests.rs
- [x] T028 Write integration test for mixed script detection in tests/integration/homoglyph_tests.rs
- [x] T029 Write test for homoglyph severity levels in tests/integration/homoglyph_tests.rs

**Checkpoint**: Homoglyph attack detection fully tested ✅

---

## Phase 4: Zero-Width and Invisible Character Tests

**Purpose**: Test detection of characters that hide or are invisible

### Test Fixtures

- [x] T030 [P] Create ZWSP (U+200B) test fixtures in tests/fixtures/unicode_attacks/zero_width/zwsp.rs
- [x] T031 [P] Create ZWNJ (U+200C) test fixtures in tests/fixtures/unicode_attacks/zero_width/zwnj.rs
- [x] T032 [P] Create ZWJ (U+200D) test fixtures in tests/fixtures/unicode_attacks/zero_width/zwj.rs
- [x] T033 [P] Create ZWNBSP/BOM (U+FEFF) test fixtures in tests/fixtures/unicode_attacks/zero_width/bom.rs
- [x] T034 [P] Create combining character abuse fixtures in tests/fixtures/unicode_attacks/zero_width/combining.rs
- [x] T035 [P] Create invisible separator fixtures in tests/fixtures/unicode_attacks/zero_width/separators.rs

### Integration Tests

- [x] T036 Write integration test for ZWSP detection in tests/integration/zero_width_tests.rs
- [x] T037 Write integration test for ZWNJ/ZWJ detection in tests/integration/zero_width_tests.rs
- [x] T038 Write integration test for BOM in middle of file in tests/integration/zero_width_tests.rs
- [x] T039 Write integration test for combining character stacking in tests/integration/zero_width_tests.rs
- [x] T040 Write test for zero-width character in identifiers in tests/integration/zero_width_tests.rs

**Checkpoint**: Zero-width character detection fully tested ✅

---

## Phase 5: Property-Based Testing with Proptest

**Purpose**: Implement property-based tests to find edge cases automatically

### Unicode Property Tests

- [x] T041 [P] Write proptest for Unicode category invariants in tests/proptest/unicode_categories.rs
- [x] T042 [P] Write proptest for Unicode range boundary conditions in tests/proptest/unicode_ranges.rs
- [x] T043 [P] Write proptest for character normalization invariants in tests/proptest/normalization.rs
- [x] T044 [P] Write proptest for bidi character combinations in tests/proptest/bidi_properties.rs
- [x] T045 [P] Write proptest for homoglyph detection accuracy in tests/proptest/homoglyph_properties.rs

### Scanner Property Tests

- [x] T046 Write proptest for scanner never panics on any input in tests/proptest/scanner_stability.rs
- [x] T047 Write proptest for scanner determinism (same input = same output) in tests/proptest/scanner_determinism.rs
- [x] T048 Write proptest for config validation robustness in tests/proptest/config_properties.rs
- [x] T049 Write proptest for encoding detection consistency in tests/proptest/encoding_properties.rs

**Checkpoint**: Property-based testing ensures robustness ✅

---

## Phase 6: Fuzzing Infrastructure

**Purpose**: Set up fuzzing to discover crashes and edge cases

### Fuzz Targets

- [x] T050 Create fuzz target for Unicode detection in fuzz/fuzz_targets/unicode_detection.rs
- [x] T051 Create fuzz target for file scanning in fuzz/fuzz_targets/file_scanner.rs
- [x] T052 Create fuzz target for TOML config parsing in fuzz/fuzz_targets/config_parser.rs
- [x] T053 Create fuzz target for encoding detection in fuzz/fuzz_targets/encoding_detection.rs
- [x] T054 Create fuzz target for homoglyph detection in fuzz/fuzz_targets/homoglyph_detector.rs

### Corpus Generation

- [x] T055 [P] Generate initial corpus for Unicode fuzzing in fuzz/corpus/unicode/
- [x] T056 [P] Generate config file corpus in fuzz/corpus/config/
- [x] T057 [P] Generate mixed encoding corpus in fuzz/corpus/encoding/
- [x] T058 [P] Create dictionary of Unicode attack patterns in fuzz/dictionary.txt

**Checkpoint**: Fuzzing infrastructure ready for continuous testing ✅

---

## Phase 7: Edge Cases and Regression Tests

**Purpose**: Add tests for specific edge cases and known issues

### Edge Case Tests

- [x] T059 [P] Test extremely long lines (>10000 chars) in tests/integration/edge_cases.rs
- [x] T060 [P] Test files with millions of Unicode characters in tests/integration/edge_cases.rs
- [x] T061 [P] Test invalid UTF-8 sequences handling in tests/integration/edge_cases.rs
- [x] T062 [P] Test mixed UTF-8/UTF-16/UTF-32 detection in tests/integration/edge_cases.rs
- [x] T063 [P] Test symlink and circular reference handling in tests/integration/edge_cases.rs
- [x] T064 [P] Test permission denied scenarios in tests/integration/edge_cases.rs

### Regression Tests

- [x] T065 [P] Add regression test for issue #1 (if exists) in tests/regression/
- [x] T066 [P] Add performance regression test in tests/performance/benchmarks.rs
- [x] T067 [P] Add memory usage regression test in tests/performance/memory.rs

**Checkpoint**: Edge cases covered, regressions prevented ✅

---

## Phase 8: Normalization and Canonicalization Tests

**Purpose**: Test Unicode normalization attacks and canonical equivalence issues

### Normalization Tests

- [x] T068 [P] Create NFC/NFD confusion test fixtures in tests/fixtures/unicode_attacks/normalization/nfc_nfd.rs
- [x] T069 [P] Create NFKC/NFKD test fixtures in tests/fixtures/unicode_attacks/normalization/nfkc_nfkd.rs
- [x] T070 [P] Test canonical combining character reordering in tests/integration/normalization_tests.rs
- [x] T071 [P] Test compatibility character detection in tests/integration/normalization_tests.rs
- [x] T072 [P] Test normalized vs denormalized identifier comparison in tests/integration/normalization_tests.rs

**Checkpoint**: Normalization attacks fully tested ✅

---

## Phase 9: Documentation and CI Integration

**Purpose**: Document test suite and integrate with CI

### Documentation

- [x] T073 [P] Document test fixture format in tests/fixtures/README.md
- [x] T074 [P] Document fuzzing setup and usage in fuzz/README.md
- [x] T075 [P] Document property-based testing patterns in tests/proptest/README.md
- [x] T076 [P] Update main README.md with test running instructions

### CI Integration

- [x] T077 Create GitHub Actions workflow for test suite in .github/workflows/test.yml
- [x] T078 Add fuzzing job to CI (time-bounded) in .github/workflows/fuzz.yml
- [x] T079 Add code coverage reporting with tarpaulin in .github/workflows/coverage.yml
- [x] T080 Configure test result reporting in CI

**Checkpoint**: Test suite fully documented and integrated ✅

---

## Phase 10: Performance Testing

**Purpose**: Ensure detection performance meets requirements (<5 seconds for typical codebases)

### Benchmark Tests

- [x] T081 [P] Create benchmark for small repository (100 files) in benches/small_repo.rs
- [x] T082 [P] Create benchmark for medium repository (1000 files) in benches/medium_repo.rs
- [x] T083 [P] Create benchmark for large repository (10000 files) in benches/large_repo.rs
- [x] T084 [P] Create benchmark for Unicode-heavy files in benches/unicode_heavy.rs
- [x] T085 [P] Create memory usage benchmark in benches/memory_usage.rs

### Performance Tests

- [x] T086 Test scanner completes in <5 seconds for 1000-file repo in tests/performance/time_limits.rs
- [x] T087 Test memory usage stays under 500MB for large repos in tests/performance/memory_limits.rs
- [x] T088 Test parallel scanning performance scaling in tests/performance/parallel_scaling.rs

**Checkpoint**: Performance validated against requirements ✅

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phases 2-4 (Attack Tests)**: Depend on Phase 1, can run in parallel with each other
- **Phase 5 (Property Testing)**: Depends on Phase 1, can run in parallel with 2-4
- **Phase 6 (Fuzzing)**: Depends on Phase 1, can run in parallel with 2-5
- **Phase 7 (Edge Cases)**: Can start after Phase 1
- **Phase 8 (Normalization)**: Can start after Phase 1
- **Phase 9 (Documentation)**: Can start after Phases 2-8 complete
- **Phase 10 (Performance)**: Should run after core tests (Phases 2-4) complete

### Parallel Opportunities

- All tasks marked [P] within a phase can run in parallel
- Phases 2-8 can all proceed in parallel after Phase 1
- Different test categories (Trojan Source, Homoglyphs, Zero-width) are independent
- Property tests and fuzz tests can be developed in parallel with integration tests

---

## Parallel Example: Phase 2 (Trojan Source Tests)

```bash
# Launch all test fixtures in parallel:
Task: "Create test fixture with RLO (U+202E) attack"
Task: "Create test fixture with LRI (U+2066) hiding attack"
Task: "Create test fixture with RLI (U+2067) reordering"
Task: "Create test fixture with FSI (U+2068) isolation attack"
Task: "Create test fixture with PDI (U+2069) pop isolation"
Task: "Create test fixture combining multiple bidi attacks"

# After fixtures are created, run integration tests sequentially
```

---

## Implementation Strategy

### Incremental Testing Approach

1. **Week 1**: Complete Phase 1 (Setup) + Start Phase 2 (Trojan Source)
2. **Week 2**: Complete Phases 2-4 (Core attack tests)
3. **Week 3**: Add Phases 5-6 (Property testing + Fuzzing)
4. **Week 4**: Complete Phases 7-8 (Edge cases + Normalization)
5. **Week 5**: Finalize with Phases 9-10 (Documentation + Performance)

### Test Coverage Goals

- Line coverage: >90% of scanner code
- Branch coverage: >85% of decision paths
- Mutation testing score: >80% (if using cargo-mutants)
- Fuzz testing: 24+ hours without crashes
- Property tests: 10000+ cases per property

### Validation Checkpoints

1. After Phase 2: Verify all Trojan Source CVEs detected
2. After Phase 4: Verify all Unicode Security Standard attacks detected
3. After Phase 6: Run 1-hour fuzz session without crashes
4. After Phase 10: Verify <5 second scan time on reference repo

---

## Task Summary

- **Total Tasks**: 88
- **Setup Tasks**: 8 (T001-T008)
- **Trojan Source Tests**: 10 (T009-T018)
- **Homoglyph Tests**: 11 (T019-T029)
- **Zero-Width Tests**: 11 (T030-T040)
- **Property Tests**: 9 (T041-T049)
- **Fuzzing Tasks**: 9 (T050-T058)
- **Edge Case Tests**: 9 (T059-T067)
- **Normalization Tests**: 5 (T068-T072)
- **Documentation Tasks**: 8 (T073-T080)
- **Performance Tests**: 8 (T081-T088)
- **Parallel Opportunities**: 61 tasks marked with [P]

### Testing Focus
- **Primary Goal**: Comprehensive Unicode attack detection coverage
- **Key Deliverables**: Property tests, fuzz tests, attack fixtures
- **Success Metric**: Zero false negatives on known attack patterns

---

## Notes

- This is a TEST ENHANCEMENT feature, not new functionality
- Focus on detecting Unicode attacks per UTS #39 and Trojan Source research
- All test fixtures should include both malicious and benign examples
- Property tests should generate diverse Unicode inputs automatically
- Fuzzing should run continuously in CI (time-bounded)
- Performance tests validate <5 second requirement from plan.md
- Tests follow TDD approach per Constitution Principle III