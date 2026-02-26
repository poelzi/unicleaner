# Work Package Plan: Branch Remediation Closure

**WP ID**: `005-branch-remediation`  
**Execution Branch**: `004-fix-review-findings` (no new feature branch)  
**Date**: 2026-02-26  
**Primary Inputs**: `REVIEW.md`, `specs/004-fix-review-findings/spec.md`, `specs/004-fix-review-findings/tasks.md`, `.specify/memory/constitution.md`

## Goal

Close the remaining branch-level gaps before merge: constitution compliance gaps, missing regression coverage for edge/error cases, and planning/workflow drift. This WP is intentionally small and focused on closure, not new feature scope.

## Scope

### In Scope

- Restore planning artifacts that drifted from intended state.
- Add missing tests for uncovered acceptance requirements.
- Align workflow/schema/test contracts so CI behavior is deterministic.
- Resolve ambiguous requirement behavior that can cause inconsistent implementation.
- Run final verification gates from a clean build target.

### Out of Scope

- New scanner features unrelated to review findings.
- Large refactors across unrelated modules.
- New CI products or distribution channels.

## Findings to Remediate

| ID | Finding | Severity | Concrete Fix |
|----|---------|----------|--------------|
| WP-01 | `specs/004-fix-review-findings/plan.md` is template content, not an actual implementation plan | HIGH | Reconstruct plan from current 004 spec/tasks and current code state |
| WP-02 | Constitution Test-First flow lacks explicit user approval checkpoint | CRITICAL | Add explicit gate in 004 plan/tasks/quickstart: write test -> request approval -> verify fail -> implement |
| WP-03 | Mandatory fuzz testing gate missing from 004 plan/tasks | CRITICAL | Add fuzz tasks and final fuzz gate for affected scanner paths |
| WP-04 | FR-003 unreadable config-file path not covered by dedicated test | HIGH | Add integration test for unreadable config + expected error classification/message |
| WP-05 | Invalid `--encoding` value behavior lacks dedicated test | HIGH | Add CLI test for invalid encoding value and deterministic failure output |
| WP-06 | `EncodingError` classification path not explicitly regression-tested | HIGH | Add test forcing decode failure and assert `ErrorType::EncodingError` |
| WP-07 | PR workflow contract test focuses mostly on flag drift, not full output field mapping | HIGH | Strengthen output/workflow test to assert jq field paths match actual JSON schema fields |
| WP-08 | `--jobs 0` behavior remains ambiguous in spec | MEDIUM | Pick one behavior and codify in spec + tests (recommended: reject with clear error) |
| WP-09 | Terminology drift (`denied_characters` vs `denied_code_points`) across spec/tasks | MEDIUM | Normalize vocabulary in 004 docs and matching tests |
| WP-10 | Binary detection requirement mentions extension heuristic while tasks emphasize control-byte ratio | MEDIUM | Either implement extension heuristic with tests or narrow requirement text to actual behavior |
| WP-11 | Review artifact naming/casing is inconsistent in git state (`review.md` vs `REVIEW.md`) | MEDIUM | Normalize to single canonical `REVIEW.md` and clean index state |

## Execution Phases

### Phase A - Artifact and Constitution Repair (blocking)

1. Restore `specs/004-fix-review-findings/plan.md` to concrete, non-template content.
2. Add explicit Test-First user-approval checkpoints in 004 docs.
3. Add fuzzing requirements/tasks to 004 docs and final validation section.

**Exit Criteria**: No template placeholders in 004 plan; constitution gates are explicit and test-first flow is actionable.

### Phase B - Missing Test Coverage (behavioral)

1. Add unreadable config test.
2. Add invalid encoding value test.
3. Add EncodingError classification test.
4. Strengthen workflow field-mapping contract test.

**Exit Criteria**: Each missing high-severity coverage gap has at least one dedicated failing-then-passing test.

### Phase C - Spec/Task Consistency Cleanup

1. Decide and document `--jobs 0` behavior, then enforce by tests.
2. Normalize terminology drift (`denied_characters` / `denied_code_points`).
3. Align binary heuristic requirement and implementation/tasks.

**Exit Criteria**: No unresolved ambiguities in 004 spec/tasks for implemented behaviors.

### Phase D - Merge Hygiene and Final Verification

1. Normalize review artifact casing and index state.
2. Run final gates from clean target dir.

Suggested verification commands:

```bash
CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo test --all-features --workspace
CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo clippy --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo fmt --check
CARGO_TARGET_DIR=/tmp/unicleaner-remediation-target cargo bench --no-run
```

Fuzz smoke gate (affected targets):

```bash
just fuzz fuzz-parallel-scanner 30
just fuzz fuzz-walker 30
```

**Exit Criteria**: All quality gates pass and branch is ready for merge decision.

## Risks and Mitigations

- Risk: Coverage additions expose latent behavior defects.  
  Mitigation: Keep each test isolated; fix in smallest possible diffs.
- Risk: Local target cache corruption causes false negatives.  
  Mitigation: use dedicated clean `CARGO_TARGET_DIR` for WP verification.
- Risk: Documentation-only edits diverge from implementation.  
  Mitigation: every doc change tied to a test or command validation check.

## Deliverables

- Updated 004 planning artifacts (`plan.md`, `tasks.md`, optional `quickstart.md` adjustments).
- New/updated regression tests in integration/unit modules for uncovered requirements.
- Verified CI workflow field/flag contract tests.
- Final verification log summary in PR description.
