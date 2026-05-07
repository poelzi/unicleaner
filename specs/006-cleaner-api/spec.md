# Feature Specification: Cleaner API

**Feature Branch**: `006-cleaner-api`
**Created**: 2026-05-07
**Status**: Draft
**Input**: User description: "unicleaner currently only DETECTS malicious unicode and returns Vec<Violation>. Add a `clean()` API that consumes the detection result to produce a sanitized string (strip / replace / NFC-normalize), so downstream Rust code can use the crate as both a validator and a sanitizer. Expose it as a library entry point and as a `unicleaner clean` CLI subcommand."

## Clarifications

### Session 2026-05-07

- Q: Should the cleaner's per-category policy default to the same severity used by the detector (Error → strip, Warning → replace, Info → keep), or expose a stand-alone defaults set? → A: Stand-alone defaults — `CleanPolicy::strict()` strips everything malicious, `CleanPolicy::lossy()` replaces everything with `U+FFFD`, `CleanPolicy::report_only()` makes no changes (parity with current detect-only). The detector's `Severity` axis is orthogonal to the cleaner's `CleanAction` axis and conflating them couples the two configuration surfaces in ways that block future tuning.
- Q: Does the CLI `clean` subcommand mutate files in place by default, or write to stdout? → A: Stdout by default. `--in-place` is opt-in. This matches the safety convention used by `sed`/`fmt`/`rustfmt` and avoids data-loss surprises when users pipe the command into a wider workflow.
- Q: When the input contains zero violations, does `clean()` return `Cow::Borrowed` (no allocation) or `Cow::Owned` (always materialize)? → A: `Cow::Borrowed` whenever no mutation occurred and NFC is disabled or the input was already NFC. Allocations cost real money in hot loops (the hive-tainted-string consumer will call this on every user-supplied string in a daemon); the SDK should not allocate when there is nothing to do.
- Q: Should NFC normalization apply unconditionally, or only when `policy.normalize_nfc = true`? → A: Opt-in. NFC mutates code points (e.g. `é` represented as `e` + combining acute → single `é`), which is correct for prose / chat input but **incorrect** for source code or JSON keys whose canonical form callers may rely on. Default off; explicit opt-in for prose / chat sinks.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Sanitize a String In Process (Priority: P1)

A library author building a downstream Rust crate (e.g. the `hive-tainted-string` provenance wrapper) wants to call `unicleaner::clean(input, &policy)` from inside a filter, receive a `Cow<str>` they can hand off to a sink, and stop reimplementing zero-width / bidi-override stripping locally.

**Why this priority**: This is the core value proposition. Today every downstream consumer that wants sanitization has to either run unicleaner as a CLI (heavy, file-based, wrong shape for in-process use) or duplicate the malicious-codepoint table. A library entry point unblocks integration as a building block, not just a tool.

**Independent Test**: Can be fully tested by calling `clean()` on a fixture string with a known zero-width sequence, asserting the output is shorter and the returned `violations` vec is non-empty.

**Acceptance Scenarios**:

1. **Given** a string `"hi\u{200B}there"` and `CleanPolicy::strict()`, **When** `clean()` is called, **Then** the returned `output` is `"hithere"`, `violations` contains one entry for U+200B, and `modified == true`.
2. **Given** a string `"plain ascii"` and `CleanPolicy::strict()`, **When** `clean()` is called, **Then** `output` is `Cow::Borrowed` over the input, `violations` is empty, and `modified == false`.
3. **Given** a string with a bidi override (`"admin\u{202E}lortnoc"`) and `CleanPolicy::lossy()`, **When** `clean()` is called, **Then** the override is replaced with `U+FFFD` and the rest of the string is preserved verbatim.
4. **Given** a string with no malicious codepoints but in NFD form (`"e\u{0301}"`) and `CleanPolicy::strict().with_nfc(true)`, **When** `clean()` is called, **Then** `output` is `"é"` (single NFC code point) and `modified == true`.

---

### User Story 2 — Clean a File from the CLI (Priority: P1)

An operator with a checked-in source file flagged by `unicleaner scan` wants a single command that produces a cleaned copy. They run `unicleaner clean src/foo.rs > src/foo.clean.rs`, eyeball the diff, then optionally `--in-place` for the actual fix.

**Why this priority**: The detection-only CLI tells operators *what* is wrong but leaves them to fix it by hand — error-prone for files with multiple violations or non-printable codepoints they can't even type. A `clean` subcommand turns "you have problems" into "here, take the fix."

**Independent Test**: Run `unicleaner clean fixtures/with-zwsp.txt`, capture stdout, byte-compare against the matching `*.cleaned.txt` golden.

**Acceptance Scenarios**:

1. **Given** `unicleaner clean fixtures/zwsp.txt` (file containing zero-width spaces), **When** the command runs, **Then** stdout contains the file's content with the zero-width spaces removed and the exit code is `0`.
2. **Given** `unicleaner clean --in-place fixtures/zwsp.txt`, **When** the command runs, **Then** the file on disk is replaced atomically (write to `.tmp`, fsync, rename) with the cleaned content; stdout is empty.
3. **Given** `unicleaner clean fixtures/clean.rs` (file with no violations), **When** the command runs, **Then** stdout is byte-identical to the input and exit code `0`.
4. **Given** `unicleaner clean nonexistent.rs`, **When** the command runs, **Then** stderr contains a clear "no such file" error and exit code is non-zero (matching the existing `scan` subcommand convention).

---

### User Story 3 — Per-Category Policy Tuning (Priority: P2)

A team wants a custom policy: strip zero-width chars (silent removal is fine for prose), but **replace** bidi overrides with a visible `[BIDI]` marker so reviewers see the intent, and **keep** homoglyphs (their team genuinely uses Cyrillic identifiers and the false-positive cost is real).

**Why this priority**: Different categories carry different operational cost — one-size-fits-all is what made the detection-only design awkward. Per-category override is the natural shape of the configuration once we accept that "clean" is policy-driven.

**Independent Test**: Build a `CleanPolicy::strict()`, call `.with_action(MaliciousCategory::Homoglyph, CleanAction::Keep)`, run `clean()` on a homoglyph-bearing fixture, assert the output equals the input and `violations` still lists the homoglyph (so callers can log it).

**Acceptance Scenarios**:

1. **Given** a policy with `default = Strip`, `bidi = Replace('?')`, `homoglyph = Keep`, **When** `clean()` is called on a string containing all three, **Then** the zero-width is gone, the bidi override is replaced with `?`, and the homoglyph is preserved.
2. **Given** any policy and an input free of malicious codepoints, **When** `clean()` is called, **Then** `output` borrows from the input (no allocation) regardless of policy.

---

### User Story 4 — Round-Trip via Existing Config File (Priority: P3)

An operator who already has an `unicleaner.toml` configured for `scan` wants the same allowlist / denylist to drive `clean` without restating it.

**Why this priority**: Convenient but not load-bearing. A single inline `--strict|--lossy` flag covers most use cases; full config parity with `scan` is a follow-up that only matters once the cleaner is widely deployed.

**Independent Test**: `unicleaner clean --config unicleaner.toml fixtures/foo.txt` honours the config's `denied_code_points` and `allowed_ranges`; verified by comparing against the same fixture run with the inline equivalent.

**Acceptance Scenarios**:

1. **Given** an `unicleaner.toml` with `denied_code_points = [0x2028]` and a fixture containing `U+2028`, **When** `unicleaner clean --config unicleaner.toml` is run, **Then** the line separator is stripped per `CleanPolicy::strict()`'s default action.

---

### Edge Cases

- **Empty input** → returns `Cow::Borrowed("")`, `violations` empty, `modified = false`.
- **Input is malicious-only** (all codepoints stripped) → returns `Cow::Owned("")` with all violations listed.
- **NFC + violations interaction** — NFC normalization runs **after** stripping/replacement so a stripped surrogate-pair half can't poison the normalizer. The order is documented and tested.
- **Multi-byte replacement char** — when `Replace(c)` is configured with `c` longer than the original codepoint's UTF-8 representation, the output's byte length increases. Not an error.
- **Surrogate codepoints** — Rust `str` cannot contain unpaired surrogates, so the `clean()` API rejects this case at the type system level. Not a runtime concern.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose `pub fn clean<'a>(input: &'a str, policy: &CleanPolicy) -> CleanResult<'a>` from the crate root.
- **FR-002**: System MUST return `CleanResult { output: Cow<'a, str>, violations: Vec<Violation>, modified: bool }`.
- **FR-003**: When no codepoint matches `policy` and NFC is disabled, `output` MUST be `Cow::Borrowed` over `input` (zero-allocation fast path).
- **FR-004**: System MUST provide three named constructors on `CleanPolicy`: `strict()`, `lossy()`, `report_only()`. Each MUST be documented with an exact behavioural description.
- **FR-005**: `CleanPolicy` MUST allow per-category override via `with_action(MaliciousCategory, CleanAction)`.
- **FR-006**: `CleanAction` MUST have variants `Strip`, `Replace(char)`, `KeepWithMark` (records a violation but keeps the codepoint).
- **FR-007**: `CleanPolicy` MUST allow toggling NFC normalization via `with_nfc(bool)`. Default off.
- **FR-008**: System MUST add a CLI subcommand `unicleaner clean [PATH]...` that writes cleaned content to stdout by default.
- **FR-009**: CLI MUST support `--in-place` (atomic rename) and `--policy {strict|lossy|report-only}` flags.
- **FR-010**: CLI MUST support `--config <PATH>` to load policy / allow-list / deny-list from an existing `unicleaner.toml`.
- **FR-011**: System MUST reuse the existing `pattern_for(code_point)` table — no duplicated codepoint catalog.
- **FR-012**: `clean()` MUST run the same per-character walk as `detect_in_string_with_policy`, sharing the iteration shape so future detector changes propagate to the cleaner without code duplication.
- **FR-013**: System MUST emit `Violation` entries for every codepoint the policy acted on, regardless of whether the action was `Strip`, `Replace`, or `KeepWithMark`.
- **FR-014**: When NFC normalization changes the output, `modified` MUST be `true` even if no violations were recorded.

### Key Entities

- **CleanPolicy** — operator configuration: default action + per-category overrides + NFC toggle + denied / allowed sets.
- **CleanAction** — what to do when a malicious codepoint is encountered: strip, replace, or keep-with-mark.
- **CleanResult** — return type bundling the (possibly mutated) `Cow<str>`, the `Vec<Violation>` for telemetry, and a `modified` flag.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A downstream Rust crate (e.g. `hive-tainted-string`) can switch from a hand-rolled detect-and-reject stub to `unicleaner::clean` with a single call site change — the runtime behaviour for malicious inputs (rejection / sanitization) is observable through the existing `Violation` shape with no schema additions.
- **SC-002**: `unicleaner clean fixtures/zwsp.txt` produces byte-identical output to the matching golden file across 100 % of the existing detection fixtures, demonstrating that the per-character walk is faithfully shared between detector and cleaner.
- **SC-003**: `clean()` with the strict default and a clean ASCII input runs at ≥ 200 MiB/s on the existing benchmark host (within 30 % of `detect_in_string_with_policy`'s throughput, since clean is a strict superset of detect's work).
- **SC-004**: 100 % of the new code is exercised by tests; coverage gates remain green.
- **SC-005**: No new dependency outside `unicode-normalization` is added; the alpha CLI's footprint (clap / rayon / ignore / globset / encoding_rs / owo-colors) is unchanged.
