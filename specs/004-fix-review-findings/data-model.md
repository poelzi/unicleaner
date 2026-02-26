# Data Model: Fix Review Findings

**Feature Branch**: `004-fix-review-findings`
**Date**: 2026-02-05

## Entity Changes

This feature primarily modifies existing entities and function signatures rather than introducing new entities. Below documents the changes to existing data structures.

### Configuration (existing - `src/config/mod.rs`)

No structural changes. The entity is already correctly defined. The change is in **how** it flows through the system:

- Currently: `Configuration::default()` used implicitly in `scan_file()`.
- After: `Configuration` loaded in `run_scan()` and passed via `Arc<Configuration>` through `scan_files_parallel` → `scan_file_with_config`.

### Violation (modified - `src/report/violation.rs`)

**New field**:
- `byte_offset: usize` — Exact byte position within the line for machine consumers.

**Modified field**:
- `column: usize` — Changes from byte offset to character-based position (Unicode scalar value count, 1-indexed).

**Serialization impact**: JSON output gains `byte_offset` field. `column` semantics change (documented in updated JSON schema).

### ScanError (modified - `src/report/violation.rs`)

No structural changes. The `ErrorType` enum already has the correct variants (`IoError`, `EncodingError`, `ParseError`, `PermissionDenied`). The change is in **classification logic** in `scan_files_parallel`:

- Currently: All errors mapped to `ErrorType::IoError`.
- After: Errors classified by `std::io::ErrorKind` and error source type.

### ScanResult (modified - `src/report/mod.rs`)

**Modified method**:
- `exit_code()` — Changes from `{0, 1, 2}` to `{0, 1, 2, 3}`:
  - `0`: No violations, no errors (clean).
  - `1`: Violations found (regardless of errors).
  - `2`: Fatal error (config parse failure, etc.).
  - `3`: Partial success — errors occurred but no violations found.

## Function Signature Changes

### `scan_files_parallel` (`src/scanner/parallel.rs`)

**Before**:
```
pub fn scan_files_parallel(
    files: Vec<PathBuf>,
    num_threads: Option<usize>,
) -> (Vec<Violation>, Vec<ScanError>)
```

**After**:
```
pub fn scan_files_parallel(
    files: Vec<PathBuf>,
    num_threads: Option<usize>,
    config: &Configuration,
    encoding_override: Option<DetectedEncoding>,
) -> (Vec<Violation>, Vec<ScanError>)
```

### `scan_file_with_config` (`src/scanner/file_scanner.rs`)

**Before**:
```
pub fn scan_file_with_config(path: &Path, config: &Configuration) -> Result<Vec<Violation>>
```

**After**:
```
pub fn scan_file_with_config(
    path: &Path,
    config: &Configuration,
    encoding_override: Option<DetectedEncoding>,
) -> Result<Vec<Violation>>
```

### `detect_in_string` (`src/scanner/unicode_detector.rs`)

No signature change. Internal column calculation changes from byte offset to character count.

### `apply_config_rules` (`src/scanner/file_scanner.rs`)

Signature unchanged but logic rewritten to:
1. Use `find_matching_rule()` for single most-specific rule.
2. Check always-deny patterns first.
3. Enforce `denied_code_points` overriding allowed ranges.
4. Respect deny-by-default / allow-by-default mode correctly.

## State Transitions

### Config Loading State Machine

```
No --config flag provided
  → Check CWD for unicleaner.toml
    → Found: Load and validate → Configuration
    → Not found: Use Configuration::default()

--config <FILE> provided
  → File exists and readable
    → Parse TOML
      → Valid schema: Configuration
      → Invalid schema: Exit with error (code 2)
  → File missing/unreadable: Exit with error (code 2)
```

### Policy Evaluation Per-Violation

```
Violation detected
  → Is always-deny pattern? (bidi controls, Trojan Source)
    → Yes: REPORT (cannot be suppressed)
    → No: Check matching rule
      → Rule found:
        → In denied_code_points? → REPORT (explicit deny wins)
        → In allowed_ranges? → SUPPRESS
        → deny_by_default? → REPORT
        → allow_by_default? → SUPPRESS
      → No rule found:
        → deny_by_default? → REPORT
        → allow_by_default? → SUPPRESS
```

## Relationships

```
Args ──loads──→ Configuration
                    │
                    ├──contains──→ FileRule[] (sorted by priority)
                    │                  │
                    │                  ├──has──→ allowed_ranges: UnicodeRange[]
                    │                  └──has──→ denied_code_points: u32[]
                    │
                    └──contains──→ language_presets: HashMap<ext, preset_name>
                                       │
                                       └──resolves──→ LanguagePreset
                                                          └──has──→ allowed_ranges

scan_files_parallel(files, threads, config, encoding_override)
    │
    └──per file──→ scan_file_with_config(path, config, encoding_override)
                       │
                       ├──→ is_binary(bytes) → skip if binary
                       ├──→ decode(bytes, encoding_override) → String
                       ├──→ detect_in_string(content, path) → Violation[]
                       └──→ apply_config_rules(violations, path, config) → filtered Violation[]
```
