# Research: Cleaner API

## Decision 1: Normalization library

**Decision**: Use the `unicode-normalization` crate (version 0.1.x).

**Rationale**:

- Pure Rust, no transitive deps that conflict with the existing crate graph.
- Single-purpose: implements the four Unicode normal forms (NFC / NFKC / NFD / NFKD). We only need NFC.
- API matches the pattern we already use elsewhere in the crate: `s.nfc().collect::<String>()` and `is_nfc(s) -> bool` for the fast-path check.
- ~100 KiB compiled, no run-time configuration cost.
- Stable: 1.0 candidate has been the de-facto Rust normalization crate for years; downstream consumers (e.g. `idna`, `unicode-segmentation`) already pull it in.
- MSRV-friendly: works on 1.85.

**Alternatives considered**:

- `icu_normalizer` (part of `icu4x`): heavier, brings the ICU data system, more deps. Overkill for a CLI-side single-call use.
- Hand-rolling NFC: out of scope. Decomposition / canonical-equivalence tables alone are tens of thousands of entries; Unicode revisions would have to be tracked manually.
- Skipping NFC entirely: tempting (the malicious-codepoint table doesn't need normalization to work), but the `hive-tainted-string` consumer is going to want NFC for free-form prose / chat sinks. Better to land it now behind an opt-in flag than retrofit later.

## Decision 2: Return type — `Cow<'_, str>` vs `String`

**Decision**: `CleanResult<'a>::output: Cow<'a, str>`.

**Rationale**:

- The common case in production (downstream `hive-tainted-string` filter) is **clean inputs**: tens of thousands of strings per minute, 99 %+ of them un-touched. Allocating a fresh `String` for every call burns measurable memory bandwidth in hot paths.
- `Cow::Borrowed(input)` when no mutation occurs and NFC is off (or the input was already in NFC) costs zero allocations.
- `Cow::Owned(s)` when something changed — natural and obvious to consumers.
- The borrow checker enforces the right shape: callers can't accidentally hold the borrowed variant past the input's lifetime.

**Alternatives considered**:

- Always-`String`: simpler signature, but throws away the no-op fast path that's the whole reason this is a library API.
- `Result<&'a str, String>`: clever but conflates the success paths. Borrowed and owned are not failure modes.
- In-place mutation via `&mut String`: would require the caller to own a mutable buffer they may not have (the `hive-tainted-string` consumer threads `&str` through layers); also harder to reason about when the cleaner runs as part of a pipeline.

## Decision 3: Order of operations — strip then NFC, or NFC then strip?

**Decision**: **Strip / replace first, then NFC** (when enabled).

**Rationale**:

- Stripping operates on the raw input: the malicious-codepoint table is keyed by raw `u32`. Running NFC first would change codepoint identities under the table, breaking the lookup or worse, producing different rejections per platform if normalization output ever shifts.
- NFC after stripping is a normal operation on a well-formed `&str` containing no surprise codepoints.
- This order also matches how detection works today (raw walk) — the cleaner is a strict superset of detection plus an optional postprocessing step.

**Alternatives considered**:

- NFC first: violates the "raw codepoint walk" invariant the detector relies on; risks future drift.
- Both directions parameterised: more knobs, no use case.

## Decision 4: CLI default — stdout vs in-place

**Decision**: **Stdout by default**, `--in-place` opt-in.

**Rationale**:

- Convention: `sed`, `fmt`, `rustfmt`, `prettier`, `awk` — every Unix text-transformer of consequence writes stdout by default and treats in-place as a deliberate gesture.
- Safety: a `unicleaner clean foo.txt` that silently rewrites the file would be catastrophic on the first user's checked-in code.
- Composability: users who want to inspect the diff first can pipe `unicleaner clean foo.txt | diff foo.txt -`. They can't easily un-rewrite a file.
- `--in-place` writes to a temp sibling, fsyncs, and renames atomically — borrowed from the existing `model_store` write path used by hiveworks.

**Alternatives considered**:

- In-place by default: rejected outright. Operationally hostile.
- Diff output by default: surprising; conflates the cleaner with a diff tool.

## Decision 5: Error reporting — keep `Violation`?

**Decision**: Yes — `CleanResult.violations` is `Vec<Violation>` (the existing detector type).

**Rationale**:

- The `Violation` shape (file path, line, column, codepoint, category, severity, description) is what the detector already produces and what every downstream consumer is configured to log / serialize.
- Reusing it means a sanitizer-mode caller sees the same telemetry shape it would from `detect_in_string` — only the `output` payload is new.
- `file_path` defaults to a synthetic `<inline>` path when callers use the in-process API; that field already exists for the same reason in `detect_in_string`.

**Alternatives considered**:

- New `CleanViolation` struct with a `CleanAction` taken: tempting but the action is already implied by the policy and the (input, output) byte diff. Any caller that needs it can compute it from the policy. Adding a struct just to carry an enum the caller already knows is bad signal-to-noise.

## Decision 6: Policy configuration — flat fields vs builder

**Decision**: Flat-fields `CleanPolicy` struct + `with_*` chained mutators returning `Self`.

**Rationale**:

- Matches how `Configuration` is structured in the crate today (`src/config/mod.rs`): a plain struct with named fields, three named constructors (`strict / lossy / report_only`), per-field setters that take `&mut self` or consume `Self` for chaining.
- Avoids a separate `CleanPolicyBuilder` type that would clutter the API surface for no benefit at this size — the policy has six fields, all knowable up front.
- TOML-deserializable for the `--config` flag with no extra glue.

**Alternatives considered**:

- Typestate builder: overkill for ~6 fields.
- Phantom-typed defaults: the same ergonomics with twice the type baggage.

## Decision 7: Where the `cleaner` module lives

**Decision**: New top-level module `src/cleaner/` with `mod.rs` (entry points) and `policy.rs` (types). Re-exported from `lib.rs`.

**Rationale**:

- Mirrors the existing layout (`src/scanner/`, `src/report/`, `src/unicode/`).
- Keeps the policy types out of the entry-point module so the public API surface (`clean`, `CleanResult`) is the obvious thing readers see first.
- One-shot integration: `pub use cleaner::{clean, CleanPolicy, CleanResult, CleanAction};` at the crate root keeps the import line in `hive-tainted-string` short.

**Alternatives considered**:

- Single-file `src/cleaner.rs`: works for v1 but will outgrow itself the moment the second `CleanAction` variant ships.
- Folding into `src/scanner/`: blurs the detect/clean separation that the rest of the design rests on.
