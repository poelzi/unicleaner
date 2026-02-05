# Tasks: Named Unicode Range Support

**Input**: Design documents from `/specs/003-named-unicode-ranges/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. Per constitution Principle III (TDD), test tasks appear before implementation within each phase. Tests must be written, reviewed, and verified to fail before implementation proceeds.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependencies and create module structure

- [x] T001 Add `unicode-blocks`, `strsim`, and `once_cell` dependencies to `Cargo.toml`
- [x] T002 Add `pub mod blocks;` to `src/unicode/mod.rs`

---

## Phase 2: Foundational (BlockRegistry Core)

**Purpose**: Build the BlockRegistry that all user stories depend on. MUST complete before any user story.

**TDD**: Write tests first (T003), verify they fail, then implement (T004-T007).

- [x] T003 Write unit tests in `src/unicode/blocks.rs` (in a `#[cfg(test)] mod tests` block) for: resolve official name ("Basic Latin" returns U+0000-U+007F), resolve with case variations ("basic latin", "BASIC LATIN"), resolve unknown name "Nonexistent" returns `BlockError` with suggestions. Create minimal stub structs/functions so tests compile but fail.
- [x] T004 Create `BlockEntry` struct (name: String, start: u32, end: u32) and `BlockError` enum with `UnknownBlock { name: String, suggestions: Vec<String> }` variant implementing `std::fmt::Display` and `std::error::Error` in `src/unicode/blocks.rs`
- [x] T005 Implement `BlockRegistry` struct in `src/unicode/blocks.rs` with a `once_cell::sync::Lazy<HashMap<String, BlockEntry>>` that populates all ~330 Unicode blocks from `unicode_blocks` crate constants, keyed by lowercased official name
- [x] T006 Implement `BlockRegistry::resolve(name: &str) -> Result<UnicodeRange, BlockError>` in `src/unicode/blocks.rs` - lowercase input, lookup in map, return `UnicodeRange` on hit or `BlockError` with suggestions on miss
- [x] T007 Implement `BlockRegistry::suggest(name: &str) -> Vec<String>` in `src/unicode/blocks.rs` using `strsim::jaro_winkler` to return top 3 similar block names from the registry
- [x] T008 Verify all T003 tests now pass. Run `cargo clippy` and `cargo fmt --check` on `src/unicode/blocks.rs`

**Checkpoint**: BlockRegistry can resolve official Unicode block names. All unit tests green. Foundation ready for user stories.

---

## Phase 3: User Story 1 - Configure Allowed Blocks by Name (Priority: P1) 🎯 MVP

**Goal**: Users can specify `allowed_blocks = ["Basic Latin", "Hebrew"]` in config and the scanner respects those ranges.

**Independent Test**: Create a config with `allowed_blocks`, scan a file with characters from those ranges, verify correct acceptance/rejection.

**TDD**: Write tests first (T009-T010), verify they fail, then implement (T011-T012).

### Tests for User Story 1

- [x] T009 [P] [US1] Write integration test in `tests/integration/block_config_tests.rs`: config with `allowed_blocks = ["Basic Latin"]` accepts ASCII, rejects Greek characters. Test should compile but fail (config parsing doesn't support `allowed_blocks` yet).
- [x] T010 [P] [US1] Write integration test in `tests/integration/block_config_tests.rs`: config with unrecognized block name `"Nonexistent Block"` produces error message containing the invalid name and at least one suggestion.

### Implementation for User Story 1

- [x] T011 [US1] Add `allowed_blocks: Vec<String>` field with `#[serde(default)]` to `RuleConfig` struct in `src/config/parser.rs`
- [x] T012 [US1] Extend `load_config()` in `src/config/parser.rs` to resolve each `allowed_blocks` entry via `BlockRegistry::resolve()`, adding the resulting `UnicodeRange` to `FileRule.allowed_ranges`. Return error with suggestions on invalid block names (FR-001, FR-006, FR-010)
- [x] T013 [US1] Verify T009 and T010 tests now pass. Run `cargo clippy` and `cargo fmt --check`.

**Checkpoint**: User Story 1 complete. Users can configure named blocks and scan files. Invalid names produce helpful errors. All tests green.

---

## Phase 4: User Story 2 - Use Multiple Named Blocks Together (Priority: P1)

**Goal**: Users can specify multiple blocks in a single rule and combine `allowed_blocks` with `allowed_ranges` using union semantics.

**Independent Test**: Config with multiple blocks and mixed block/range fields, scan file with mixed scripts, verify union semantics.

**TDD**: Write tests first (T014-T015), verify they fail (or pass if US1 implementation already covers it), then confirm correctness.

### Tests for User Story 2

- [x] T014 [P] [US2] Write integration test in `tests/integration/block_config_tests.rs`: config with `allowed_blocks = ["Basic Latin", "Hebrew"]` accepts both ASCII and Hebrew, rejects Cyrillic (FR-002)
- [x] T015 [P] [US2] Write integration test in `tests/integration/block_config_tests.rs`: config with `allowed_blocks = ["Basic Latin"]` and `allowed_ranges = [[0x0400, 0x04FF]]` applies union semantics correctly - both Latin and Cyrillic accepted (FR-003, FR-009)

### Implementation for User Story 2

- [x] T016 [US2] Verify T014 and T015 tests pass (US1 implementation should already handle multiple blocks and union semantics). If any fail, fix the config loading logic in `src/config/parser.rs`. Run `cargo clippy` and `cargo fmt --check`.

**Checkpoint**: User Story 2 complete. Multiple blocks and mixed block/range configs work with union semantics. All tests green.

---

## Phase 5: User Story 3 - Use Short Aliases for Common Blocks (Priority: P2)

**Goal**: Users can use short aliases like `"ascii"`, `"latin-1"`, `"hebrew"` instead of full official block names.

**Independent Test**: Config with alias names resolves to correct code point ranges.

**TDD**: Write tests first (T017-T018), verify they fail, then implement (T019).

### Tests for User Story 3

- [x] T017 [P] [US3] Write unit tests in `src/unicode/blocks.rs` for: resolve each alias ("ascii" → Basic Latin U+0000-U+007F, "latin-1" → Latin-1 Supplement U+0080-U+00FF, "hebrew" → Hebrew U+0590-U+05FF), verify alias and official name resolve to same range (FR-004, FR-005). Tests should fail since aliases aren't registered yet.
- [x] T018 [P] [US3] Write integration test in `tests/integration/block_config_tests.rs`: config with `allowed_blocks = ["ascii"]` accepts ASCII characters and rejects non-ASCII. Test should fail since "ascii" alias isn't registered yet.

### Implementation for User Story 3

- [x] T019 [US3] Add alias entries to the `BlockRegistry` HashMap in `src/unicode/blocks.rs` for all 13 defined aliases: ascii, latin-1, latin-extended-a, latin-extended-b, greek, cyrillic, hebrew, arabic, cjk, hangul, hiragana, katakana, emoji (FR-004)
- [x] T020 [US3] Verify T017 and T018 tests now pass. Run `cargo clippy` and `cargo fmt --check`.

**Checkpoint**: User Story 3 complete. Short aliases resolve correctly alongside official names. All tests green.

---

## Phase 6: User Story 4 - Discover Available Block Names (Priority: P2)

**Goal**: Users can run `unicleaner list-blocks` to see all available block names, ranges, and aliases.

**Independent Test**: Run the list command and verify all blocks are shown with code point ranges.

**TDD**: Write tests first (T021), verify they fail, then implement (T022-T024).

### Tests for User Story 4

- [x] T021 [US4] Write unit tests in `src/unicode/blocks.rs` for: `list_blocks(None)` returns all blocks sorted by start code point, `list_blocks(Some("hebrew"))` returns only matching blocks, `list_blocks(Some("zzzzz"))` returns empty vec. Create stub `list_blocks` function and `BlockInfo` struct so tests compile but fail.

### Implementation for User Story 4

- [x] T022 [US4] Implement `BlockInfo` struct and `BlockRegistry::list_blocks(filter: Option<&str>) -> Vec<BlockInfo>` in `src/unicode/blocks.rs` returning all blocks sorted by start code point, with optional case-insensitive substring filter. `BlockInfo` includes name, start, end, and aliases. Include Unicode version in output header (FR-007).
- [x] T023 [US4] Add `ListBlocks` variant to `Command` enum in `src/cli/args.rs` with optional `filter: Option<String>` argument
- [x] T024 [US4] Handle `ListBlocks` command in `src/main.rs`: call `BlockRegistry::list_blocks()`, format output as table with columns: Name, Range (U+XXXX-U+XXXX), Aliases
- [x] T025 [US4] Verify T021 tests now pass. Run `cargo clippy` and `cargo fmt --check`.

**Checkpoint**: User Story 4 complete. Users can discover available blocks via CLI. All tests green.

---

## Phase 7: Fuzz Testing

**Purpose**: Fuzz test block name resolution per constitution Principle IV (fuzz testing MANDATORY).

- [x] T026 Add fuzz target for `BlockRegistry::resolve()` in `fuzz/fuzz_targets/` that feeds arbitrary strings to the resolve function, verifying it never panics (returns `Ok` or `Err` gracefully)
- [x] T027 Run fuzz target for a brief session (`cargo fuzz run fuzz_block_resolve -- -max_total_time=60`) and fix any panics discovered

**Checkpoint**: Fuzz testing confirms BlockRegistry handles arbitrary input without panics.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, backward compatibility, and cleanup

- [x] T028 [P] Write and run backward compatibility test in `tests/integration/block_config_tests.rs`: existing configs with only `allowed_ranges` (no `allowed_blocks`) continue to work unchanged (FR-009)
- [x] T029 [P] Run full quality gate: `cargo test --all-features`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- [x] T030 Run quickstart.md validation: verify the config examples from `specs/003-named-unicode-ranges/quickstart.md` work as documented

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Phase 2. Core MVP.
- **User Story 2 (Phase 4)**: Depends on Phase 3 (needs config parsing from US1). Tests union semantics.
- **User Story 3 (Phase 5)**: Depends on Phase 2 only. Can run in parallel with US1/US2.
- **User Story 4 (Phase 6)**: Depends on Phase 2 only (and US3 for alias display). Can mostly run in parallel with US1/US2.
- **Fuzz Testing (Phase 7)**: Depends on Phase 2 (BlockRegistry must exist). Can run after foundational phase.
- **Polish (Phase 8)**: Depends on all user stories and fuzz testing being complete.

### TDD Workflow Within Each Phase

1. Write tests → verify they compile but fail
2. Implement minimal code to make tests pass
3. Run `cargo clippy` and `cargo fmt --check`
4. Proceed to next phase

### Parallel Opportunities

- T001 and T002 can run in parallel (different files)
- US3 (aliases) can start as soon as Phase 2 is complete, parallel with US1
- US4 (list command) can start as soon as Phase 2 is complete, parallel with US1
- Fuzz testing (Phase 7) can start after Phase 2, parallel with user stories
- T028 and T029 can run in parallel in Polish phase
- Within each user story, test tasks marked [P] can run in parallel

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational BlockRegistry with TDD (T003-T008)
3. Complete Phase 3: User Story 1 with TDD (T009-T013)
4. Complete Phase 4: User Story 2 with TDD (T014-T016)
5. **STOP and VALIDATE**: Test with real config files

### Incremental Delivery

1. Setup + Foundational → BlockRegistry works (all unit tests green)
2. Add US1 → Named blocks in config → MVP ready
3. Add US2 → Multiple blocks + union → Core feature complete
4. Add US3 → Aliases → Improved UX
5. Add US4 → List command → Full discoverability
6. Fuzz testing → Robustness verified
7. Polish → Production ready

---

## Notes

- `unicode-blocks` crate provides constants but no ALL_BLOCKS array - must enumerate manually or use a macro
- Block names are resolved at config load time (FR-010) - no runtime overhead during scanning
- After resolution, named blocks are indistinguishable from numeric ranges in `FileRule.allowed_ranges`
- The `emoji` alias maps to Emoticons (U+1F600-U+1F64F); users needing other emoji blocks use official names
- MSRV is 1.85 (edition 2024); uses `once_cell::sync::Lazy` for lazy initialization
- Duplicate ranges from overlapping `allowed_blocks` and `allowed_ranges` are harmless - union semantics means both match, no deduplication needed
