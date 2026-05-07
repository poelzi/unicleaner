# Tasks: Cleaner API

**Input**: Design documents from `/specs/006-cleaner-api/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. Per constitution Principle III (TDD), test tasks appear before implementation within each phase. Tests must be written, reviewed, and verified to fail before implementation proceeds.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add the new dependency and create module skeletons.

- [ ] T001 Add `unicode-normalization = "0.1"` to `[dependencies]` in `Cargo.toml`. Verify the dep brings no new transitive crates by running `cargo tree -p unicode-normalization` and recording the result in a comment on the cleaner WP commit.
- [ ] T002 [P] Create empty `src/cleaner/mod.rs` and `src/cleaner/policy.rs` placeholders. Add `pub mod cleaner;` to `src/lib.rs`. Verify `cargo check` succeeds (no public items yet).
- [ ] T003 [P] Create empty `tests/integration/cleaner_tests.rs` and `tests/integration/cli_clean_tests.rs` referenced from `tests/integration/main.rs` so the new test files are picked up.

---

## Phase 2: Foundational — `CleanPolicy` + `CleanAction` (no user story yet)

**Purpose**: Build the policy types every user story depends on. MUST complete before any user story.

**TDD**: Write tests first (T004), verify they fail, then implement (T005-T008).

- [ ] T004 [P] Write unit tests in `src/cleaner/policy.rs` `#[cfg(test)] mod tests` that assert:
  1. `CleanPolicy::strict()` has `default_action == CleanAction::Strip`, NFC off, no overrides.
  2. `CleanPolicy::lossy()` has `default_action == CleanAction::Replace('\u{FFFD}')`.
  3. `CleanPolicy::report_only()` has `default_action == CleanAction::KeepWithMark`.
  4. `with_action(BidiOverride, Replace('?')).effective_action(BidiOverride) == Replace('?')`.
  5. `effective_action` falls through to `default_action` for an un-overridden category.
  6. `CleanPolicy::strict() == CleanPolicy::strict()` (PartialEq derive works).
  7. Round-trip a policy through `serde_json` and back; equality preserved.
  Run `cargo test -p unicleaner --lib cleaner::policy` and verify each test fails for the right reason ("not yet defined").
- [ ] T005 Define `CleanAction` enum in `src/cleaner/policy.rs` with the three variants. Derive `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`. Document each variant per data-model.md.
- [ ] T006 Define `CleanPolicy` struct with the six fields per data-model.md §`CleanPolicy`. Derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`.
- [ ] T007 Implement `strict()`, `lossy()`, `report_only()` constructors and the `with_*` chained mutators (`with_action`, `with_default_action`, `with_nfc`, `with_denied`, `with_allowed_ranges`).
- [ ] T008 Implement crate-private `effective_action(&self, category) -> CleanAction`. Run `cargo test cleaner::policy` and verify all T004 tests pass. Run `cargo clippy -p unicleaner -- -D warnings` and `cargo fmt --check`.

**Checkpoint**: `CleanPolicy` is fully self-contained. `clean()` does not yet exist — the next phase consumes the policy.

---

## Phase 3: User Story 1 — Sanitize a String In Process (Priority: P1) 🎯 MVP

**Goal**: `pub fn clean(input: &str, policy: &CleanPolicy) -> CleanResult` works end to end with the strict / lossy / report-only presets.

**Independent Test**: `cargo test -p unicleaner --test cleaner_tests` — all unit + integration tests pass.

**TDD**: Tests T009 / T010 / T011 written and verified failing before T012 / T013 / T014.

- [ ] T009 [P] [US1] Write integration tests in `tests/integration/cleaner_tests.rs`:
  1. `clean_strict_strips_zero_width` — `"hi\u{200B}there"` → `"hithere"`, `modified == true`, one violation U+200B.
  2. `clean_lossy_replaces_with_fffd` — bidi-override input → `U+FFFD` substitution.
  3. `clean_report_only_no_mutation` — input equals output, violations recorded, `modified == false`.
  4. `clean_clean_input_borrows` — plain ASCII input + strict policy → `output` is `Cow::Borrowed`, pointer equal to input.
  5. `clean_per_category_override` — strict + `with_action(Homoglyph, KeepWithMark)` keeps homoglyphs while stripping zero-widths.
  6. `clean_empty_input` — `""` round-trips, `Cow::Borrowed`, no violations.
  7. `clean_all_malicious_input` — `"\u{200B}\u{202E}"` strict → `""`, two violations, `Cow::Owned`.
  Run `cargo test -p unicleaner --test cleaner_tests` and verify all fail (function doesn't exist).
- [ ] T010 [P] [US1] Write unit tests in `src/cleaner/mod.rs` `#[cfg(test)]` for `decide_action` and the `needs_mutation` pre-scan helper:
  - `decide_action` returns `None` for codepoints not in the malicious table and not in `denied_code_points`.
  - `decide_action` returns the per-category override when present.
  - `needs_mutation` returns `false` for an empty string.
  - `needs_mutation` returns `false` for plain ASCII regardless of policy.
  - `needs_mutation` returns `true` when at least one matched codepoint is encountered.
- [ ] T011 [P] [US1] Write unit tests for `CleanResult` field invariants (per data-model.md §Invariants):
  - I-1: `!modified` ⇒ output bytes equal input bytes.
  - I-2: clean input + NFC off ⇒ `Cow::Borrowed`.
- [ ] T012 [US1] Define `CleanResult<'a>` in `src/cleaner/mod.rs` per data-model.md.
- [ ] T013 [US1] Implement `decide_action(cp, policy) -> Option<(MaliciousCategory, &'static MaliciousPattern, CleanAction)>` in `src/cleaner/mod.rs`. Consults `pattern_for()`, then `policy.denied_code_points`, then the `deny_by_default` rule. Crate-private.
- [ ] T014 [US1] Implement `pub fn clean(input, policy) -> CleanResult` per `plan.md::Architecture::clean() hot loop`. Use `String::with_capacity(input.len())` for the output buffer; call `decide_action` per char; emit `Violation::new(...)` (path = `<inline>`) for every match. Skip the allocation when `needs_mutation == false && !policy.normalize_nfc`. Run all T009 / T010 / T011 tests and verify they pass.
- [ ] T015 [US1] Re-export from `src/lib.rs`: `pub use cleaner::{clean, CleanPolicy, CleanResult, CleanAction};`. Run `cargo doc --no-deps` and confirm the new items are documented.

**Checkpoint**: US1 acceptance scenarios are satisfied. The library API is usable. `cargo test` is green; `cargo clippy -- -D warnings` is clean.

---

## Phase 4: User Story 1.5 — NFC Normalization (Priority: P1, depends on US1)

**Goal**: `policy.normalize_nfc = true` correctly applies NFC after stripping. Order-of-operations is asserted by a dedicated test (research.md Decision 3).

**TDD**: T016 written and verified failing before T017.

- [ ] T016 [US1] Write integration tests in `tests/integration/cleaner_tests.rs`:
  - `clean_nfc_normalizes_e_acute` — `"e\u{0301}"` + strict.with_nfc(true) → `"é"`, `modified == true`, no violations.
  - `clean_nfc_off_preserves_decomposed` — same input + NFC off → input unchanged.
  - `clean_nfc_runs_after_strip` — `"\u{200B}e\u{0301}"` strict.with_nfc(true) → `"é"`, one violation; output is the NFC-normalized form of the post-strip text. (This proves order: stripping first leaves `"e\u{0301}"`, NFC then normalizes.)
- [ ] T017 [US1] Add the NFC pass to `clean()`: when `policy.normalize_nfc && !is_nfc(&output)`, call `output.nfc().collect::<String>()` and update `modified`. Verify all T016 tests pass.

**Checkpoint**: NFC is opt-in, ordered correctly, and gated by tests that would catch a regression.

---

## Phase 5: User Story 2 — CLI `clean` Subcommand (Priority: P1, depends on US1)

**Goal**: `unicleaner clean PATH [...]` works end to end (stdout + `--in-place`).

**Independent Test**: `cargo test -p unicleaner --test cli_clean_tests` — all integration tests pass.

**TDD**: T018 written and verified failing before T019 / T020.

- [ ] T018 [P] [US2] Add fixtures under `tests/fixtures/cleaner/`:
  - `zwsp.txt` — sample with U+200B sprinkled in.
  - `zwsp.cleaned.txt` — golden cleaned output.
  - `clean.rs` — input with no violations.
  - `clean.cleaned.rs` — byte-identical to `clean.rs`.
  - `bidi.txt` + `bidi.cleaned.txt` — bidi-override sample.
- [ ] T019 [P] [US2] Write CLI integration tests in `tests/integration/cli_clean_tests.rs` using `assert_cmd` (already in dev-deps):
  1. `cli_clean_stdout_default` — `unicleaner clean fixtures/zwsp.txt` stdout byte-equals `fixtures/zwsp.cleaned.txt`, exit 0.
  2. `cli_clean_clean_file_unchanged` — `unicleaner clean fixtures/clean.rs` stdout byte-equals input.
  3. `cli_clean_in_place_atomic` — `unicleaner clean --in-place <copy of zwsp.txt>` rewrites the file in place; original copy under tempdir, post-run content equals golden, no `.tmp` left behind.
  4. `cli_clean_missing_file_errors` — `unicleaner clean nope.txt` exit code non-zero, stderr contains "no such file" or equivalent.
  5. `cli_clean_policy_lossy_flag` — `--policy lossy fixtures/zwsp.txt` produces the FFFD-replacement output.
  6. `cli_clean_stdin_dash` — `cat fixtures/zwsp.txt | unicleaner clean -` produces the cleaned output on stdout.
- [ ] T020 [US2] Add `Clean { paths, in_place, policy_preset, config }` variant to the existing CLI subcommand enum in `src/cli/mod.rs`. Wire it into the dispatch in `src/main.rs`.
- [ ] T021 [US2] Implement `src/cli/clean.rs::run(args)`: read file (or stdin), build the policy from `--policy {strict|lossy|report-only}` (default strict), call `clean()`, write to stdout or atomic-rename. Use the same atomic write pattern (`.tmp` → fsync → rename) referenced from `src/scanner/file_scanner.rs`.
- [ ] T022 [US2] Run `cargo test --test cli_clean_tests` — verify all T019 tests pass. Run `cargo clippy -- -D warnings` and `cargo fmt --check`.

**Checkpoint**: US2 acceptance scenarios are satisfied. `unicleaner clean --help` lists all flags.

---

## Phase 6: User Story 3 — Per-Category Policy Tuning (Priority: P2)

**Goal**: Operators can mix-and-match `CleanAction`s per category from both the library and the CLI.

**Independent Test**: One library test (T023) + one CLI test exercising the new TOML field (T024).

- [ ] T023 [P] [US3] Write a library test that constructs `CleanPolicy::strict().with_action(MaliciousCategory::Homoglyph, CleanAction::KeepWithMark).with_action(MaliciousCategory::BidiOverride, CleanAction::Replace('?'))` and verifies the per-codepoint behaviour against a hand-crafted multi-category fixture.
- [ ] T024 [P] [US3] Write a CLI integration test that exercises `--config <toml>` where the TOML has an `[cleaner]` block with `default_action = "strip"` and `[cleaner.per_category]` overrides; verify the CLI produces the same cleaned output as the equivalent library call.
- [ ] T025 [US3] Wire `CleanPolicy` deserialization through `serde` so the existing TOML loader picks up the optional `[cleaner]` block. The block is **optional** — files without it work as before.
- [ ] T026 [US3] Update `examples/unicleaner.toml` per AGENTS.md "Config Sync Rules": add a fully-commented `[cleaner]` block showing every option with its default. Run the existing `test_example_config_loads_successfully` integration test and verify it still passes.

**Checkpoint**: US3 acceptance scenarios are satisfied. The example config still loads.

---

## Phase 7: Performance Benchmark

**Purpose**: Lock in the perf budget per spec.md §SC-003.

- [ ] T027 [P] Add `benches/clean_throughput.rs` with two cases:
  - 4 MiB of plain ASCII input + `CleanPolicy::strict()` — measures the no-op fast path.
  - 4 MiB of mixed input (1 % violations) + `CleanPolicy::strict()` — measures the steady-state cleaning path.
  Each case asserts ≥ 200 MiB/s on the existing benchmark host (using `criterion`'s `throughput`).
- [ ] T028 Run `cargo bench --bench clean_throughput` and capture the numbers in the WP commit message.

---

## Phase 8: Fuzz Coverage

**Purpose**: Robustness — fuzz the new entry point against random byte / codepoint streams.

- [ ] T029 Extend `fuzz/fuzz_targets/` with a new `clean_target.rs` that calls `clean(arbitrary_str, &CleanPolicy::strict())`. Run `cargo fuzz run clean_target -- -max_total_time=120` locally and confirm no panics. Add the target to the CI fuzz manifest.

---

## Phase 9: Polish

**Purpose**: Documentation, CHANGELOG, README updates.

- [ ] T030 [P] Add a `## Cleaning` section to `README.md` with the library + CLI examples from `quickstart.md`.
- [ ] T031 [P] Add a CHANGELOG entry under the next-version heading naming the new public items (`clean`, `CleanPolicy`, `CleanResult`, `CleanAction`).
- [ ] T032 [P] Run `cargo doc --no-deps -p unicleaner` and confirm every new public item carries a doc comment per Constitution VII.
- [ ] T033 Final `just check` (or equivalent: `cargo test && cargo clippy -- -D warnings && cargo fmt --check && cargo bench --no-run`) — all green.

---

## Dependencies

- **Setup (Phase 1)** must complete before anything else.
- **Foundational (Phase 2)** must complete before Phases 3 / 5.
- **US1 (Phase 3)** is required by US1.5 (Phase 4) and US2 (Phase 5) — they call `clean()`.
- **US3 (Phase 6)** depends on Phase 2 (uses `CleanPolicy`) and Phase 5 (uses CLI dispatch).
- **Bench (Phase 7)** can start in parallel with Phase 5 once Phase 3 is done.
- **Fuzz (Phase 8)** depends on Phase 3 only.
- **Polish (Phase 9)** depends on every preceding phase.

### TDD Workflow Within Each Phase

1. Write tests → verify they compile but fail.
2. Implement minimal code to make tests pass.
3. Run `cargo clippy` and `cargo fmt --check`.
4. Proceed to next phase.

### Parallel Opportunities

- T002 and T003 can run in parallel (different files).
- T004, T009, T010, T011, T016, T018, T019 (test-writing) can run in parallel — they touch independent test files.
- US3 (Phase 6) can begin as soon as Phase 2 and Phase 5 complete; T023 / T024 can run in parallel with each other.
- Phase 7 (bench) and Phase 8 (fuzz) can run in parallel with Phase 5 once Phase 3 is done.
- Within Polish, T030 / T031 / T032 can run in parallel.

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 (Setup): T001–T003.
2. Phase 2 (CleanPolicy): T004–T008.
3. Phase 3 (`clean()`): T009–T015.
4. **STOP and VALIDATE**: invoke `clean()` from a downstream consumer (e.g. `hive-tainted-string`'s `UnicodeSanitizer`); confirm strict / lossy / report-only round-trip the expected behaviour against fixture inputs.

### Incremental Delivery

1. Setup + Foundational + US1 → library API works → MVP ready.
2. Add US1.5 → NFC available for prose sinks.
3. Add US2 → CLI subcommand → operator-facing fix workflow.
4. Add US3 → per-category tuning + TOML config integration → power-user knob.
5. Bench + fuzz → robustness verified.
6. Polish → docs, CHANGELOG, ship.

---

## Notes

- **Single source of truth**: `pattern_for()` is consulted by both `decide_action` (new, in cleaner) and `detect_in_string_with_policy` (existing). Future malicious-codepoint changes propagate to both without code duplication. This is the structural reason the cleaner is built on top of the detector and not as a parallel walk.
- **Order matters**: strip / replace runs **first**, NFC second. See `research.md` Decision 3 and the dedicated test in T016.
- **`Cow::Borrowed` discipline**: T009 case 4 and T011 I-2 are the regression guards. Don't let "small refactor" tempt you into always allocating.
- **The `<inline>` path placeholder** for `Violation::file_path` mirrors the existing convention in `detect_in_string` — no schema change.
- **AGENTS.md "Config Sync Rules"** mandates the `examples/unicleaner.toml` edit (T026); skipping it breaks the example-config integration test in `tests/integration/config_tests.rs`.
- **Constitution III (TDD-Non-Negotiable)** applies: every task pair `(test, impl)` must run tests in failing state before the impl is written. CI does not enforce this directly; reviewers do.
