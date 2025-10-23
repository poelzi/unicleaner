# Quickstart Guide: Unicode Malicious Character Detector

## Installation

### Using Cargo (Rust package manager)
```bash
cargo install unicleaner
```

### Using Nix
```bash
# Run directly
nix run github:yourusername/unicleaner

# Install to profile
nix profile install github:yourusername/unicleaner

# In development shell
nix develop
```

### From Source
```bash
git clone https://github.com/yourusername/unicleaner
cd unicleaner
cargo build --release
./target/release/unicleaner --version
```

## Basic Usage

### 1. Scan Your First Repository

Scan the current directory for malicious Unicode:
```bash
unicleaner .
```

Expected output (clean scan):
```
✓ Scanned 150 files in 0.5s
✓ No malicious Unicode detected
```

Example output (violations found):
```
error: Zero-width space detected
  --> src/main.rs:42:15
   |
42 | let user​name = "admin";  // <- invisible U+200B here
   |            ^
   = note: Zero-width characters can hide malicious code

warning: Homoglyph detected
  --> src/auth.rs:18:9
   |
18 | let аdmin = true;  // <- Cyrillic 'а' instead of Latin 'a'
   |     ^
   = note: Character 'а' (U+0430) looks like 'a' (U+0061)

✗ Found 2 violations in 2 files (scanned 150 files in 0.5s)
```

### 2. Create a Configuration File

Generate a default configuration:
```bash
unicleaner init
```

This creates `unicleaner.toml`:
```toml
# Unicleaner configuration

[default]
# Global allowed character sets
allow = ["ascii"]  # Only allow ASCII by default

# Always deny these regardless of other rules
deny = [
    0x200B,  # ZERO WIDTH SPACE
    0x200C,  # ZERO WIDTH NON-JOINER
    0x200D,  # ZERO WIDTH JOINER
    0xFEFF,  # ZERO WIDTH NO-BREAK SPACE
    0x202A,  # LEFT-TO-RIGHT EMBEDDING
    0x202B,  # RIGHT-TO-LEFT EMBEDDING
    0x202C,  # POP DIRECTIONAL FORMATTING
    0x202D,  # LEFT-TO-RIGHT OVERRIDE
    0x202E,  # RIGHT-TO-LEFT OVERRIDE
]

[ignore]
# Skip these patterns
patterns = [
    ".git/**",
    "node_modules/**",
    "target/**",
    "vendor/**",
    "*.min.js",
    "*.lock",
]

[reporting]
severity_threshold = "warning"  # error, warning, or info
```

### 3. Allow Specific Languages

Edit `unicleaner.toml` to allow legitimate Unicode for your project:

```toml
[default]
# Allow ASCII plus specific languages
allow = ["ascii", "emoji"]

# Rules for specific file patterns
[[file_rules]]
pattern = "docs/**/*.md"
allow = ["ascii", "emoji", "chinese", "japanese"]

[[file_rules]]
pattern = "src/**/*.rs"
allow = ["ascii", "greek"]  # Greek letters for math code

[[file_rules]]
pattern = "i18n/**/*.json"
allow = ["ascii", "latin-extended", "cyrillic", "chinese", "japanese", "korean", "arabic"]
```

### 4. Scan with Custom Configuration

```bash
unicleaner . --config=unicleaner.toml
```

### 5. CI/CD Integration

#### GitHub Actions

Create `.github/workflows/unicode-scan.yml`:
```yaml
name: Unicode Security Scan
on: [pull_request]

jobs:
  unicode-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install unicleaner
        run: cargo install unicleaner
      
      - name: Scan for malicious Unicode
        run: unicleaner . --diff --json
```

#### GitLab CI

Add to `.gitlab-ci.yml`:
```yaml
unicode-scan:
  stage: test
  image: rust:latest
  before_script:
    - cargo install unicleaner
  script:
    - unicleaner . --diff --json
  only:
    - merge_requests
```

### 6. Scan Only Changed Files

For pull requests, scan only modified files:
```bash
unicleaner . --diff
```

### 7. JSON Output for Automation

Get machine-readable JSON output:
```bash
unicleaner . --json > results.json
```

Example JSON output:
```json
{
  "version": "1.0.0",
  "timestamp": "2025-10-23T10:30:00Z",
  "summary": {
    "files_scanned": 150,
    "files_clean": 148,
    "files_with_violations": 2,
    "total_violations": 3
  },
  "violations": [
    {
      "file": "src/main.rs",
      "line": 42,
      "column": 15,
      "code_point": "U+200B",
      "character_name": "ZERO WIDTH SPACE",
      "severity": "error",
      "pattern": "zero-width-space",
      "message": "Zero-width characters can hide malicious code"
    }
  ]
}
```

## Common Scenarios

### Strict Security (No Unicode)
```toml
[default]
allow = ["ascii"]  # ASCII only, no other Unicode
```

### Multi-language Documentation
```toml
[[file_rules]]
pattern = "**/*.md"
allow = ["ascii", "emoji", "chinese", "japanese", "korean"]
```

### Scientific Computing (Greek/Math Symbols)
```toml
[[file_rules]]
pattern = "**/*.py"
allow = ["ascii", "greek", "latin-extended"]
```

### Web Development with I18n
```toml
[[file_rules]]
pattern = "src/**/*.js"
allow = ["ascii"]  # Code stays ASCII

[[file_rules]]  
pattern = "locales/**/*.json"
allow = ["ascii", "latin-extended", "cyrillic", "chinese", "japanese", "korean", "arabic", "hebrew"]
```

## Command Reference

### Essential Commands
- `unicleaner .` - Scan current directory
- `unicleaner init` - Generate config file
- `unicleaner --help` - Show all options
- `unicleaner list-presets` - Show available language presets

### Key Options
- `--config=FILE` - Use custom config file
- `--diff` - Scan only Git changes
- `--json` - Output JSON format
- `--color=never` - Disable colors
- `--severity=error` - Only show errors
- `--verbose` - Show scanning progress

## Troubleshooting

### "Binary file detected" errors
Some files are binary and can't be scanned. Add them to ignore patterns:
```toml
[ignore]
patterns = ["**/*.png", "**/*.jpg", "**/*.pdf"]
```

### False positives in legitimate multi-language files
Configure language-specific rules:
```toml
[[file_rules]]
pattern = "translations/**/*"
allow = ["ascii", "latin-extended", "cyrillic"]  # Add languages you need
```

### Scanning is slow
Use parallel threads:
```bash
unicleaner . --threads=8
```

Or limit file size:
```bash
unicleaner . --max-file-size=10  # Skip files >10MB
```

### Exit Codes

- `0` - Success, no violations found
- `1` - Violations detected
- `2` - Configuration or runtime error
- `3` - Some files couldn't be scanned

Use in scripts:
```bash
if unicleaner . --quiet; then
    echo "✓ No malicious Unicode found"
else
    echo "✗ Security issues detected"
    exit 1
fi
```

## Next Steps

1. **Run your first scan**: `unicleaner .`
2. **Create configuration**: `unicleaner init` and customize for your project
3. **Add to CI/CD**: Use the examples above for GitHub/GitLab
4. **Learn more**: Run `unicleaner --help` for all options

## Getting Help

- Run `unicleaner --help` for command documentation
- Check exit code meanings with `echo $?` after running
- File issues at: https://github.com/yourusername/unicleaner/issues