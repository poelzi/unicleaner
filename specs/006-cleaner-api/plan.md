# Implementation Plan: Cleaner API

**Branch**: `006-cleaner-api` | **Date**: 2026-05-07 | **Spec**: `specs/006-cleaner-api/spec.md`
**Input**: Feature specification from `/specs/006-cleaner-api/spec.md`

## Summary

Add a `clean()` library entry point alongside the existing detection-only scanner. The function walks the input once, consults the existing `pattern_for(code_point)` malicious-codepoint table, and applies a per-category `CleanAction` (strip / replace / keep-with-mark) from a caller-supplied `CleanPolicy`. Returns a `CleanResult { output: Cow<str>, violations: Vec<Violation>, modified: bool }` so the no-op fast path costs zero allocations. Optional NFC normalization runs after stripping. A new `unicleaner clean` CLI subcommand exposes the same machinery — stdout by default, atomic `--in-place` opt-in.

The change is **purely additive**: every existing detector path (`detect_in_string`, `scan_file`, `unicleaner scan`) and every existing `Configuration` field works unchanged. New code lives under `src/cleaner/`; the only edits to existing files are:

1. `src/lib.rs` — add `pub mod cleaner;` and re-export `clean`, `CleanPolicy`, `CleanResult`, `CleanAction`.
2. `Cargo.toml` — add `unicode-normalization = "0.1"` dependency.
3. `src/cli/` — add the `clean` subcommand alongside the existing `scan`.
4. `examples/unicleaner.toml` — add the optional `[cleaner]` config block (per AGENTS.md "Config Sync Rules").

## Technical Context

**Language/Version**: Rust 1.85+ (MSRV, edition 2024)
**Primary Dependencies**: existing (`clap`, `serde`, `toml`, `unicode-blocks`, `encoding_rs`, `rayon`, `ignore`, `globset`, `owo-colors`) + **new**: `unicode-normalization = "0.1"`.
**Storage**: TOML configuration files (existing); no new on-disk format.
**Testing**: `cargo test` (unit + integration), `cargo bench` (existing harness for throughput regression check).
**Target Platform**: Linux, macOS, Windows (cross-platform CLI).
**Project Type**: Single Rust crate (lib + bin).
**Performance Goals**: `clean()` strict-mode throughput ≥ 200 MiB/s on the existing benchmark host (within 30 % of `detect_in_string_with_policy`). The detection walk dominates; the cleaner adds bounded work per matched codepoint only.
**Constraints**:

- Zero-allocation fast path when there are no violations and NFC is off.
- No new transitive deps beyond `unicode-normalization`.
- The existing `pattern_for()` table is the single source of truth — no duplication.
- Backward-compatible: every existing detector test stays green without modification.

**Scale/Scope**: ~250 LOC of new code across `cleaner/{mod,policy}.rs` and CLI handler, plus tests / docs / golden fixtures.

## Constitution Compliance

Per `.specify/memory/constitution.md` principles:

- **I. Rust-First**: Pure Rust. Re-uses `Result` / `Cow` / iterator combinators idiomatically.
- **II. CLI Interface**: New `clean` subcommand follows the established `scan` shape — text input via paths or stdin, stdout for output, stderr for diagnostics, conventional exit codes.
- **III. Test-First (NON-NEGOTIABLE)**: Every WP starts with failing tests; implementation lands only after the tests are reviewed and verified red.
- **IV. Comprehensive Testing**: Unit tests for `clean()` and `CleanPolicy`, integration tests against existing detection fixtures (golden output), benchmark to prove the perf budget, fuzz harness extension to cover the new entry point.
- **VI. Nix Integration**: No build-system change required — the new dep flows through the existing Cargo → `naersk` pipeline.
- **VII. Code Quality**: `cargo clippy` and `cargo fmt --check` gates run after each phase. The new module respects the existing `src/<area>/{mod,…}.rs` layout convention.

## Project Structure

### Documentation (this feature)

```text
specs/006-cleaner-api/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: dep choice, return type, ordering
├── data-model.md        # Phase 1: CleanPolicy / CleanAction / CleanResult
├── quickstart.md        # Phase 1: usage guide
├── checklists/
│   └── requirements.md  # Quality checklist
└── tasks.md             # Phase 2: WPs
```

### Source Code (repository root)

```text
src/
├── cleaner/
│   ├── mod.rs           # NEW — pub fn clean(...) + CleanResult struct
│   └── policy.rs        # NEW — CleanPolicy, CleanAction, presets, mutators
├── cli/
│   ├── mod.rs           # MODIFIED — add `Clean { ... }` subcommand variant
│   └── clean.rs         # NEW — handler for the `clean` subcommand
├── lib.rs               # MODIFIED — pub mod cleaner; re-exports
└── main.rs              # MODIFIED — dispatch `Clean` to the new handler

tests/
├── integration/
│   ├── cleaner_tests.rs # NEW — library API end-to-end tests
│   └── cli_clean_tests.rs # NEW — CLI subcommand integration tests
└── fixtures/
    └── cleaner/         # NEW — golden inputs + cleaned outputs

benches/
└── clean_throughput.rs  # NEW — perf regression guard against detection baseline

examples/
└── unicleaner.toml      # MODIFIED — add optional [cleaner] block
```

### Why this layout

- `src/cleaner/` matches the established `src/<area>/` convention (`scanner/`, `report/`, `unicode/`).
- The CLI handler lives in `src/cli/clean.rs` symmetrically to the existing scan handler — no central handler / dispatch refactor needed.
- Integration tests sit in `tests/integration/` next to the existing `cli_tests.rs` / `config_tests.rs`. This keeps fuzz fodder in `fuzz/` and CLI behaviour next to the rest of the CLI.
- The `examples/unicleaner.toml` edit is required by the `test_example_config_loads_successfully` integration test (per AGENTS.md "Config Sync Rules").

## Architecture

### `clean()` hot loop

```rust
pub fn clean<'a>(input: &'a str, policy: &CleanPolicy) -> CleanResult<'a> {
    // Fast pre-scan: if no codepoint matches and NFC is off, return Borrowed.
    if !policy.normalize_nfc && !needs_mutation(input, policy) {
        return CleanResult { output: Cow::Borrowed(input), violations: Vec::new(), modified: false };
    }

    let mut output = String::with_capacity(input.len());
    let mut violations = Vec::new();

    for (line_num, line) in input.split_inclusive('\n').enumerate() {
        let mut col = 1usize;
        for (byte_off, ch) in line.char_indices() {
            let cp = ch as u32;
            let action = decide_action(cp, policy);
            match action {
                Some((cat, pat, CleanAction::Strip)) => {
                    violations.push(Violation::new(/* … */));
                }
                Some((cat, pat, CleanAction::Replace(rep))) => {
                    violations.push(Violation::new(/* … */));
                    output.push(rep);
                }
                Some((cat, pat, CleanAction::KeepWithMark)) => {
                    violations.push(Violation::new(/* … */));
                    output.push(ch);
                }
                None => output.push(ch),
            }
            col += 1;
        }
    }

    let modified_pre_nfc = output.as_str() != input;
    let final_output = if policy.normalize_nfc && !is_nfc(&output) {
        Cow::Owned(output.nfc().collect())
    } else {
        Cow::Owned(output)
    };

    CleanResult {
        output: final_output,
        violations,
        modified: modified_pre_nfc || policy.normalize_nfc && /* nfc changed bytes */,
    }
}
```

The `needs_mutation` pre-scan is cheap (single linear walk consulting `pattern_for`) and avoids the allocation altogether for the common clean-input case. `decide_action` consults `policy.per_category` then `policy.default_action`; on a miss in the malicious table it also checks `denied_code_points` and the `deny_by_default` rule, mirroring `detect_in_string_with_policy` line-for-line.

### `CleanPolicy` resolution

```rust
fn effective_action(policy: &CleanPolicy, category: MaliciousCategory) -> CleanAction {
    policy.per_category.get(&category).copied().unwrap_or(policy.default_action)
}
```

Crate-private. The hot loop calls it once per matched codepoint, so the cost is one B-tree probe and a copy.

### CLI flow

`unicleaner clean PATH...` reads each file, runs `clean()` against the resolved policy, then either:

- writes the cleaned content to stdout (default, one path → byte-stream output; multiple paths is an error, mirrors `unicleaner scan`'s single-file mode), or
- writes to `<PATH>.tmp`, fsyncs, and `rename()`s onto the original path when `--in-place` is set.

`Stdin` mode (`unicleaner clean -`) is supported for parity with `scan`. The output goes to stdout regardless.

### Backward compatibility

- The detector's public API (`detect_in_string`, `detect_in_string_with_policy`) is untouched.
- `Configuration` (the existing TOML root) is untouched. The CLI's new `[cleaner]` block is **optional** — files without it work as before.
- `Severity` / `MaliciousCategory` / `MaliciousPattern` / `Violation` are reused as-is.

## Risks & Mitigations

| Risk | Mitigation |
| --- | --- |
| `unicode-normalization` introduces transitive deps. | Verified in research.md: zero transitive deps beyond `tinyvec` (already in tree via `clap`'s deps). Confirmed by `cargo tree -p unicode-normalization` in WP01. |
| Slow path allocations regress hot consumers. | Fast-path pre-scan (`needs_mutation`) verified by benchmark in WP06; CI gate on the throughput target. |
| NFC ordering bug (NFC before strip) reintroduced by future refactor. | Order is asserted by a dedicated test (`tests/integration/cleaner_tests.rs::nfc_runs_after_strip`) using a fixture where the order is observable. |
| In-place writer corrupts on crash. | Atomic write pattern (`.tmp` → `fsync` → `rename`) is reused from `src/scanner/file_scanner.rs`; integration test simulates a kill-during-write via a wrapper fixture. |
| Drift between detector and cleaner walks. | The `decide_action` helper is the **only** new place that consults `pattern_for`; `detect_in_string_with_policy` is refactored in WP02 to call the same helper, so future codepoint changes propagate to both. |

## Phases

| Phase | Output |
| --- | --- |
| **0 — Research** | `research.md` (this feature dir) — done. |
| **1 — Design** | `data-model.md`, `quickstart.md` (this feature dir) — done. |
| **2 — Tasks** | `tasks.md` (next; WP-level breakdown). |
| **3 — Implementation** | One WP per task group; TDD inside each. |
| **4 — Polish** | Doc / README / CHANGELOG; `cargo bench` results captured; `unicleaner clean --help` reviewed. |

## Open Questions

None. All clarifications recorded in `spec.md::Clarifications` and `research.md`.

## References

- `src/scanner/unicode_detector.rs:9` — existing per-character walk that the cleaner mirrors.
- `src/unicode/malicious.rs:65` — `pattern_for(code_point) -> Option<&'static MaliciousPattern>`, the shared codepoint table.
- `src/report/violation.rs` — `Violation::new(...)` constructor reused unchanged.
- `src/scanner/file_scanner.rs` — atomic write pattern referenced by the `--in-place` CLI mode.
- `examples/unicleaner.toml` — must stay loadable; add the `[cleaner]` block here when the config support lands.
- `AGENTS.md` "Config Sync Rules" section — gates the example-config edit.
