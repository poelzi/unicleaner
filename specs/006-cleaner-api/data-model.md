# Data Model: Cleaner API

## New Entities

### CleanAction

What to do with a codepoint that the policy considers malicious.

- **Variants**:
  - `Strip` — remove the codepoint from the output entirely. The byte-length of the output may be shorter than the input.
  - `Replace(char)` — replace the codepoint with `char`. Common values: `'?'` (visible), `'\u{FFFD}'` (REPLACEMENT CHARACTER, machine-readable), `'_'` (filename-friendly). Not all replacements are length-preserving.
  - `KeepWithMark` — leave the codepoint in place but record a `Violation`. Used by callers that want telemetry without actual mutation (e.g. dashboards) and by the `report_only` preset.
- **Derives**: `Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize`.
- **Default**: not applicable (caller picks via `CleanPolicy::strict()` etc.).

### CleanPolicy

Operator-controlled configuration for `clean()`. Cheap to construct; cheap to clone.

- **Fields**:
  - `default_action: CleanAction` — used for any malicious codepoint with no per-category override.
  - `per_category: BTreeMap<MaliciousCategory, CleanAction>` — overrides keyed by the existing `MaliciousCategory` enum (`ZeroWidth`, `BidiOverride`, `Homoglyph`, `ControlChar`, `NonStandard`).
  - `denied_code_points: Vec<u32>` — extra codepoints to treat as malicious; empty by default. Mirrors the existing `Configuration.denied_code_points`.
  - `allowed_ranges: Option<Vec<UnicodeRange>>` — when `deny_by_default = true`, codepoints outside these ranges are treated as malicious; matches the existing detector flag.
  - `deny_by_default: bool` — see `allowed_ranges`. Default `false`.
  - `normalize_nfc: bool` — apply NFC normalization to the output after stripping. Default `false`.
- **Derives**: `Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize`.
- **Constructors**:
  - `CleanPolicy::strict()` — `default_action = Strip`, NFC off, no per-category overrides, no allow / deny lists. Stripping every malicious codepoint is the safest universal default.
  - `CleanPolicy::lossy()` — `default_action = Replace('\u{FFFD}')`, NFC off. Preserves byte positions where possible (replacement char is 3 bytes; many malicious targets are 1–4 bytes), useful for log lines where alignment matters.
  - `CleanPolicy::report_only()` — `default_action = KeepWithMark`, NFC off. Output equals input (modulo the `Cow::Borrowed` fast path); only `violations` is populated. Drop-in replacement for `detect_in_string` callers that want the unified return type.
- **Mutators (chainable)**:
  - `with_action(category: MaliciousCategory, action: CleanAction) -> Self`
  - `with_default_action(action: CleanAction) -> Self`
  - `with_nfc(enable: bool) -> Self`
  - `with_denied(code_points: impl IntoIterator<Item = u32>) -> Self`
  - `with_allowed_ranges(ranges: Vec<UnicodeRange>, deny_by_default: bool) -> Self`
- **Resolution**: `effective_action(category: MaliciousCategory) -> CleanAction` — returns the override if present, else `default_action`. Crate-private; the cleaner's hot loop calls this once per codepoint match.

### CleanResult

What `clean()` returns.

- **Fields**:
  - `output: Cow<'a, str>` — the (possibly cleaned) string. Borrowed when nothing changed; owned otherwise.
  - `violations: Vec<Violation>` — every codepoint the policy reacted to (one entry each, in input order). Empty when nothing matched.
  - `modified: bool` — `true` iff the cleaner mutated the input. Specifically: `true` when at least one `Strip` or `Replace` took effect, **or** when NFC normalization changed any byte. `false` for `report_only` mode regardless of how many violations were recorded.
- **Derives**: `Debug, Clone, PartialEq, Eq`.
- **Lifetime**: `'a` is the input string's lifetime; `Cow::Borrowed(output)` cannot outlive `input`.

## Reused Entities (no schema change)

- **`MaliciousPattern`** (`src/unicode/malicious.rs:25`) — referenced through `pattern_for(code_point) -> Option<&'static MaliciousPattern>` to look up the category for every flagged codepoint.
- **`MaliciousCategory`** (`src/unicode/malicious.rs:5`) — keys `CleanPolicy.per_category`.
- **`Severity`** (`src/unicode/malicious.rs:14`) — propagated into the `Violation` records that `clean()` emits, unchanged from the detector.
- **`Violation`** (`src/report/violation.rs`) — emitted exactly the same way `detect_in_string_with_policy` emits them today. The cleaner does not introduce a new violation shape.
- **`UnicodeRange`** (`src/unicode/ranges.rs`) — backing type for `CleanPolicy.allowed_ranges`.

## Module Layout

```text
src/cleaner/
├── mod.rs        # `pub fn clean(...)` + `CleanResult`
└── policy.rs     # `CleanPolicy`, `CleanAction`, presets, mutators

src/lib.rs
└── pub use cleaner::{clean, CleanPolicy, CleanResult, CleanAction};
```

## Relationships

```text
                   ┌─────────────────┐
                   │  CleanPolicy    │
                   │  (operator cfg) │
                   └────────┬────────┘
                            │ effective_action()
                            ▼
                   ┌─────────────────┐
   input: &str ───►│   clean()       │───► CleanResult
                   │   per-codepoint │       ├─ output: Cow<str>
                   │   walk + opt    │       ├─ violations: Vec<Violation>
                   │   NFC pass      │       └─ modified: bool
                   └─────────────────┘
                            │
                            └─► pattern_for() (existing table, unchanged)
```

## Invariants

- **I-1**: `CleanResult.modified == false` ⇒ `output.as_ref() == input` byte-for-byte.
- **I-2**: `CleanResult.violations.is_empty() && !policy.normalize_nfc` ⇒ `output` is `Cow::Borrowed`.
- **I-3**: For every codepoint position `(byte_offset, codepoint)` in the input where `pattern_for(codepoint).is_some()` (or the codepoint is in `policy.denied_code_points`, or fails the `deny_by_default` check), exactly one entry is appended to `violations`.
- **I-4**: NFC normalization runs on the post-strip / post-replace string, never the raw input. (See research.md Decision 3.)
- **I-5**: All `CleanPolicy::*()` presets satisfy `with_nfc(false)` — NFC is opt-in.
