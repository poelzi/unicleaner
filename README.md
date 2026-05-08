# Unicleaner

[![CI](https://github.com/poelzi/unicleaner/workflows/CI/badge.svg)](https://github.com/poelzi/unicleaner/actions/workflows/ci.yml)
[![PR Security Check](https://github.com/poelzi/unicleaner/workflows/PR%20Security%20Check/badge.svg)](https://github.com/poelzi/unicleaner/actions/workflows/pr-check.yml)
[![Release](https://github.com/poelzi/unicleaner/workflows/Release/badge.svg)](https://github.com/poelzi/unicleaner/actions/workflows/release.yml)
[![codecov](https://codecov.io/gh/poelzi/unicleaner/branch/main/graph/badge.svg)](https://codecov.io/gh/poelzi/unicleaner)
[![Crates.io](https://img.shields.io/crates/v/unicleaner.svg)](https://crates.io/crates/unicleaner)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

**Detect malicious Unicode characters in source code**

Unicleaner is a security-focused CLI tool that scans source code repositories to detect potentially malicious Unicode characters that could hide backdoors or exploits, including:

- **Zero-width characters** (U+200B, U+200C, U+200D, U+FEFF)
- **Bidirectional override characters** (U+202A-U+202E) - Trojan Source attacks
- **Homoglyphs** - visually similar characters from different scripts
- **Non-printable control characters** outside standard ASCII range

## Features

- 🔒 **Deny-by-default security** - only explicitly allowed characters pass
- ⚙️ **Configurable** - TOML-based configuration with language presets
- 🚀 **Fast** - parallel scanning with Rayon
- 🎨 **Colored output** - human-readable terminal output with automatic TTY detection
- 📊 **JSON output** - machine-parseable format for CI/CD integration
- 🔄 **Git integration** - scan only changed files in pull requests
- 🌍 **Multilingual support** - 50+ language presets for legitimate Unicode

## Installation

### Using Cargo

```bash
cargo install unicleaner
```

### Using Nix

```bash
nix run github:poelzi/unicleaner
```

### Using Docker

```bash
# Pull from GitHub Container Registry
docker pull ghcr.io/poelzi/unicleaner:latest

# Scan current directory
docker run --rm -v "$(pwd):/workspace" ghcr.io/poelzi/unicleaner:latest .
```

See [Docker Usage Guide](docs/DOCKER.md) for detailed instructions and CI/CD integration examples.

### As a GitHub Check

Use the published action in your repository workflow:

```yaml
name: Unicode Security Check
on:
  pull_request:
    branches: [main]

jobs:
  unicode-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: poelzi/unicleaner@v1
        with:
          mode: diff
          base-ref: main
          fail-on-violations: true
```

You can also call the reusable workflow directly:

```yaml
jobs:
  unicode-security:
    uses: poelzi/unicleaner/.github/workflows/unicode-check.yml@v1
    with:
      mode: diff
      base-ref: main
      fail-on-violations: true
```

### From Source

```bash
git clone https://github.com/poelzi/unicleaner
cd unicleaner
cargo build --release
./target/release/unicleaner --version
```

## Quick Start

```bash
# Scan current directory
unicleaner scan .

# Generate default config
unicleaner init

# Scan with custom config
unicleaner scan . --config unicleaner.toml

# Scan only Git changes (for CI/CD)
unicleaner scan . --diff

# Output JSON for machine parsing
unicleaner scan . --format json

# Filter by severity level
unicleaner scan . --severity error

# Control color output
unicleaner scan . --color always
unicleaner scan . --color never
unicleaner scan . --no-color  # deprecated but supported

# Quiet mode (summary only)
unicleaner scan . --quiet

# Verbose mode (show progress)
unicleaner scan . --verbose

# List available language presets
unicleaner list-presets
```

## Example Output

### Detecting Trojan Source Attacks

When scanning code with bidirectional override characters (like [CVE-2021-42574](https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2021-42574)):

```bash
$ unicleaner scan tests/fixtures/trojan_source.rs
```

```
🔍 Scanning: tests/fixtures/trojan_source.rs

❌ VIOLATION: tests/fixtures/trojan_source.rs:12:45
   Character: U+202E (RIGHT-TO-LEFT OVERRIDE)
   Category: Bidi Control
   Severity: ERROR
   Pattern: Bidirectional Override
   Description: Character can reorder text visually, potentially hiding malicious code

   Context:
   10 | fn is_admin(user: &str) -> bool {
   11 |     let access_level = check_user(user);
   12 |     if access_level == "admin"/*‮ }⁦if access_level != "user‭⁩ { // */
                                          ^
   13 |         return true;
   14 |     }

───────────────────────────────────────────────────────────────────────────────

Scan Result: FAILED
Files scanned: 1
Files clean: 0
Files with violations: 1
Total violations: 1

Severity breakdown:
  ERROR: 1
  WARNING: 0
  INFO: 0
```

### Detecting Zero-Width Characters

Scanning for invisible characters that could hide backdoors:

```bash
$ unicleaner scan tests/fixtures/zero_width.py --verbose
```

```
🔍 Scanning directory: tests/fixtures/

[1/3] tests/fixtures/zero_width.py
❌ VIOLATION: tests/fixtures/zero_width.py:5:23
   Character: U+200B (ZERO WIDTH SPACE)
   Category: Zero Width
   Severity: WARNING
   Pattern: Zero-Width Character
   Description: Invisible character that serves no legitimate purpose in code

   Context:
   3 | def authenticate(username, password):
   4 |     # Check credentials
   5 |     if username == "admin​":  # Zero-width space after admin
                              ^
   6 |         return check_admin_access(password)
   7 |     return False

[2/3] tests/fixtures/clean_file.rs ✓
[3/3] tests/fixtures/clean_file.py ✓

───────────────────────────────────────────────────────────────────────────────

Scan Result: FAILED
Files scanned: 3
Files clean: 2
Files with violations: 1
Total violations: 1

Severity breakdown:
  ERROR: 0
  WARNING: 1
  INFO: 0

Duration: 12ms
```

### Clean Repository Scan

When everything is safe:

```bash
$ unicleaner scan src/ --quiet
```

```
Scan Result: PASSED ✓
Files scanned: 42
Files clean: 42
Files with violations: 0

Duration: 156ms
```

### JSON Output for CI/CD

Machine-readable output for automation:

```bash
$ unicleaner scan suspicious.rs --format json
```

```json
{
  "violations": [
    {
      "file_path": "suspicious.rs",
      "line": 12,
      "column": 45,
      "code_point": 8238,
      "character": "‮",
      "category": "BidiControl",
      "severity": "Error",
      "pattern_name": "Bidirectional Override",
      "description": "Character can reorder text visually, potentially hiding malicious code",
      "context": {
        "before": "if access_level == \"admin\"/*",
        "match": "‮",
        "after": " }⁦if access_level != \"user‭⁩ { // */"
      }
    }
  ],
  "files_scanned": 1,
  "files_clean": 0,
  "files_with_violations": 1,
  "errors": [],
  "duration_ms": 8,
  "config_used": "unicleaner.toml"
}
```

### Self-Testing with Test Corpus

Unicleaner includes a test corpus with intentional malicious Unicode to verify detection:

```bash
$ unicleaner scan tests/fixtures/ 
```

```
🔍 Scanning: tests/fixtures/

❌ Found 12 violations in test corpus (expected for testing)

Test files intentionally contain malicious Unicode patterns:
  ✓ Trojan Source attacks (CVE-2021-42574)
  ✓ Zero-width characters
  ✓ Homoglyph attacks
  ✓ Non-printable control characters
  ✓ Mixed script confusables

This verifies that detection is working correctly!

───────────────────────────────────────────────────────────────────────────────

Scan Result: FAILED (as expected for test corpus)
Files scanned: 8
Files with violations: 8
Total violations: 12
```

### Git Diff Mode (CI/CD)

Only scan changed files in a pull request:

```bash
$ unicleaner scan . --diff
```

```
🔍 Git diff mode: scanning only changed files

Changed files in current branch:
  M src/auth.rs
  M src/utils.rs
  A tests/test_new_feature.rs

[1/3] src/auth.rs ✓
[2/3] src/utils.rs ✓
[3/3] tests/test_new_feature.rs ✓

───────────────────────────────────────────────────────────────────────────────

Scan Result: PASSED ✓
Files scanned: 3
Files clean: 3
Files with violations: 0

All changed files are safe to merge!
```

## CLI Reference

### Commands

- `scan [PATH]` - Scan files for malicious Unicode (default command)
- `clean [PATH]` - Sanitize a file or stdin and write to stdout (or `--in-place`)
- `init [FILE]` - Generate a default configuration file
- `list-presets` - Show available language presets

### Global Flags

- `-c, --config <FILE>` - Path to configuration file
- `-f, --format <FORMAT>` - Output format: human, json, github, gitlab (default: human)
- `--color <WHEN>` - Color output: auto, always, never (default: auto)
- `--no-color` - Disable color output (deprecated, use --color=never)
- `-q, --quiet` - Show only summary (suppress individual violations)
- `-v, --verbose` - Show verbose output with progress messages
- `--severity <LEVEL>` - Minimum severity to report: error, warning, info

### Scan Flags

- `--diff` - Scan only files changed in Git (requires Git repository)
- `-j, --jobs <N>` - Number of parallel threads (default: number of CPUs)
- `--encoding <ENC>` - Force specific encoding: utf8, utf16-le, utf16-be, utf32-le, utf32-be

### Clean Flags

- `--in-place` - Atomically rewrite the input file (write to `.tmp`, fsync, rename)
- `--policy <PRESET>` - Policy preset: `strict` (strip), `lossy` (replace with U+FFFD), `report-only`
- `--nfc` - Apply NFC normalization to the cleaned output

## Cleaning

Beyond detection, unicleaner can sanitize strings via the `clean` library
entry point and matching CLI subcommand.

**Library**:

```rust
use unicleaner::{clean, CleanPolicy};

let result = clean("hi\u{200B}there", &CleanPolicy::strict());
assert_eq!(result.output.as_ref(), "hithere");
assert!(result.modified);
assert_eq!(result.violations.len(), 1);
```

The clean-input fast path is zero-allocation: the result's `output`
borrows from the input when no codepoint matches the policy and NFC is
disabled.

```rust
use std::borrow::Cow;
use unicleaner::{clean, CleanPolicy};

let result = clean("plain ascii", &CleanPolicy::strict());
assert!(matches!(result.output, Cow::Borrowed(_)));
```

Per-category tuning:

```rust
use unicleaner::{clean, CleanAction, CleanPolicy};
use unicleaner::unicode::malicious::MaliciousCategory;

let policy = CleanPolicy::strict()
    .with_action(MaliciousCategory::BidiOverride, CleanAction::Replace('?'))
    .with_action(MaliciousCategory::Homoglyph, CleanAction::KeepWithMark);
```

**CLI**:

```bash
# Default: write cleaned content to stdout, source file untouched
unicleaner clean src/foo.rs

# Diff against the original
unicleaner clean src/foo.rs | diff src/foo.rs -

# In-place atomic rewrite
unicleaner clean --in-place src/foo.rs

# Policy presets
unicleaner clean --policy lossy src/foo.rs        # replace with U+FFFD
unicleaner clean --policy report-only src/foo.rs  # no mutation; exit 1 if found

# Reuse an existing config
unicleaner --config unicleaner.toml clean src/foo.rs
```

A configuration file may carry an optional `[cleaner]` block — see
`examples/unicleaner.toml`.

## Exit Codes

- `0` - Success: No violations found
- `1` - Violations found
- `2` - Error: Invalid arguments, file read errors, etc.

## Documentation

- [Quickstart Guide](specs/001-unicode-malicious-detector/quickstart.md)
- [Configuration Examples](examples/unicleaner.toml)
- [CI/CD Integration](examples/)
- [GitHub Action Guide](docs/GITHUB_ACTION.md)
- [Docker Usage Guide](docs/DOCKER.md)
- [Nix Build System](docs/NIX_BUILD_SYSTEM.md)

## Real-World Use Cases

### 1. Pre-Commit Hook

Prevent malicious Unicode from entering your repository:

```bash
# .git/hooks/pre-commit
#!/bin/bash
if command -v unicleaner &> /dev/null; then
    unicleaner scan --diff --severity error
    exit $?
fi
```

Or use with [pre-commit framework](https://pre-commit.com/):

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: unicleaner
        name: Unicode Security Scanner
        entry: unicleaner scan --diff --severity error
        language: system
        pass_filenames: false
```

### 2. GitHub Actions

Scan pull requests automatically:

```yaml
# .github/workflows/unicode-security.yml
name: Unicode Security Check
on:
  pull_request:
    branches: [main]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: poelzi/unicleaner@v1
        with:
          mode: diff
          base-ref: main
          fail-on-violations: true
```

### 3. GitLab CI

```yaml
# .gitlab-ci.yml
unicode-security-scan:
  stage: test
  image: ghcr.io/poelzi/unicleaner:latest
  script:
    - unicleaner scan . --format json > gl-code-quality-report.json
  artifacts:
    reports:
      codequality: gl-code-quality-report.json
```

### 4. Supply Chain Security

Scan third-party dependencies before integration:

```bash
# Scan a downloaded library
unicleaner scan vendor/suspicious-library/ --severity error

# Scan before npm/cargo/pip install
unicleaner scan package-to-audit/ && npm install
```

### 5. Code Review Tool Integration

Generate reports for code review platforms:

```bash
# GitHub format (for PR comments)
unicleaner scan . --format github > review-comments.json

# GitLab format
unicleaner scan . --format gitlab > gitlab-report.json
```

### 6. IDE Integration

VS Code task configuration:

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Unicode Security Scan",
      "type": "shell",
      "command": "unicleaner scan ${file} --color always",
      "problemMatcher": [],
      "group": {
        "kind": "test",
        "isDefault": false
      }
    }
  ]
}
```

### 7. Monorepo Scanning

Scan specific packages or services:

```bash
# Scan all services
for service in services/*; do
  echo "Scanning $service..."
  unicleaner scan "$service" --quiet || exit 1
done

# Scan only changed packages in monorepo
CHANGED_PACKAGES=$(git diff --name-only main... | cut -d/ -f1-2 | sort -u)
for pkg in $CHANGED_PACKAGES; do
  unicleaner scan "$pkg"
done
```

### 8. Automated Security Reports

Daily scans with notification:

```bash
#!/bin/bash
# daily-scan.sh

REPORT_FILE="scan-$(date +%Y%m%d).json"
unicleaner scan . --format json > "$REPORT_FILE"

VIOLATIONS=$(jq '.violations | length' "$REPORT_FILE")

if [ "$VIOLATIONS" -gt 0 ]; then
  # Send alert (Slack, email, etc.)
  curl -X POST "$SLACK_WEBHOOK" \
    -H 'Content-Type: application/json' \
    -d "{\"text\": \"⚠️ Found $VIOLATIONS Unicode violations in codebase!\"}"
fi
```

### 9. Release Verification

Ensure clean releases:

```bash
# Before tagging a release
unicleaner scan . --severity error
if [ $? -eq 0 ]; then
  git tag v1.0.0
  git push origin v1.0.0
else
  echo "❌ Cannot release: Unicode violations found!"
  exit 1
fi
```

### 10. Compliance Auditing

Generate compliance reports:

```bash
# Scan and generate audit report
unicleaner scan . \
  --format json \
  > compliance-report-$(date +%Y%m%d).json

# Convert to PDF for compliance documentation
jq -r '.violations[] | 
  "File: \(.file_path)\n" +
  "Line: \(.line)\n" + 
  "Issue: \(.pattern_name)\n" +
  "Severity: \(.severity)\n\n"' \
  compliance-report-*.json > audit.txt
```

## Development

### Setup

#### Using Nix Flakes (Recommended)

```bash
nix develop
```

#### Manual Setup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Test

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

Or use `just` recipes:

```bash
just build
just test
just check
just fmt-check
just build-static
just build-docker
just coverage
just fuzz fuzz-parallel-scanner 30
```

### Run

```bash
cargo run -- .
```

### Testing

Unicleaner has comprehensive test coverage including:

**Unit tests**:
```bash
cargo test --lib
```

**Integration tests**:
```bash
cargo test --test integration
```

**Property-based tests** (with proptest):
```bash
cargo test --test proptest
# Run with more cases
PROPTEST_CASES=10000 cargo test --test proptest
```

**Fuzz testing** (requires nightly Rust):
```bash
cargo +nightly fuzz run fuzz_unicode -- -max_total_time=60
cargo +nightly fuzz run fuzz_config -- -max_total_time=60
cargo +nightly fuzz run encoding_detection -- -max_total_time=60
```

**Performance benchmarks**:
```bash
cargo bench
```

**Code coverage**:
```bash
cargo tarpaulin --out Html
```

See [Testing Documentation](tests/proptest/README.md) for more details.

## License

Dual-licensed under MIT OR Apache-2.0

## Security

This tool helps detect Unicode-based security vulnerabilities. For security issues in the tool itself, please report responsibly via GitHub Security Advisories.

## References

- [Trojan Source: Invisible Vulnerabilities](https://trojansource.codes/)
- [Unicode Security Mechanisms (TR39)](https://unicode.org/reports/tr39/)
- [Unicode Confusables](https://www.unicode.org/Public/security/latest/confusables.txt)
