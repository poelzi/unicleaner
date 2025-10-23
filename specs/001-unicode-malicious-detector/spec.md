# Feature Specification: Unicode Malicious Character Detector

**Feature Branch**: `001-unicode-malicious-detector`  
**Created**: 2025-10-23  
**Status**: Draft  
**Input**: User description: "unicleaner is a unicode malicious sourcecode detector. It searches source code for hidden unicode character, characters that hide others. It is possible to define which languages and certain sets of characters are allowed per file or general. toml configuration files. Deny by default. Can be used as linter on changesets or whole repositories. Can be added as github and gitlab workflow step"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Repository for Malicious Unicode (Priority: P1)

A developer wants to scan their entire codebase to detect potentially malicious Unicode characters that could hide backdoors, such as zero-width characters, homoglyphs, bidirectional override characters, and other non-printable or visually deceptive Unicode sequences.

**Why this priority**: This is the core value proposition—detecting security vulnerabilities from malicious Unicode. Without this, the tool has no purpose. This is the MVP.

**Independent Test**: Can be fully tested by running the scanner on a repository containing files with known malicious Unicode characters and verifying that all malicious characters are detected and reported correctly.

**Acceptance Scenarios**:

1. **Given** a repository with files containing zero-width characters, **When** the scanner runs on the repository, **Then** all files with zero-width characters are flagged with specific line and column numbers
2. **Given** a repository with bidirectional override characters (U+202E), **When** the scanner runs, **Then** these characters are detected and reported as security risks
3. **Given** a clean repository with no malicious Unicode, **When** the scanner runs, **Then** the tool reports success with no issues found
4. **Given** a repository with homoglyph attacks (e.g., Cyrillic 'a' instead of Latin 'a'), **When** the scanner runs, **Then** suspicious character substitutions are flagged
5. **Given** a repository with mixed text encodings, **When** the scanner runs, **Then** all files are correctly processed regardless of encoding

---

### User Story 2 - Configure Language-Specific Allowed Characters (Priority: P2)

A developer working on a multilingual project needs to allow legitimate Unicode characters for specific programming languages (e.g., allow Greek letters in mathematical code, allow Cyrillic in Russian comments) while still blocking malicious Unicode elsewhere.

**Why this priority**: After basic detection (P1), users need configuration to reduce false positives in legitimate multilingual codebases. This makes the tool practical for real-world use.

**Independent Test**: Can be tested by creating a TOML configuration that allows specific Unicode ranges for specific file patterns, then verifying that legitimate characters pass while malicious ones are still caught.

**Acceptance Scenarios**:

1. **Given** a TOML config allowing Greek letters (U+0370-U+03FF) for .py files, **When** scanning Python code with Greek variable names, **Then** those characters are allowed and not flagged
2. **Given** a TOML config specifying allowed characters per file pattern (e.g., *.rs, *.py), **When** scanning matching files, **Then** only characters outside the allowed set are flagged
3. **Given** a TOML config with language presets (e.g., "allow-chinese", "allow-japanese"), **When** these presets are enabled, **Then** legitimate characters from those languages are permitted
4. **Given** no configuration file, **When** the scanner runs, **Then** deny-by-default behavior applies and only ASCII + minimal safe Unicode is allowed
5. **Given** a TOML config with per-file overrides, **When** scanning specific files with custom rules, **Then** file-specific rules take precedence over global rules

---

### User Story 3 - Lint Changesets in CI/CD Pipeline (Priority: P3)

A development team wants to integrate the scanner into their CI/CD pipeline (GitHub Actions, GitLab CI) to automatically check only changed files in pull requests and block merges if malicious Unicode is detected.

**Why this priority**: CI/CD integration automates security checks and prevents malicious code from entering the repository. This builds on P1 and P2 to provide continuous protection.

**Independent Test**: Can be tested by running the scanner in diff mode on a Git changeset, verifying that only modified files are scanned, and confirming that the exit code properly signals pass/fail for CI integration.

**Acceptance Scenarios**:

1. **Given** a Git repository with uncommitted changes containing malicious Unicode, **When** the scanner runs in changeset mode, **Then** only the changed files are scanned and issues are reported
2. **Given** a pull request with clean changes, **When** the CI workflow runs the scanner, **Then** the check passes and returns exit code 0
3. **Given** a pull request with malicious Unicode in changed files, **When** the CI workflow runs, **Then** the check fails with exit code 1 and detailed violation report
4. **Given** a GitHub Actions workflow configuration, **When** a PR is opened, **Then** the scanner runs automatically and posts results as PR check status
5. **Given** a GitLab CI configuration, **When** a merge request is created, **Then** the scanner runs and reports results in the pipeline log

---

### User Story 4 - Generate Human and Machine-Readable Reports (Priority: P4)

Users need clear, actionable reports showing exactly where malicious Unicode was found, with both human-friendly colored terminal output and machine-parseable JSON for automated processing.

**Why this priority**: Good reporting makes the tool usable. Users need to understand what was found and where. JSON output enables integration with other tools.

**Independent Test**: Can be tested by scanning files with known issues and verifying that both terminal output (with colors) and JSON output contain accurate file paths, line numbers, column numbers, character descriptions, and severity levels.

**Acceptance Scenarios**:

1. **Given** malicious Unicode detected in multiple files, **When** running in default mode, **Then** colored terminal output shows file paths, line numbers, character names, and visual indicators
2. **Given** the --json flag is specified, **When** the scanner completes, **Then** structured JSON output is written to stdout with all violations in parseable format
3. **Given** the --no-color flag is set, **When** output is generated, **Then** no ANSI color codes are emitted
4. **Given** output is redirected to a pipe or file, **When** the scanner runs, **Then** color is automatically disabled
5. **Given** multiple severity levels (error, warning, info), **When** reporting violations, **Then** each issue is tagged with appropriate severity
6. **Given** the NO_COLOR environment variable is set, **When** the scanner runs, **Then** colored output is suppressed

---

### Edge Cases

- What happens when scanning binary files or files with invalid UTF-8 encoding?
- How does the tool handle extremely large files (>1GB) or repositories with millions of files?
- What happens when the TOML configuration file itself contains malicious Unicode?
- How does the tool behave when file permissions prevent reading certain files?
- What happens when scanning symlinks or circular directory structures?
- How are performance and memory usage managed when scanning very large codebases?
- What happens when configuration allows overlapping or conflicting character ranges?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST scan text files in a directory tree recursively to detect Unicode characters flagged as malicious or suspicious
- **FR-002**: System MUST detect the following categories of malicious Unicode: zero-width characters (U+200B, U+200C, U+200D, U+FEFF), bidirectional override characters (U+202A-U+202E), homoglyphs commonly used in attacks, non-printable control characters outside standard ASCII control range
- **FR-003**: System MUST implement deny-by-default behavior where only explicitly allowed character ranges are permitted
- **FR-004**: System MUST read configuration from TOML files specifying allowed Unicode ranges globally and per-file-pattern
- **FR-005**: System MUST support language presets in configuration (e.g., allow-greek, allow-cyrillic, allow-chinese, allow-japanese) that map to safe Unicode ranges for those languages
- **FR-006**: System MUST allow file-pattern-based rules (glob patterns like *.rs, src/**/*.py) to specify which files use which character allowlists
- **FR-007**: System MUST support scanning modes: full repository scan, changeset/diff mode (scan only modified files), single file scan
- **FR-008**: System MUST report violations with exact file path, line number, column number, character code point (U+XXXX), character name, and severity level
- **FR-009**: System MUST output results in multiple formats: human-readable colored terminal output, JSON for machine parsing
- **FR-010**: System MUST respect color output controls: auto-detect TTY, support --color flag (auto/always/never), respect NO_COLOR environment variable
- **FR-011**: System MUST return appropriate exit codes: 0 for clean scan, 1 for violations found, 2 for configuration/runtime errors
- **FR-012**: System MUST provide example workflow files for GitHub Actions and GitLab CI integration
- **FR-013**: System MUST handle files with different encodings (UTF-8, UTF-16, Latin-1) or gracefully skip non-text files
- **FR-014**: System MUST allow users to specify custom character allowlists using Unicode range syntax (U+0000-U+007F)
- **FR-015**: System MUST support ignore patterns to skip specific files or directories (e.g., .git/, node_modules/, vendor/)
- **FR-016**: System MUST validate TOML configuration on startup and report clear errors for invalid config
- **FR-017**: System MUST provide a --generate-config flag to create a default configuration file template

### Key Entities

- **Scan Target**: Represents the input to be scanned (directory path, file path, or Git diff). Contains path, scan mode, and file inclusion/exclusion rules.

- **Configuration**: TOML-based settings defining allowed character ranges, language presets, file pattern rules, ignore patterns, and severity mappings. Can be global or per-file-pattern.

- **Character Rule**: Defines an allowed Unicode range (start codepoint, end codepoint) with optional description. Can be part of a language preset or custom rule.

- **Violation**: Represents a detected malicious Unicode character. Contains file path, line number, column number, character codepoint, character name, severity level, and rule violated.

- **Scan Result**: Aggregate result of a scan containing list of violations, summary statistics (files scanned, violations found, clean files), and overall pass/fail status.

- **Language Preset**: Named collection of character rules for a specific language/script (e.g., "greek" maps to U+0370-U+03FF). Reusable across configurations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can scan a 10,000-file repository and receive complete results in under 30 seconds on standard developer hardware
- **SC-002**: The scanner correctly identifies 100% of test cases containing the top 20 malicious Unicode patterns (zero-width, bidi, common homoglyphs) in the test suite
- **SC-003**: Users can configure language-specific character allowlists and reduce false positives by at least 90% for legitimate multilingual codebases
- **SC-004**: CI/CD integration examples work out-of-the-box for GitHub Actions and GitLab CI with zero modification required
- **SC-005**: JSON output validates against a published JSON schema and contains all required fields (file, line, column, codepoint, severity)
- **SC-006**: Tool correctly handles repositories up to 100,000 files without crashes or excessive memory usage (stays under 500MB RAM)
- **SC-007**: Colored output is never emitted when stdout is redirected to a file or pipe (auto-detection works 100% of the time)
- **SC-008**: Users can generate a working default configuration file and customize it to their needs in under 5 minutes
- **SC-009**: 95% of users successfully integrate the tool into their CI pipeline on first attempt using provided examples
- **SC-010**: All configuration errors produce clear, actionable error messages that guide users to fix the issue

### Assumptions

- Users have basic familiarity with command-line tools and configuration files
- Repositories primarily contain UTF-8 encoded text files
- Git is available for changeset/diff scanning mode
- CI/CD environments support running arbitrary command-line tools
- Standard Unicode character database definitions are used for character categorization
- Default deny-list covers the most common attack vectors documented in Unicode security reports
