# Feature Specification: Fix Review Findings

**Feature Branch**: `004-fix-review-findings`
**Created**: 2026-02-05
**Status**: Draft
**Input**: User description: "Create a clear plan to remedy all review findings in review.md. Ensure for every finding there is a testcase."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Config-Driven Scanning (Priority: P1)

A user creates or edits a `unicleaner.toml` configuration file to define their project's Unicode policy (allowed blocks, denied characters, language presets). They run `unicleaner scan --config unicleaner.toml` and the scan results reflect the configured policy rather than ignoring it.

**Why this priority**: Configuration is the core feature that governs all policy decisions. Without it working end-to-end, the tool cannot enforce custom policies and all config-related features are dead code.

**Independent Test**: Run a scan with a config that allows Greek characters in `.rs` files, then verify Greek characters are not reported as violations, while they would be reported without config.

**Acceptance Scenarios**:

1. **Given** a valid config file allowing Greek block in `.rs` files, **When** user runs `unicleaner scan --config config.toml` on a file containing Greek characters, **Then** no violations are reported for Greek characters.
2. **Given** no `--config` flag and a `unicleaner.toml` in the current directory, **When** user runs `unicleaner scan`, **Then** the tool automatically loads and applies that config.
3. **Given** `--config missing.toml` pointing to a non-existent file, **When** user runs the scan, **Then** the tool exits with an error message indicating the config file was not found.
4. **Given** no config file provided and none found in the current directory, **When** user runs `unicleaner scan`, **Then** the tool uses default settings and scanning proceeds normally.

---

### User Story 2 - Correct Policy Enforcement (Priority: P1)

A security-conscious user configures deny-by-default mode with specific allowed ranges and explicit denied characters. The tool correctly enforces rule priority (more specific rules override less specific), explicit denies always win over allowlists, and built-in malicious patterns (Trojan Source bidi controls) are always reported regardless of allow-by-default mode.

**Why this priority**: Policy enforcement is security-sensitive. Incorrect filtering could suppress real attacks or produce false negatives.

**Independent Test**: Create a config with an allowed Unicode range that includes bidi control characters, and a denied_characters list for those bidi controls. Verify the bidi controls are still reported as violations.

**Acceptance Scenarios**:

1. **Given** deny-by-default mode with `denied_characters` including U+200B, **When** scanning a file containing U+200B, **Then** the violation is reported even if U+200B falls within an allowed range.
2. **Given** allow-by-default mode, **When** scanning a file containing Trojan Source bidi controls (RLO, LRI, etc.), **Then** the bidi controls are still reported as violations.
3. **Given** two rules where one is file-specific (e.g., `*.rs`) and another is global (`*`), **When** scanning a `.rs` file, **Then** the file-specific rule takes priority over the global rule.

---

### User Story 3 - Valid Init Config Generation (Priority: P1)

A new user runs `unicleaner init` to bootstrap their configuration. The generated config file is valid, parseable by the tool, and matches the current parser schema.

**Why this priority**: First-time user experience. If init produces broken config, users cannot get started.

**Independent Test**: Run `unicleaner init`, then load the generated config with the parser and verify it parses without error.

**Acceptance Scenarios**:

1. **Given** user runs `unicleaner init`, **When** the config is generated, **Then** the file parses successfully with `Configuration::from_file`.
2. **Given** user runs `unicleaner init`, **When** the config is generated, **Then** the structure matches the current parser schema (uses `[global]`, `[[rules]]`, `[languages.<ext>]`).

---

### User Story 4 - Working Concurrency Control (Priority: P2)

A user or CI system passes `--jobs 4` to limit parallelism. The scan actually executes with the specified number of threads.

**Why this priority**: CI environments often need to control resource usage. A broken `--jobs` flag undermines this.

**Independent Test**: Run scan with `--jobs 1` and verify that scanning uses exactly 1 thread in the rayon pool.

**Acceptance Scenarios**:

1. **Given** `--jobs 4`, **When** scanning is performed, **Then** the rayon thread pool uses exactly 4 threads.
2. **Given** `--jobs 1`, **When** scanning is performed, **Then** scanning runs single-threaded.
3. **Given** `--jobs 0`, **When** user starts a scan, **Then** the CLI exits with a validation error and non-zero status.

---

### User Story 5 - Working Encoding Override (Priority: P2)

A user knows a file is UTF-16LE but automatic detection fails. They pass `--encoding utf16-le` and the scan decodes the file using the specified encoding.

**Why this priority**: Encoding override is an existing CLI flag that does nothing. It should either work or be removed.

**Independent Test**: Create a UTF-16LE file, scan with `--encoding utf16-le`, and verify the file is decoded correctly and violations (if any) are found.

**Acceptance Scenarios**:

1. **Given** a UTF-16LE encoded file, **When** user passes `--encoding utf16-le`, **Then** the file is decoded using UTF-16LE and scanned for Unicode issues.
2. **Given** an invalid encoding flag value, **When** user passes `--encoding invalid`, **Then** the tool exits with a clear error message.

---

### User Story 6 - Accurate JSON Output Schema (Priority: P2)

A CI pipeline consumes unicleaner's JSON output. The JSON schema is well-defined, documented, and matches actual output so that downstream tooling can reliably parse results.

**Why this priority**: CI integration is a key use case. Mismatched schemas break automation.

**Independent Test**: Run scan with `--format json`, capture output, and validate it against the documented JSON schema.

**Acceptance Scenarios**:

1. **Given** `--format json`, **When** scan completes, **Then** the JSON output matches the documented schema (all required fields present, correct types).
2. **Given** the spec contract JSON schema, **When** comparing to actual output, **Then** they are consistent (same field names, types, and structure).

---

### User Story 7 - Working CI Workflow (Priority: P2)

The GitHub Actions workflow for PR security checks uses correct CLI flags and parses the actual JSON output fields.

**Why this priority**: Broken CI workflows undermine the tool's integration story.

**Independent Test**: Verify the workflow YAML references only flags that exist in the CLI (`--format` not `--output`) and parses JSON fields that exist in actual output.

**Acceptance Scenarios**:

1. **Given** the PR check workflow, **When** it invokes unicleaner, **Then** it uses `--format json` (not `--output json`).
2. **Given** the PR check workflow, **When** it parses JSON output with jq, **Then** it references fields that exist in the actual JSON output.

---

### User Story 8 - Correct Column Reporting (Priority: P2)

A developer sees a violation at a specific line and column. They navigate to that position in their editor and find the actual problematic character at or very near the reported column.

**Why this priority**: Incorrect column numbers make remediation harder, especially for multi-byte scripts.

**Independent Test**: Scan a file with multi-byte characters and verify the reported column matches the character/grapheme column, not the byte offset.

**Acceptance Scenarios**:

1. **Given** a file with multi-byte UTF-8 characters before a violation, **When** scanning reports a column, **Then** the column represents a character-based position (not byte offset).
2. **Given** a violation report, **When** both byte_offset and column are available, **Then** byte_offset reflects exact byte position and column reflects human-readable character position.

---

### User Story 9 - Proper Error Classification and Exit Codes (Priority: P2)

A CI system uses exit codes to determine pipeline behavior. The exit code correctly distinguishes between "clean scan", "violations found", "scan errors only", and "both violations and errors".

**Why this priority**: CI integration requires reliable exit code semantics.

**Independent Test**: Scan a directory with an unreadable file and a clean file; verify exit code reflects partial success. Scan with encoding errors; verify error type is `EncodingError` not generic `IoError`.

**Acceptance Scenarios**:

1. **Given** a scan with errors but no violations, **When** the scan completes, **Then** exit code is 3 (partial success).
2. **Given** a permission-denied error on a file, **When** the error is recorded, **Then** the error type is `PermissionDenied` (not generic `IoError`).
3. **Given** an encoding failure, **When** the error is recorded, **Then** the error type is `EncodingError`.

---

### User Story 10 - Efficient Pattern and Preset Lookups (Priority: P3)

When scanning large repositories with many files, the tool does not repeatedly allocate large data structures for malicious pattern detection or language preset lookups.

**Why this priority**: Performance matters for large repos but is not a correctness issue.

**Independent Test**: Verify that malicious patterns and language presets are initialized once (static/lazy) rather than rebuilt per file.

**Acceptance Scenarios**:

1. **Given** scanning 1000 files, **When** malicious pattern detection runs, **Then** pattern data is allocated once and reused across all files.
2. **Given** scanning files with language presets, **When** preset lookup occurs, **Then** presets are loaded once from a static cache.

---

### User Story 11 - Robust Binary File Detection (Priority: P3)

When scanning a real-world repository containing binaries (images, compiled objects, compressed files), the tool skips binary files gracefully without producing scan errors.

**Why this priority**: Reduces noise in scan results for real repositories.

**Independent Test**: Scan a directory containing various binary files (PNG, compiled .o, .zip) and verify they are skipped without errors.

**Acceptance Scenarios**:

1. **Given** a directory with binary files (images, compiled objects), **When** scanning, **Then** binary files are skipped without producing errors.
2. **Given** a binary file with few consecutive null bytes, **When** the heuristic runs, **Then** the file is still identified as binary using additional heuristics (control-byte ratio and null-byte patterns).

---

### User Story 12 - Accurate Documentation (Priority: P3)

A user reads the README or docs and copies a command. The command works as documented because all referenced CLI flags and options actually exist.

**Why this priority**: Documentation accuracy is important for UX but not a runtime correctness issue.

**Independent Test**: Parse README and docs for CLI invocations and verify each referenced flag exists in the actual CLI parser.

**Acceptance Scenarios**:

1. **Given** the README, **When** a user copies any documented CLI command, **Then** the command is accepted by the CLI parser without errors.
2. **Given** docs/DOCKER.md, **When** a user copies any documented command, **Then** the command works with the current CLI.

---

### User Story 13 - Clean Build System (Priority: P3)

The Nix flake does not include unnecessary dependencies (OpenSSL), `.gitignore` is consistent with what is committed, and all benchmark targets compile and run.

**Why this priority**: Build hygiene affects contributor experience and build reproducibility.

**Independent Test**: Build with Nix and verify no OpenSSL dependency is pulled. Run `cargo bench` and verify all benchmark targets compile.

**Acceptance Scenarios**:

1. **Given** the Nix flake, **When** building, **Then** OpenSSL and pkg-config are not in build inputs.
2. **Given** `.gitignore`, **When** `Cargo.lock` is committed, **Then** it is not listed in `.gitignore`.
3. **Given** 6 benchmark files in `benches/`, **When** running `cargo bench`, **Then** all targets compile and execute.

---

### User Story 14 - All Tests Compile and Run (Priority: P3)

All test files under `tests/` are either compiled and executed as part of the test suite, or removed if stale.

**Why this priority**: Dead test code gives false confidence about coverage.

**Independent Test**: Run `cargo test` and verify all test files under `tests/` are included in the test run. No stale references to removed APIs.

**Acceptance Scenarios**:

1. **Given** all test files under `tests/`, **When** running `cargo test`, **Then** every test file is compiled and executed (no orphaned test modules).
2. **Given** test files referencing internal APIs, **When** compiling, **Then** all API references are valid and up-to-date.

---

### Edge Cases

- What happens when a config file has valid TOML but invalid unicleaner schema? The tool should exit with a descriptive validation error.
- What happens when `--encoding` is combined with binary file detection? The encoding override should force decoding even for suspected binary files.
- What happens when `--jobs 0` is passed? The tool should fail fast with a validation error (jobs must be >= 1).
- What happens when the same code point appears in both `allowed_ranges` and `denied_characters`? `denied_characters` takes precedence.
- What happens when no rules match a file in deny-by-default mode? All non-ASCII characters should be reported.
- What happens when a CI workflow runs on a repo with no config file? The tool should use defaults and still produce valid JSON output.

## Requirements *(mandatory)*

### Functional Requirements

**Phase 1 - CLI Correctness (Critical)**

- **FR-001**: The tool MUST load and apply configuration from a file specified via `--config <FILE>`.
- **FR-002**: The tool MUST auto-discover and load `unicleaner.toml` from the current directory when `--config` is not specified.
- **FR-003**: The tool MUST exit with an error when `--config` points to a non-existent or unreadable file.
- **FR-004**: The `unicleaner init` command MUST generate a config file that parses successfully with the current config parser.
- **FR-005**: The `--encoding` flag MUST override automatic encoding detection and force the specified encoding for file decoding.
- **FR-006**: The `--jobs` flag MUST control the actual number of threads used for parallel scanning, and values less than 1 MUST be rejected.

**Phase 2 - Policy Enforcement (Security-Critical)**

- **FR-007**: In deny-by-default mode, `denied_characters` MUST always be reported as violations, even if the character falls within an allowed range.
- **FR-008**: Rule priority MUST be respected: file-specific rules (e.g., `*.rs`) override global rules (`*`).
- **FR-009**: Built-in malicious patterns (Trojan Source bidi controls, zero-width characters used for attacks) MUST always be reported regardless of allow-by-default mode.
- **FR-010**: The tool MUST use the most specific matching rule (via `find_matching_rule`) for policy decisions, not aggregate all matching rules.

**Phase 3 - Output and CI Alignment**

- **FR-011**: JSON output MUST conform to a single documented schema, and all fields referenced by CI workflows MUST exist in actual output.
- **FR-012**: The GitHub Actions PR check workflow MUST use correct CLI flags (`--format` not `--output`) and reference valid JSON fields.
- **FR-013**: Column numbers in violation reports MUST represent character-based positions (not byte offsets). Byte offsets SHOULD also be available for machine consumers.
- **FR-014**: Scan errors MUST be classified by type (EncodingError, PermissionDenied, IoError) rather than flattened to a single generic type.
- **FR-015**: Exit code 3 (partial success) MUST be returned when scan errors occur but no violations are found.

**Phase 4 - Performance and Hygiene**

- **FR-016**: Malicious pattern definitions MUST be initialized once and reused across files (not rebuilt per scan invocation).
- **FR-017**: Language presets MUST be cached statically rather than rebuilt on every lookup.
- **FR-018**: Binary file detection MUST use multiple heuristics (not just consecutive nulls) to reduce false negatives.
- **FR-019**: All documentation (README, docs/DOCKER.md) MUST reference only CLI flags that actually exist.
- **FR-020**: The Nix flake MUST NOT include OpenSSL or pkg-config as build inputs.
- **FR-021**: `.gitignore` MUST NOT list `Cargo.lock` (since it is committed for this binary crate).
- **FR-022**: All benchmark files in `benches/` MUST have corresponding `[[bench]]` entries in `Cargo.toml` with `harness = false`.
- **FR-023**: All test files under `tests/` MUST either compile and execute, or be removed if stale.

### Key Entities

- **Configuration**: The parsed representation of a `unicleaner.toml` file, containing global settings, rules, and language presets.
- **Rule**: A policy unit that matches files by pattern and specifies allowed/denied Unicode ranges and characters, with a priority based on specificity.
- **Violation**: A detected Unicode issue in a file, with line, column (character-based), byte offset, code point, pattern name, and message.
- **ScanError**: A classified error encountered during scanning, with a specific error type (Encoding, Permission, IO).
- **ScanResult**: The complete output of a scan, containing violations, errors, file counts, duration, and config metadata.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Scanning with a custom config produces different results than scanning without config, proving config is applied end-to-end.
- **SC-002**: `unicleaner init` generates a config that loads and parses without errors 100% of the time.
- **SC-003**: `--jobs N` results in scanning using exactly N threads (verifiable in tests).
- **SC-004**: `--encoding <value>` successfully overrides detection and decodes files using the specified encoding.
- **SC-005**: Explicit denied characters are reported as violations even when inside an allowed range, with 100% reliability.
- **SC-006**: Built-in malicious patterns (bidi controls) are always reported regardless of allow/deny mode configuration.
- **SC-007**: JSON output validates against the documented schema with zero field mismatches.
- **SC-008**: All CI workflow invocations use valid CLI flags and parse existing JSON fields.
- **SC-009**: Column numbers in violations match character-based positions for multi-byte content.
- **SC-010**: Exit code 3 is returned when errors occur without violations.
- **SC-011**: Scanning a repository with 1000+ files shows no per-file allocation of pattern data (static initialization verified in tests).
- **SC-012**: Binary files in scanned directories are skipped without producing scan errors.
- **SC-013**: Every CLI flag referenced in README and docs is accepted by the current CLI parser.
- **SC-014**: `cargo bench` compiles and runs all 6 benchmark targets without errors.
- **SC-015**: `cargo test` compiles and runs all test files under `tests/` without orphaned or stale modules.
- **SC-016**: Nix build completes without OpenSSL dependency.
- **SC-017**: Each of the 17 review findings has at least one dedicated test case verifying the fix.

## Assumptions

- The current parser schema (`[global]`, `[[rules]]`, `[languages.<ext>]`) is the correct target schema; the `init` command should be updated to match it.
- The spec JSON schema will be updated to match actual output (Option B from review) rather than rewriting the output to match the spec.
- Exit code 3 semantics: returned when `errors.len() > 0 && violations.is_empty()`. When both errors and violations exist, exit code 1 (violations found) takes precedence.
- Character-based column counting means Unicode scalar value positions (not grapheme clusters), as this aligns with most editor column counting.
- Binary detection improvements use multiple byte-level heuristics (consecutive-null and control-byte ratio) without requiring external dependencies.
- Stale/uncompiled tests will be removed rather than resurrected, unless they test functionality that still exists.
