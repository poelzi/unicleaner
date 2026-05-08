# Quickstart: Cleaner API

## Library use (in-process sanitization)

Add `unicleaner` to `Cargo.toml` and call `clean()`:

```rust
use std::borrow::Cow;
use unicleaner::{clean, CleanPolicy};

let dirty = "admin\u{202E}lortnoc";   // bidi-override Trojan Source
let result = clean(dirty, &CleanPolicy::strict());

assert_eq!(result.output.as_ref(), "adminlortnoc");
assert!(result.modified);
assert_eq!(result.violations.len(), 1);
assert_eq!(result.violations[0].code_point, 0x202E);
```

For prose / chat input where NFC normalization is wanted:

```rust
let policy = CleanPolicy::strict().with_nfc(true);
let result = clean("e\u{0301}", &policy);   // NFD-encoded é

assert_eq!(result.output.as_ref(), "é");
assert!(result.modified);                    // NFC changed bytes
assert!(result.violations.is_empty());       // no malicious chars
```

For sites that want a visible marker rather than silent stripping:

```rust
let policy = CleanPolicy::lossy();   // default_action = Replace('\u{FFFD}')
let result = clean("hi\u{200B}there", &policy);
assert_eq!(result.output.as_ref(), "hi\u{FFFD}there");
```

For per-category tuning (e.g. strip zero-widths, replace bidi, keep homoglyphs):

```rust
use unicleaner::{CleanAction, CleanPolicy};
use unicleaner::unicode::malicious::MaliciousCategory;

let policy = CleanPolicy::strict()
    .with_action(MaliciousCategory::BidiOverride, CleanAction::Replace('?'))
    .with_action(MaliciousCategory::Homoglyph, CleanAction::KeepWithMark);
```

## Fast-path zero-allocation guarantee

When the input contains nothing the policy would act on **and** NFC is disabled, `clean()` returns `Cow::Borrowed` over the input — no allocation:

```rust
let input = "plain ascii";
let result = clean(input, &CleanPolicy::strict());

match result.output {
    Cow::Borrowed(s) => assert_eq!(s.as_ptr(), input.as_ptr()),
    Cow::Owned(_) => panic!("clean input must not allocate"),
}
```

## CLI use (one-shot file cleanup)

```bash
# Default: write cleaned content to stdout. Source file is never touched.
unicleaner clean src/foo.rs

# Pipe-friendly. Diff against the original to inspect what changed.
unicleaner clean src/foo.rs | diff src/foo.rs -

# In-place rewrite (atomic: write to .tmp, fsync, rename).
unicleaner clean --in-place src/foo.rs

# Pick a policy preset:
unicleaner clean --policy lossy src/foo.rs        # replace with U+FFFD
unicleaner clean --policy report-only src/foo.rs  # no mutation; use scan instead

# Reuse an existing detector config:
unicleaner clean --config unicleaner.toml src/foo.rs
```

Exit codes:

- `0` — cleaned successfully (file existed; output written).
- `1` — generic failure (file not found, IO error, malformed config).
- `2` — policy was `report-only` AND violations were detected. Mirrors the existing `unicleaner scan` "violations found" exit code.

## Policy presets at a glance

| Preset                  | Default action            | NFC | Use when                                                        |
| ----------------------- | ------------------------- | --- | --------------------------------------------------------------- |
| `CleanPolicy::strict()` | `Strip`                   | off | Source code, identifiers, JSON keys — anywhere you want the bad codepoint just gone. |
| `CleanPolicy::lossy()`  | `Replace('\u{FFFD}')`     | off | Logs, error messages, audit trails — keep byte-positions roughly aligned, mark visibly. |
| `CleanPolicy::report_only()` | `KeepWithMark`       | off | Telemetry / dashboards — no mutation, but unified `CleanResult` shape with sanitizer mode. |

## Integration: drop-in for `detect_in_string` callers

```rust
// Before
let violations = unicleaner::scanner::unicode_detector::detect_in_string(&content, path);

// After (no behavioural change; exact same Violation entries)
let result = unicleaner::clean(&content, &unicleaner::CleanPolicy::report_only());
let violations = result.violations;
```

The two return-shapes are identical apart from the new `output` and `modified` fields, which `report_only` callers can ignore.
