# Tasks: Branch Remediation Closure

**WP**: `005-branch-remediation`  
**Execution Branch**: `004-fix-review-findings`  
**Rule**: For each test addition, follow constitution flow: write test -> get user approval -> verify failing -> implement -> verify passing.

## Phase A - Artifact and Constitution Repair

- [X] W001 Restore `specs/004-fix-review-findings/plan.md` from template to concrete implementation plan content.
- [X] W002 Update `specs/004-fix-review-findings/tasks.md` to include explicit user-approval checkpoints before implementation tasks.
- [X] W003 Update `specs/004-fix-review-findings/tasks.md` polish gates to include mandatory fuzz validation tasks.
- [X] W004 Update `specs/004-fix-review-findings/quickstart.md` with explicit test-first approval flow and fuzz gate commands.

## Phase B - Missing Behavioral Coverage

- [X] W005 Add integration test in `tests/integration/config_tests.rs` for unreadable config file path (`--config` points to unreadable file).
- [X] W006 Add integration test in `tests/integration/encoding_tests.rs` for invalid encoding value (clear non-zero exit and message).
- [X] W007 Add regression test in `tests/integration/encoding_tests.rs` or `src/scanner/parallel.rs` for `EncodingError` classification on decode failure.
- [X] W008 Strengthen workflow contract test in `tests/integration/output_tests.rs` to validate jq field usage against produced JSON fields.

## Phase C - Ambiguity and Drift Cleanup

- [X] W009 Decide `--jobs 0` behavior (recommended: reject with error) and update `specs/004-fix-review-findings/spec.md` acceptance/requirements accordingly.
- [X] W010 Add/adjust tests for chosen `--jobs 0` behavior in `src/scanner/parallel.rs` or integration tests.
- [X] W011 Normalize terminology across docs/tests (`denied_characters` vs `denied_code_points`) in `specs/004-fix-review-findings/spec.md` and `specs/004-fix-review-findings/tasks.md`.
- [X] W012 Align binary-detection requirement in `specs/004-fix-review-findings/spec.md` with implementation in `src/scanner/encoding.rs` (either add extension heuristic + tests or narrow requirement wording).

## Phase D - Merge Hygiene and Final Validation

- [X] W013 Normalize review artifact naming and git index state to keep only canonical `REVIEW.md`.
- [X] W014 Run full test gate with clean target dir: `CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo test --all-features --workspace`.
- [X] W015 Run lint/format gates with clean target dir: `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`.
- [X] W016 Run bench compile gate: `cargo bench --no-run`.
- [X] W017 Run fuzz smoke gate for affected targets (`fuzz_parallel_scanner`, `fuzz_walker`) via `just` recipes.
- [X] W018 Produce final remediation checklist in PR notes mapping WP-01..WP-11 to evidence (tests/commands/files). (Drafted in `specs/005-branch-remediation/evidence.md`)

## Dependency Notes

- W001-W004 are blocking for process compliance and should be completed first.
- W005-W008 can run in parallel after Phase A.
- W009-W012 should be completed before final verification because they affect expected behavior.
- W013-W018 are finalization and should be run last.
