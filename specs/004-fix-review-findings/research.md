# Research: Fix Review Findings

**Feature Branch**: `004-fix-review-findings`
**Date**: 2026-02-05

## Research Areas

### R1: Config Loading Pipeline (Findings #1, #2, #3)

**Question**: How should config be wired into the scanning pipeline?

**Decision**: Load config in `run_scan()` and pass `Arc<Configuration>` through `scan_files_parallel` to `scan_file_with_config`.

**Rationale**:
- `Configuration::from_file(path)` already exists in `src/config/mod.rs` and calls `parser::load_config(path)`.
- `scan_file_with_config(path, config)` already exists in `src/scanner/file_scanner.rs` but is never called from the parallel scanner.
- `scan_files_parallel` currently calls `scan_file(file)` which uses `Configuration::default()`.
- The fix is straightforward: add a `config: &Configuration` parameter to `scan_files_parallel`, load config once in `run_scan()`, and pass it through.

**Alternatives considered**:
- Global static config: Rejected because it prevents testing and makes config lifetime implicit.
- Per-file config loading: Rejected due to unnecessary I/O overhead.

**Config auto-discovery logic**:
1. If `--config <FILE>` provided: load that file, error if missing.
2. If not provided: check for `unicleaner.toml` in CWD. If found, load it. If not found, use `Configuration::default()`.

---

### R2: Init Command Schema Mismatch (Finding #2)

**Question**: What should `unicleaner init` generate?

**Decision**: Generate config matching the current parser schema (`[global]`, `[[rules]]`, `[languages.<ext>]`), ideally based on `examples/unicleaner.toml`.

**Rationale**:
- Current `run_init()` in `src/main.rs:160-185` generates a TOML stub with top-level `deny_by_default`, `[language_presets]`, and `[[file_rules]]`.
- The parser in `src/config/parser.rs` expects `[global] deny_by_default`, `[languages.<ext>] preset`, and `[[rules]]` (not `[[file_rules]]`).
- The mismatch means init output cannot be parsed by the config loader.

**Alternatives considered**:
- Change the parser to accept both schemas: Rejected as unnecessary complexity.
- Use `examples/unicleaner.toml` verbatim: Possible, but a template with comments explaining each section is better UX.

---

### R3: Rayon Thread Pool Usage (Finding #4)

**Question**: How to correctly use a local rayon thread pool?

**Decision**: Use `pool.install(|| { ... })` to execute `par_iter()` within the custom pool.

**Rationale**:
- Current code in `src/scanner/parallel.rs:15-20` creates a pool with `ThreadPoolBuilder::new().num_threads(threads).build()` but then drops it immediately. `par_iter()` runs on the global rayon pool.
- The correct pattern is:
  ```rust
  let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
  pool.install(|| {
      files.par_iter().for_each(|file| { ... });
  });
  ```
- This ensures `par_iter()` executes within the custom-sized pool.

**Alternatives considered**:
- `build_global()`: Rejected because it can only be called once per process, causing test failures.

---

### R4: Policy Enforcement Logic (Finding #5)

**Question**: How should `apply_config_rules()` be restructured?

**Decision**: Replace the current `apply_config_rules()` with policy checks that:
1. Use `find_matching_rule()` to get the single most-specific rule for the file.
2. Enforce `denied_code_points` as always-deny (overrides any allowlist).
3. Define "always-deny" patterns (bidi controls, Trojan Source) that cannot be suppressed by any config.
4. In deny-by-default mode without a matching rule, report all non-ASCII.

**Rationale**:
- Current `apply_config_rules()` in `src/scanner/file_scanner.rs:44-82` collects ALL matching rules and checks against all of them, ignoring rule priority.
- In deny-by-default mode, it only checks `allowed_ranges` and ignores `denied_code_points` entirely.
- In allow-by-default mode, it only reports characters in `denied_code_points`, which means built-in malicious patterns can be silently suppressed.

**Concrete logic**:
```
for each violation:
  if violation is_always_deny_pattern (bidi, trojan source):
    always report → keep
  else if matching_rule exists:
    if denied_code_points contains code_point:
      report → keep (explicit deny overrides allow)
    else if allowed_ranges contains code_point:
      suppress → remove
    else if deny_by_default:
      report → keep
    else:
      suppress → remove (allow-by-default, not explicitly denied)
  else (no matching rule):
    if deny_by_default:
      report → keep (no rule = no allowlist)
    else:
      suppress → remove
```

**Always-deny patterns** (cannot be suppressed by config):
- Bidi override controls: U+202A-202E (LRE, RLE, PDF, LRO, RLO)
- Bidi isolate controls: U+2066-2069 (LRI, RLI, FSI, PDI)
- These constitute "Trojan Source" attacks per CVE-2021-42574.

**Alternatives considered**:
- Making ALL malicious patterns always-deny: Rejected because some (like NBSP, combining marks) have legitimate uses in documentation/comments.

---

### R5: Encoding Override (Finding #3)

**Question**: How to wire `--encoding` through the scanning pipeline?

**Decision**: Add an `encoding_override: Option<DetectedEncoding>` parameter to the scanning functions. When set, skip auto-detection and use the forced encoding.

**Rationale**:
- `EncodingOption` is already parsed in `src/cli/args.rs:56-72` with variants `Utf8, Utf16Le, Utf16Be, Utf32Le, Utf32Be`.
- `DetectedEncoding` in `src/scanner/encoding.rs` has matching variants.
- The mapping is straightforward. The override bypasses `detect_decode_with_encoding()` and uses direct decoding.

**Alternatives considered**:
- Remove the flag entirely: Rejected because encoding override is useful for edge cases.

---

### R6: JSON Schema Alignment (Finding #6)

**Question**: Update spec schema to match code, or update code to match spec?

**Decision**: Update the spec contract JSON schema to match current `ScanResult` serialization (Option B from review).

**Rationale**:
- The current code's JSON output is functional and tested. Changing it would break existing consumers.
- The spec schema was aspirational and never enforced.
- Updating the spec is lower risk and lower effort.

**Fields to document in updated schema**:
- `violations[]`: `file_path`, `line`, `column`, `code_point` (numeric), `character`, `context`, `pattern_name`, `category`, `severity`, `message`, `encoding`
- Top level: `files_scanned`, `files_clean`, `files_with_violations`, `errors[]`, `duration`, `config_used`

---

### R7: Column Calculation (Finding #8)

**Question**: How to report character-based columns instead of byte offsets?

**Decision**: Change column calculation from `grapheme_indices` byte offset to character count. Add `byte_offset` as a separate field.

**Rationale**:
- `grapheme_indices(true)` returns byte offsets, not character positions.
- Most editors (VS Code, vim, etc.) count columns by Unicode scalar values (chars).
- The fix: count characters as we iterate instead of using the byte index.
- Add a `byte_offset` field to `Violation` for machine consumers who need exact byte positions.

**Implementation approach**:
- Track a `char_column` counter that increments by `grapheme.chars().count()` per grapheme.
- Use `char_column + 1` (1-indexed) as the reported column.
- Store the byte offset from `grapheme_indices` in a new `byte_offset` field.

---

### R8: Error Classification (Finding #9)

**Question**: How to properly classify scan errors?

**Decision**: Map `std::io::ErrorKind` to `ErrorType` variants in `scan_files_parallel`.

**Rationale**:
- Current code in `parallel.rs:33-38` hardcodes all errors as `ErrorType::IoError`.
- The `ErrorType` enum already has `EncodingError`, `PermissionDenied`, `ParseError`.
- Mapping: `ErrorKind::PermissionDenied → ErrorType::PermissionDenied`, encoding errors from `detect_decode_with_encoding` → `ErrorType::EncodingError`, everything else → `ErrorType::IoError`.

**Exit code 3 logic**:
- Current `exit_code()` in `src/report/mod.rs:32-41` returns 0 (clean), 1 (violations), or 2 (errors).
- Change: return 3 when `errors.len() > 0 && violations.is_empty()`. Return 1 when violations exist (regardless of errors). Return 2 only for fatal errors (config parse failure, etc.).

---

### R9: Static Pattern/Preset Initialization (Findings #10, #11)

**Question**: How to make patterns and presets static?

**Decision**: Use `once_cell::sync::Lazy` for both `get_malicious_patterns()` and `get_all_presets()`.

**Rationale**:
- `get_malicious_patterns()` in `src/unicode/malicious.rs` builds large `Vec<u32>` ranges on every call. Called per-file via `detect_in_string()`.
- `get_all_presets()` in `src/config/presets.rs` builds a `HashMap` on every call. Called per-file via `get_preset()`.
- Both are pure functions with no parameters → perfect candidates for `Lazy<T>`.
- `once_cell` is already in `Cargo.toml` dependencies.

---

### R10: Binary Detection (Finding #12)

**Question**: How to improve binary file detection?

**Decision**: Add a control-byte ratio heuristic alongside the existing consecutive-nulls check.

**Rationale**:
- Current `is_binary()` in `src/scanner/encoding.rs:295-316` only checks for >10 consecutive nulls.
- Random binary data with few nulls passes through and causes decoding errors.
- Additional heuristic: if >30% of the first 8KB consists of non-text control bytes (bytes 0-8, 14-31 except TAB/CR/LF), classify as binary.
- No external dependencies needed.

**Alternatives considered**:
- Extension-based skip list: Useful as an optimization but not sufficient alone (files can be misnamed).
- `file` command/magic bytes: Adds external dependency, rejected per project guidelines.

---

### R11: CI Workflow Fix (Finding #7)

**Question**: What needs changing in `.github/workflows/pr-check.yml`?

**Decision**: Replace `--output json` with `--format json` and update jq field references.

**Rationale**:
- `--output` is not a valid flag; CLI uses `--format` (or `-f`).
- jq references like `.description` should be `.message` and `.pattern` should be `.pattern_name`.
- Workflow should also use `scan` subcommand explicitly for clarity.

---

### R12: Benchmark Harness Configuration (Finding #16)

**Question**: How to fix benchmark compilation?

**Decision**: Add `[[bench]]` entries for all 6 benchmark files with `harness = false`.

**Rationale**:
- Only `scan_performance` has a `[[bench]]` entry in `Cargo.toml`.
- The other 5 (`unicode_heavy`, `small_repo`, `medium_repo`, `large_repo`, `memory_usage`) use `criterion_main!` and need `harness = false`.

---

### R13: Stale Tests (Finding #17)

**Question**: Remove stale tests or resurrect them?

**Decision**: Audit each test directory. Remove files that reference non-existent APIs. Keep and wire in files that test existing functionality.

**Rationale**:
- `tests/performance/`, `tests/unit/`, `tests/regression/`, `tests/contract/` exist but are not included by any test harness.
- Some reference removed APIs (e.g., `detect_malicious_unicode`).
- Tests that reference valid APIs should be included via `tests/integration.rs` mod includes.
- Tests referencing removed APIs should be deleted.

---

### R14: Nix Flake Cleanup (Finding #14)

**Question**: Is OpenSSL actually needed?

**Decision**: Remove `openssl` and `pkg-config` from `flake.nix` buildInputs.

**Rationale**:
- No Rust dependency in `Cargo.toml` requires OpenSSL. The project uses `encoding_rs` (pure Rust), not `openssl-sys`.
- `AGENTS.md` explicitly says "AVOID openssl".
- Git integration uses the `git` CLI command, not `libgit2`.

---

### R15: .gitignore / Cargo.lock (Finding #15)

**Question**: Should Cargo.lock be in .gitignore for a binary crate?

**Decision**: Remove `Cargo.lock` from `.gitignore`.

**Rationale**:
- Rust best practice: binary crates should commit `Cargo.lock` for reproducible builds.
- `Cargo.lock` is already committed in the repository despite being in `.gitignore`.
- This contradiction should be resolved by removing it from `.gitignore`.

---

### R16: Documentation Drift (Finding #13)

**Question**: Implement missing flags or update docs?

**Decision**: Update docs to match implemented CLI. Do not implement unplanned flags.

**Rationale**:
- Implementing `--output <FILE>`, `--json`, `--threads`, `--max-file-size` would be scope creep.
- The simpler fix is to update README.md and docs/DOCKER.md to reference only existing flags.
- Future features can be added when needed with proper specs.
