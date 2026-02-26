# Remediation Evidence: WP-01..WP-11

## Status Overview

- Phase A: complete
- Phase B: complete
- Phase C: complete
- Phase D: complete

## Finding-to-Evidence Mapping

| WP ID | Finding | Evidence |
|------|---------|----------|
| WP-01 | 004 plan file was template content | `specs/004-fix-review-findings/plan.md` now contains concrete technical context, constitution gates, and verification gates. |
| WP-02 | Missing explicit Test-First user approval gate | `specs/004-fix-review-findings/tasks.md` includes constitution execution gates and per-story approval checkpoints; `specs/004-fix-review-findings/quickstart.md` includes explicit approval step. |
| WP-03 | Missing mandatory fuzz gate | `specs/004-fix-review-findings/tasks.md` adds `T092`/`T093`; `specs/004-fix-review-findings/quickstart.md` and `specs/004-fix-review-findings/plan.md` include fuzz smoke commands; executed successfully via `just fuzz ...` recipes (see Command Evidence). |
| WP-04 | Unreadable config file path not covered | Added `test_config_unreadable_file_error` in `tests/integration/config_tests.rs`. |
| WP-05 | Invalid `--encoding` value not covered | Added `test_encoding_override_invalid_value_errors` in `tests/integration/encoding_tests.rs`. |
| WP-06 | `EncodingError` classification unproven | Added `test_encoding_error_classification_on_decode_failure` in `tests/integration/encoding_tests.rs`. |
| WP-07 | Workflow contract test not validating field mapping deeply enough | Strengthened `test_pr_check_workflow_uses_valid_flags` in `tests/integration/output_tests.rs` to assert required jq paths and validate them against actual scanner JSON output keys. |
| WP-08 | `--jobs 0` behavior ambiguous | `specs/004-fix-review-findings/spec.md` now explicitly defines reject-on-zero behavior in acceptance, edge cases, and FR-006. |
| WP-09 | Terminology drift (`denied_characters` vs `denied_code_points`) | Normalized user-facing wording in `specs/004-fix-review-findings/tasks.md` and `specs/004-fix-review-findings/quickstart.md` to `denied_characters` with internal mapping note. |
| WP-10 | Binary heuristic requirement drift | `specs/004-fix-review-findings/spec.md` now aligns with implemented heuristics (consecutive-null + control-byte ratio) and removes extension-skip expectation. |
| WP-11 | Review artifact naming drift (`review.md` vs `REVIEW.md`) | Git index normalized to canonical `REVIEW.md`; lowercase `review.md` removed from index. |

## Command Evidence

Successful verification gates:

- `CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo test --all-features --workspace` (pass)
- `CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo clippy --all-targets --all-features -- -D warnings` (pass)
- `cargo fmt --check` (pass; rustfmt emitted nightly-option warnings from config, no formatting failures)
- `CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo bench --no-run` (pass)

Fuzz smoke gates (via `just`, per branch instruction):

- `CARGO_TARGET_DIR=/tmp/unicleaner-fuzz-target just fuzz fuzz-parallel-scanner 30` (pass; 379 runs, no crash)
- `CARGO_TARGET_DIR=/tmp/unicleaner-fuzz-target just fuzz fuzz-walker 30` (pass; completed with expected path-access warnings while mutating arbitrary paths)
